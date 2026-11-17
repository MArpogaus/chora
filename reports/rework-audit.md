# Rework audit: where we did the same work several times (owner question 11-17)

Read-only audit, 2026-11-17. Sources: session log (reports/, findings/, friction.md, backlog.md, briefs/) and
`git log origin/claude/autonomous-dev-session-3w-i33pg7` (1921 commits, 2026-10-13 to 11-17, 203 merges; nothing built or run).
Counts come from commit-subject greps and are approximate (a grep over-counts: "give" also matches drawing code). "Saved" figures are my
rough estimate, not measured. file:line refers to the head `fe8ba6d0` as read by reports/architecture-rc.md (called ARC below); I did not re-read the code.
Overall: the plan in ARC covers most code duplication. It does not cover the process side (section 2) or test and tour tooling well.

Size signals: commits per file (whole branch): plugins/chora-mc/src/main.rs 103, chora-xp/src/wm.rs 81, chora-tiling/src/main.rs 80,
docs/configuration.md 216, docs/api.md 163, tests/xp.sh 52, tests/merge-gate.sh 46. Some of this is honest feature work, not rework.
Commit scopes: fix(voice) 83, fix(core) 73, fix(construct) 73, fix(mc) 59, fix(xp) 46, fix(tiling) 44, fix(assistant) 43.
641 of 1921 subjects start with "fix". Briefs per lane: xp-polish r1-r7, mc r1-r6, flakes r1-r6, harness r1-r5 (+rc r1-r2), ux r1-r5,
security-a r1-r5, ci-e2e r1-r5, media r1-r5, construct r1-r4, doors r1-r4, tiling-look r1-r4. 98 review-*.md exist; about 36 carry a FIX verdict (grep, line-based).

## 1. Top 15 rework clusters

### C1. Injected-input refusal, fixed look by look and path by path
- Done repeatedly: security-a r1-r5 (5 rounds, 5 reviews), then 56 commits mentioning "injected" (11-09: 5, 11-10: 21, 11-11: 23, 11-15: 30, 11-16: 17).
  The same fix was made per look on one day (11-15): xp `78d79db7`, tiling `be39d1bd`, construct `cf7e5e9c`, mc `dfef5333`, `5e36eb46`; also notify `2f147e42`, chat `70f8d1f2`.
- Evidence: friction.md 51 (r1 and r2 "each fixed the named path; each review found the same class elsewhere") and 52 (fourth reopening: events without the flag);
  briefs/security-a-r4.md ("close the class in the type, not per path"); ARC row 12, AR5, debt-rc RF5 (xp forgot pointer filtering until `78d79db7`);
  sh-24 (stray keys, review-multilook-r1 point 1); lint `tests/injected-optout.py` (`85476db0`) written to catch handlers that drop the flag.
- Design cause: the policy "ignore injected input" lived in every handler of every plugin, and the event type did not carry the flag. Each new look or event copied the hole.
- Change: flag in the event type and one filter in kit dispatch (what r4 and AR5 do), done before the second look existed.
- Saved (rough): 3 of 5 fix rounds with their reviews, about 20 of the 56 commits, one reopened friction entry (51, 52).

### C2. System actions and "doors" (lock, power, screenshot, volume, clock, launcher, helper), rebuilt per look
- Done repeatedly: findings sh-1..sh-11, all MAJOR, all "exists in tiling, missing in xp/mc/construct" (findings/shared.md; findings/shared-doors.md table);
  doors r1-r4 (3 of 4 reviews FIX: review-doors-r1 dr-1/dr-2 BLOCK, r2 dr2-1 BLOCK, r3 dr3-1 MAJOR); 53 commits with "doors" in the subject;
  tiling-look r1-r4 (power menu `8fcafcde`, scripts `5e236257`, move `af2d0567`, `aa245abd` gate); the screenshot notice was fixed three times in tiling
  (`b1412720` 11-16, `29ee9ebc` 11-17, "Print notice on its own bus" in merge `7e066694`) and seen as open in tour v5, v6 (review-tour-v6 row j1-5) before v7 showed it fixed.
- Design cause: each look has its own Action enum and spelling; xp has no Action enum at all, its keys are hard-coded (ARC row 2, wm.rs:3825-3864; row 3: `Keys/ShowKeys/Help`, `Volume(Change)` vs `VolumeUp/Down/Mute`).
  A system action was a look feature, not a kit function with a result notice. Only `volume.rs` was shared early (`a56ab019`) and it went smoothly in all four.
- Change: kit `actions` with one `System` enum and `perform()` that returns a notice, built when the first look (tiling) got Lock/Power. Doors stay per look (drawing), actions do not.
- Saved (rough): doors r1 to r3 would shrink to "draw the door" (1 to 2 rounds instead of 4), about 10 commits of the tiling power/screenshot chain, 2 of the 5 tour re-records (j1-5 only).

### C3. Key list, key spelling, key wording
- Done repeatedly: four different keys opened the list (sh-6); give keys missing from two lists (sh-7, `5aaf7a13`, `b1a185d7`); volume wording (`efac1ed2`, `3b7d7f4c` "ask every key list for one volume wording");
  "spell keys alike" (`bfb48e3d`); long keys overflow in mc (`35bb2613`, `6009cebc`); tests were then added per list ("ask every key list for ..."), so one wording fix costs a test per look.
- Evidence: ARC row 4 (help.rs:72/130, keys.rs:255/314, keys.rs:80/122/214, hints.rs); sh-16 launcher words differ; sh-13/sh-18 spelling.
- Design cause: no shared model of "binding -> row -> words"; each look builds its own key sheet from its own enum.
- Change: kit `keysheet` (ARC step 7). Needs C2 first.
- Saved (rough): 6 to 8 commits and one cross-look gate cell family; no round on its own.

### C4. Notices: three servers and a fourth path
- Done repeatedly: ~24 notice commits across looks: xp balloon (`165a1105`, `4a9a7fe2`), mc card (`3c19f217`, `cf6998c2`, `346a6a98`), tiling no-bus notice (see C2), `notify.clear` privileged (`ac90a07b`), body cap (`961a24ed`),
  an injected press on the card (`2f147e42`); xp Notepad fallback notice told success before the window existed: review-xp-polish-r5 MAJOR 1, fixed in r6 (`c7e4e59e`).
- Evidence: ARC row 7 (xp chora-notify via zbus; tiling mako or `notify-send`; mc own `notify.push` server main.rs:1275; construct `hud.notice` hud.rs:1091); sh-18 "notification shape differs per look".
- Design cause: no single notification path; three implementations, each with its own limits, guard and failure message. Failures (no bus) were fixed in one and re-found in the next.
- Change: kit `notice` plus chora-notify as the only server (ARC step 10). I note ARC itself is unsure for mc (notices inside its content flow).
- Saved (rough): 8 to 10 commits, one review FIX (xp-polish r5 is partly a launch-state bug, see C13).

### C5. Helper presenter and the state word ("No helper")
- Done repeatedly: "No helper" wording fixed in `03e24c1b`, `279e4b72`, `b6354fbd`, `f681757a`, `5dc29b11`, `ea47e27b`; review-doors-r3 dr3-1: construct island says it only when `agent_list` is empty while xp, mc and opener.rs say it whenever no helper runs;
  tour v5 j1-4 "No helperVolume 40%" run together; sh-13 (state word spelled three ways, surface counted twice), sh-9, sh-10. About 48 commits mention helper.
- Design cause: three helper UIs (assistant, mc agent.rs 1,082 lines, construct own voice HUD when `voice_hud: Own`; ARC row 6) with separate state logic.
  Partly deliberate: friction 29 says one generalized HUD gave foreign-looking surfaces, so per-look HUDs came back. The cost is that the state logic is copied too.
- Change: one presenter/state machine (chora-assistant or kit) that emits state words; looks only supply a costume or slot (ARC step 11, marked L and unsure). Do not read this as "one HUD": friction 29 argues against that.
- Saved (rough): 5 to 8 commits and one MAJOR review round; hard to say more because part of the cost is real design work.

### C6. Safety question (confirm): three copies, two of them unsanitized
- Done repeatedly: sh-26 (mc agent.rs and construct hud.rs show confirm text unsanitized, no 400 ms GUARD; review-security-confirm.md MAJOR), sh-27 (hard-coded Ja/Nein in both),
  secfix-c r1 and r2 (2 rounds, 2 reviews), RF3 (answer wire builder in each HUD). The sanitize fix in review-security-confirm point 5 is listed in 5 places.
- Design cause: kit `ask.rs` exists, but only chora-assistant uses it; mc and construct re-implemented the question (ARC row 5). A security rule was therefore written in the wrong place twice.
- Change: kit `ask` as the only question model (ARC step 9); a lint like injected-optout for "confirm text must pass sanitize".
- Saved (rough): about 1 fix round and 6 to 8 commits; the bug itself would not have existed in mc and construct.

### C7. Window lifecycle and handover tested and fixed per look and per pair
- Done repeatedly: handover matrix gate of 12 pairs by 7 paths (`371bfbdf`, then 8 more handover test commits, `gate-handover-matrix.sh`, `-virtual.sh`, `gate-mc-handover.sh`, `df6dc69b`, `f6c09e5b`);
  per-look fixes: tiling `1275e151`, construct `674a7a94` and `5e2e979d`, mc review-mc-r4 A (focus under chat, FIX), merge `60c214f5` focus-flake; tour handover chapter in every tour (v2 to v7);
  tour v5 rv5-2 "mc leg: window never drawn for ~6 s".
- Design cause: the claim filter `to != NAME`, detach-is-free and primary-output logic exist four times (ARC row 1: wm.rs:768-810, main.rs:648-720, main.rs:815-850, plugin.rs:22-80). Every new look pair is a new test pair (4 looks give 12 ordered pairs).
- Change: kit `book` (ARC step 8, ranked first) plus FakeCore tests, so the pair matrix becomes one unit test over one state machine and a few shell cells for real windows.
- Saved (rough): about 6 commits of per-look focus fixes; the shell matrix would shrink to cross-process cells only.

### C8. Construct pane/cube state in loose booleans (re-found by flakes, reviews, tour)
- Done repeatedly: about 100 commits match cube/seam/stand; construct r1-r4 and review-construct-r1..r3 (+v2), flakes F13 and F14 (flakes-r2/r3 reports: "cause: Host::press took a press only while the cube rested"; "Construct::land asked every floating face in one round");
  M1 stood-back cube fixes (`8e6a0ae3`, `83c44f81`), `pane_drawn` re-fixes (`d33bbc8e`); cube seams missed by reviews v6, v7 and the PM until the owner saw them (friction.md last section, 11-17 10:40);
  review-construct-r2 MINOR 1: a gate cell green for the wrong reason (never asserts the mode).
- Evidence: debt-rc RF7 (construct `Pane.placed` main.rs:233, mc chat focus: "five review rounds"); ARC AR14 and section 8 (`placed` not saved, file.rs:27).
- Design cause: states derived from several loose flags (`placed`, `taken`, `attached`; mc chat `focused_under_chat`) with no transition table, so each change breaks a neighbour.
- Change: enums with transition-table tests (AR14). This was needed at the first pane/cube commit; I count it as design debt in two looks only.
- Saved (rough): 3 to 5 fix commits and 1 or 2 review rounds in mc and construct.

### C9. Test harness: 8 fake cores, copied waits, shell gates for scene facts
- Done repeatedly: flakes r1-r6 (6 rounds, F13 to F28, many "cause: the gate, not the product": F15 flakes-r3 item 3; F26 flakes-r6); `until_ok` copied into 23 scripts (git grep on the head; ARC counts 21), 775 `sleep` lines (ARC AR9), 8 fake cores (ARC row 13);
  `tests/merge-gate.sh` and `gate-lib.sh` touched 73 times; its "# Steps:" header conflicts at every train that adds a step (friction 55: trains 27 and 31; review-tiling-look-r2 rv-1 again);
  64 `gate-*` scripts added; ci-e2e r1-r5 and ci r1-r4 mostly gate plumbing (review-ci-e2e-r5: `skip` exits 0 under CI).
- Design cause: no shared test kit (FakeCore, wait helpers); behaviour that is a scene fact is checked in shell with real processes and sleeps, so it is slow and flaky under load (F16, F26 rates 11 of 102 failed before fix).
- Change: `kit::testing` FakeCore first (ARC step 1, AR9), shared waits in gate-lib, step table derived from one list.
- Saved (rough): 2 to 3 of the 6 flakes rounds, most of the merge-gate header conflicts (at least 3 trains), several gate re-runs per train.

### C10. Tour harness: one script set copied per version (see section 2 for the process side)
- Done repeatedly: tour v1, v2 (reports/tour-handover.md), v3 (media-r1; its scripts ch1..ch7 never ran for v2 looks, then redone), v4 to v7: `v4_*.py`, `v5_*.py`, `v6_*` "copies of v5 with the tiling companion changed" (media-r4), v7 (media-r5). 5 media rounds, 3 independent tour reviews (v5 to v7), and 3 findings of the owner or PM on top.
- Design cause: tours are scripts keyed to each version's paths, with no reuse of the gate drivers (tests/doors/*.sh) and no check of the session setup.
- Change: see section 2 (driver library plus a setup self-check). Not in ARC.
- Saved (rough): 2 of the 5 recordings (v4 to v7 differ by a few paths per media-r3/r4/r5).

### C11. God files: every feature lands in the same four files and the same docs
- Evidence: wm.rs 4,794 lines and 81 commits, mc main.rs 4,606 and 103, tiling main.rs 3,533 and 80 (head counts by me); docs/configuration.md 216 commits; ARC AR7, AR8, RF6.
  The 11-14 to 11-17 lanes (xp-polish, tiling-look, mc, doors) merge-conflict in these files and in docs/configuration.md; review-tiling-look-r2 reports a real conflict, review-xp-polish-r2 a "noisy three-dot diff, merge needs redoing".
- Design cause: no module boundary inside each look; parallel lanes cannot work on one look without touching the same file. Docs for config are hand-written per look (ARC step 6 and D7 propose generation).
- Change: step-0 moves (AR7, AR8) and generated config docs. The size is partly feature growth, so the saving applies to merge friction and review cost, not to feature work.
- Saved (rough): unknowable from commits; I would claim merge conflicts and review noise only.

### C12. Config sections drift per look
- Evidence: ARC row 9 (`test_accept_injected_keys`, programs, terminal, clock, bindings in four config.rs); xp has no `bindings` section (debt-rc D7); `1615316b` "list test_accept_injected_keys per window manager"; sh-28 stale doc.
- Design cause: no shared settings schema; each look re-declares the common sections and the docs list them again.
- Change: kit `settings::Common` (ARC step 6, D7). Saved (rough): 3 to 5 commits plus doc drift.

### C13. Launching a program and waiting for its window, per look
- Done repeatedly: xp `1f32f322`, `0d92149b` "no false launch balloons", `92e75397` merge, launch.rs 618 lines; construct `f9d4e657` "a launcher under a forking shell keeps its window"; mc `b69e87e7` "wait anew for ended apps after each launch";
  kit launcher fixed in two steps (`7beef6ed`, `38e09511`) after xp and tiling each wrapped shared entries; review-doors-r3 dr3-3 "entry_dirs repeats the rule of shared"; xp-polish r5 MAJOR 1 (notice said success before the editor window existed).
- Evidence: ARC row 8 (xp launch.rs, tiling 10 `Command::new`, mc fonts.rs 5, construct cube.rs:166), D1 in debt-rc.
- Design cause: no shared "pending launch" model (token, pid, unit, timeout, window arrival); each look tracks it with its own rules.
- Change: kit `launch` (ARC step 9, D1 pidfd). Saved (rough): 4 to 6 commits and one review FIX.

### C14. Small copies: language, runtime dirs, `on_path`, HUD themes
- Evidence: RF1 (language in 4 places; tiling theme.rs:707 buggy: empty `LC_ALL` masks `LANG`, ARC section 8), RF4/ARC row 11 (`XDG_RUNTIME_DIR` read at 8+ sites with different safety checks; kit icons.rs:76 falls back to the temp dir: secfix-a r2), `on_path` (2 production copies, ARC AR4);
  HUD dark colours per folder (`eb38e8d7`), sh-17 (HUD colours per theme).
- Design cause: no foundation modules (`paths`, `words`) in chora-scene/kit, so security and locale rules were applied inconsistently (secfix-a r1 and r2 found path issues the same way as C1).
- Change: ARC step 5. Saved (rough): 1 secfix round and 4 commits; small each, but they bring security rework.

### C15. Costume folders and look names inside the shared assistant
- Evidence: ARC row 14 and section 3: `plugins/chora-assistant/huds/{mc,construct,xp}` are identical to the look folders (`diff -rq`), tiling's costume exists only there; assistant main.rs:90-94 subscribes to each look's theme event by name; folders.rs `SOURCE`; PKGBUILD:105.
  Every HUD colour or layout fix touches two copies (`eb38e8d7`, `6a07dd6e`; 43 fix(assistant) commits).
- Design cause: the plugin ships the looks it serves, instead of each look shipping its costume.
- Change: ARC section 3 points 1 and 2, step 3 and 4. Evidence on commit counts is weak: I cannot separate copy edits from other assistant fixes. Saved: not estimated.

## 2. Process rework (not code)

P1. Review rounds that repeat. xp-polish r1-r7 (7 rounds; r5 FIX with 2 MAJOR), mc r1-r6 (r4 FIX), doors r1-r4 (3 FIX), ux r1-r5, harness r1-r5 (several FIX with B1/M1), security-a r1-r5.
  Causes seen: (a) briefs listed findings, not the invariant (friction 51, 52); (b) reviews judged against the shared HUD note, not the look's own document (friction 29, fixed 11-15 by design/looks/);
  (c) lanes kept pushing after their review and the train merged the branch head, not the reviewed commit (friction 55: xp-25 and xp-26 landed unreviewed in train 26; review-xp-polish-r2 could not check them);
  (d) a gate cell green for the wrong reason (review-construct-r2 MINOR 1). Fixes: invariant plus lint gate in every security brief (done in r3/r4), train reads the head from the review report (planned, "to build"), cells assert the state they depend on.
P2. Tour re-records: v4, v5, v6, v7 recorded after v3 (5 media rounds; media-r4 "re-recorded unchanged in path"). Several were re-recorded only to see whether one MAJOR was fixed (j1-5 across v5, v6, v7).
  Fix: confirm a tour finding with the single gate cell or a still first (doors gates already exist: tests/doors/*.sh); record the full tour only when paths changed.
P3. Harness faults read as product bugs. review-tour-v7 F-1: the recorder's "new MAJOR" (no Notepad notice) was a harness artefact: `lowriter` opened no window in the headless session, so the notice never fired (media-r5);
  tiling bar missing in the handover session (media-r4: multi.py starts only chora-tiling); F15 "cause found: the gate, not the product" (flakes-r3); F26 and F28 gate causes (flakes-r6); F16 doors step all "missing" then passed alone on the same tree (backlog F16).
  Cause: no setup self-check and no distinction between "harness failure" and "product failure" in reports. Fix: the session helper prints a header (tools present, bar running, editor stub) and a tour row can be marked "setup" in the report.
P4. Lane container drift read as red: friction 54 and F27 (fuzzel 1.9.2 vs 1.11.1; doors FAIL in xp-polish r2 with no grim); review lane `git checkout --detach` in the PM worktree broke hooks (friction, 11-16 10:06);
  disk full during trains (friction 48, 53); two gates at once (26). Fix already agreed: trains in the PM container (D4), disk line in routines.md, gate lock `e64a6b59`; F30 df check not built yet.
P5. Reviews that checked listed rows only: owner found cube seams 11-17 10:40 that reviews v6, v7 and the PM missed (friction, last entry). Fix named there: fresh-eyes pass per look against the vision card and a seam gate cell (construct r4).
P6. Budget and start churn: all lanes on opus after Fable ended (friction 50); PENDING session starts 11-16 (friction, "Recurrence"). Fix: Model line in briefs (done); nothing I can add on service-side delays.
P7. Records written too early: decisions lane wrote D1-D8 as open while the owner was answering (friction 11-17 04:4x); PM "Now" times guessed twice. Small; guard: brief a records lane after the owner's answers.

## 3. Mapping to reports/architecture-rc.md

| Cluster | ARC status | Where |
|---|---|---|
| C1 injected input | partly | AR5 (kit dispatch, M, sprint 2) and RF5; the type change and lint already exist (security-a r4, `85476db0`); pointer filtering still per plugin per ARC |
| C2 system actions and doors | covered | row 2, 3; `kit::actions`, step 6. Missing: a rule that every new door has a notice on failure (the no-bus case) |
| C3 key sheet | covered | row 4, `kit::keysheet`, step 7 |
| C4 notices | covered | row 7, `kit::notice` and chora-notify, step 10; mc inside-flow case marked unsure |
| C5 helper presenter and state word | partly | row 6, step 11 (L, unsure). Does not say who owns the state-word logic if per-look HUDs stay (friction 29) |
| C6 safety question | covered | row 5, step 9, section 8 bugs (sanitize, Ja/Nein) |
| C7 lifecycle and handover | covered | row 1, `kit::book`, step 8. The shell handover matrix is not mentioned; AR9 only says handover gates stay in shell |
| C8 pane/cube loose flags | covered | AR14 (M, sprint 2) and section 8 `placed`; no mention of the cube/seam gate cells |
| C9 test harness | partly | AR9 and step 1 (FakeCore, `until_ok`, sleeps). Not covered: merge-gate header conflict (debt/friction 55, F30), "cell green for the wrong reason", derived step table |
| C10 tour harness | not covered | no mention of demo/tour in ARC |
| C11 god files and docs | partly | AR7, AR8, AR10 cover files; generated config docs only via D7; docs/configuration.md churn not discussed |
| C12 config drift | covered | row 9, `settings::Common`, D7 |
| C13 launch and arrival | covered | row 8, `kit::launch`, D1 (A', pidfd) |
| C14 small copies | covered | rows 10, 11, AR4 (and RF2 correction) |
| C15 costume folders | covered | row 14, section 3 points 1 and 2, steps 3 and 4 |

Not covered by ARC: all of section 2 (P1 to P7) except the workflow lines in ARC section 10 ("every new shared part names the look code it deletes"; "an audit names the commit it read"), and C10.
Suggested additions: (1) a rule that every feature touching two or more looks starts as a kit function with a test over FakeCore, then one look, then the rest (this would have prevented C2, C3, C4, C6 from repeating per look);
(2) a tour driver library plus a setup self-check, and "confirm by gate cell before re-recording" (C10, P2, P3); (3) derive the merge-gate step list from one table (friction 55);
(4) brief lines for any bug class: invariant, enforcing gate, and "check the same class in the other three looks" (P1a).

## Limits and uncertainty
- Commit counts are subject greps, not diffs; "saved" numbers are estimates, rough to within a factor of two.
- I did not read code; file:line citations for code come from architecture-rc.md (head `fe8ba6d0`), checked only where I counted lines or scripts myself (file lengths, `until_ok` 23 scripts, gate counts).
- Several clusters are partly legitimate feature work (C5, C8, C11). I rank C1, C2, C9 highest on evidence: they have explicit friction entries or repeated same-day fixes in all four looks.
