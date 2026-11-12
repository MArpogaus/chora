# State

last_reviewed: a03be22 (11-10: every branch merged since df6dc69 had its own review before the merge: acp follow-ups, H1, H3, H4c, focus fix, 3D steps 2 to 6, H2, H4a, deps, H5, H6, H4b, mc clock)
base_2d: the commit before 0153f0d (the 2D guard); no scene or renderer change since
block: none (11-11 17:40 UTC)

pm: session_016iS3kQmq6Q6Ce3hxtenYSR
session branch HEAD: 59df16e (mc v2 and HUD host merged 11-11 ~23:30 UTC after tiling v2 fe3483f; merge-gate 17/17 PASS each)

## Now (11-11 17:40 UTC; sections "Plan of the day" and "Next step" below are of 11-10 and superseded)

Swarms (11-11): swarm 3 found 6 new verified findings; swarm 4 (mouse only) found 0 new (13 duplicates), so the swarms stop per the user's rule (test only while new issues come). Triage in reports/swarm-2/findings.md.

Goal until Saturday (user 11-11): UI/UX polish toward perfection, each look with its own consistent aesthetic; the PM works alone and decides taste.

Running:
- Design passes done for all four looks (design/looks/*-v2.md; PM taste decisions in design/decisions-taste.md).
- mc-v2 + HUD host: round 3 (d9a8a72) UI review MERGE (reports/review-mc-v2-r3.md); code review FIX (3 MEDIUM: silent lost click inside the card guard after a re-layout, gate sleeps instead of signals, a check that passes when ctl fails; docs); one fix round running, then the PM merges with merge-gate.sh.
- Build lanes: tiling-v2 (local sonnet, wt/tiling-v2, steps 1-7), mc-v2 (local sonnet, wt/mc-v2, 11 steps), construct-v2 on the tour worker session_016iS3kNcWCmF1YP1YaxYmP6 (already set up; tour-v2 done d5d5126); xp-v2 waits for disk: one more target would drop free space under 8 GB (11-11 18:45: 12 GB free after removing the wobble3d target and target-base); it starts when mc-v2 merges and its target goes. tiling-v2 MERGED as fe3483f (3 UI rounds, code review fixed in dd5d064..b4c6ab8, gate 17/17). Merge train 1 of 3 for 11-12. A chat lane (chora-assistant chat tab per look, sent bubble, Stop state) is needed next: the white "Chat ..." tab breaks the tiling bar. Each merges only after an independent pixel review at 1x and 2x.
- Parked until the looks meet their RC requirements (user 11-11 "stay focused"): flakes, CI dry run, logging inventory.
- screencast worker session_016iS3kNcWCmF1YP1YaxYmP6: tour of every look, then reshoot of construct from 06044cf into media/2026-11-12/tour-v2; interim friction list received.

Done 11-11 (late): swarm 2 (9 runs) triaged: 20 verified, 9 rejected (env), `reports/swarm-2/findings.md`; PM card and compact hook (friction 33, 34).

Next:
1. PM decides the taste questions of each design pass (logged in design/decisions-taste.md), then build lanes per look (one worker per area), builder never reviews its own pixels.
2. Cheap lanes: flakes F10/F11, T3 gate-construct-windows, CI draft and release dry run (local only), logging inventory.
3. Status page refresh after each event.

After the reset (Sat 10:59, Fable): fixes of the swarm findings by area (helper chat entry in every look, key help in xp, tiling, mc, Super chords leaking into terminals, cube error state, mc first screen), review per area, three clean gates, RC about 11-19/20.

## Plan of 11-12 (run 05:58 Bangkok; PM, user away)

Budget about 93 % of the week by usage.py; the reset is Sat 10:59. Light work only until then:
1. mc-v2 MERGED as 59df16e (train 2 of 3 on 11-12).
2. construct-v2 on the tour worker continues; review and merge when it reports.
3. Disk: 7.9 GB free; remove the tiling-v2 target (merged) and the mc-v2 target after its merge; no new builds below 8 GB.
4. review_due.py says DUE (range a03be22..HEAD, 64 critical files): every merge had its own review; the weekly round over the whole range runs as the per-area code review on Fable after the reset (decisions-now.md).
4b. Screencast of the new looks: after construct-v2 merges, the tour worker records tour-v3 (tiling v2, mc v2, construct v2, xp as far as merged) and the status page gets it (user 11-12).
4c. 11-12 ~01:50 UTC (user asked for parallel lanes): xp-v2 remaining steps (local sonnet, wt/xp-v2) started; the core fontconfig fix was dropped: system fonts already load, Pagella is just not installed here (packaging dependency); construct worker nudged for its gate report; 7 finished worker sessions archived; watchdog trigger trig_01VxAcAFayNMtGiWDVuzJuRz every 2 h at :30.
4f. construct-v2 (09223c9): 14 of 15 steps done by the tour worker; step 9 (a different manager per face) needs a Host refactor: faces stay XP and get honest labels "XP 1" to "XP 4" now, the refactor is a Fable lane after the reset. Code review running; UI review when the captures are pushed; then the worker records tour-v3 of the new looks. Flaky core tests under load (words::a_stack_overflow_leaves_its_words, last-words SIGSEGV handler) to investigate.
4e. Swarm 5 (new looks) triaged: 5 verified (tiling Apps ignores clicks, suspect fuzzel 1.9.2; mc no mouse way back; no user-facing light-look switch in tiling; tiling key list without mouse basics; mc AGENDA heading looks like a link), 13 env. Launcher fixes before swarm 6: start tests/fake-voice.py, a notify-send stub or mako, a fake loginctl that runs swaylock. Fix lane for the 5 next (cloud worker if the env check passes).
4d. 11-12 ~02:20 UTC: packaging lane claude/pkg reviewed (python3 for CI fixed in 80f31a7; Pagella visibility already handled by 09-chora-tex-gyre.conf); it merges with construct in train 3. Watchdog now hourly at :35 (30 min is under the scheduler minimum). Swarm 5 on the new looks (tiling v2, mc v2) starts when the swarm binaries at 59df16e are built (user: swarms on new looks).
5. xp-v2: 8 of 15 build steps pushed (5e61a6f: F1 help, Start menu, keys, caption buttons, tooltips and pager, icon grid and balloon, Control Panel hover, Stop greyed, balloon fade). Open: steps 2, 4, 10, 13 and the helper tray icon, the 14 px chrome sweep, taskbar 36 px, Help as a window; then UI review and code review. Continues on Fable after the reset. After the reset (Fable): the core fontconfig fix, packaging of the new flavor files, the per-area review, the leftovers in backlog.md.

## Foundation gate

Charter section 1 item 20, status at the end of 11-09. Not met.

| Criterion | Status | Evidence |
|---|---|---|
| 3D steps 0 to 4 merged with their exit criteria | 2 of 5 | step 0 (45d85f2) and step 1 (c90b2be) merged; steps 2 to 4 (about 1,500 lines in `design/3d.md` section 4) not started |
| Handover guard green | green, with gaps | `tests/gate-handover-matrix.sh`: 69 of 85 pass, 12 gaps, 4 skips (counted in its EXPECTED table); the gate is green because it expects the gaps. Gaps: the key path in tiling, xp and mc (9 cells), the mc drag (3 cells). Open steps: give key, mc drag (`design/window-handover.md` section 6) |
| No open finding of high or medium severity (review, feedback, baseline defects) | not met | the review round on df6dc69..HEAD is due and not run (grants step 1, XP start feedback, 674a7a9 and the design principles were not reviewed as a range); issue #102 (activation token without a press) is open and H4c would widen it; backlog items 3 to 6 (watchdog, silent core death, flaky speech test, 16 bugs) are open and items 4 and 5 have no issue |
| XP split X2 to X5 merged | 1 of 4 | X2 merged (e5391bd); X3 to X5 not started |
| Due split reviews done | done | xp, voice, mc and construct (see "Split reviews"); the splits themselves are not done |

Order of work until the gate holds: stability and bug fixes, 3D, plugin
separation. From 11-10 at least one foundation lane always runs and
the sprint has at most two lanes (reflection of the evening, point 1).

## Plan of the day

2026-11-10 to 11-13 (user request 11-09: finish 3D this week, so that
week 1 has room for playful 3D demos and feature sprints):

- Main lane, 3D core, in order: step 2 (Tue), step 3 (Wed and Thu, the
  hard one: perspective, plane-split, ray hit; opus implements), step 4
  (Thu), step 5 (Fri); step 6 beside them on Tue. Each step: tests
  first, 2D guard, review (opus security view on input and hit test),
  merge through the merge-gate script.
- First task 11-10: the merge-gate script (no pipes, log per step, rc
  checks, FAIL grep), then the H4c design check (opus).
- Sprint lane (one at a time): H2, then H4a and H5.
- Cheap lane (haiku/sonnet): Dependabot bumps, inline comment cleanup,
  issues for the stability items, shared CARGO_TARGET_DIR.
- Not this week: construct on 3D (K5), virtual outputs (D69), the cube
  geometry; they start week 1 as the base of the 3D demos.

## Next step

Fable (11-10, user: no mindless burning, heavy lifting only): Fable
takes only the hard steps where a stronger model changes the outcome:
construct on 3D (K5), virtual outputs (D69), the cube geometry, give
with grab (E3), plus the judge audits. Everything else (XP split X3 to
X5, grants steps, the voice policy crate, fixes, docs) stays on sonnet
with opus reviews. One Fable lane at a time; its brief names the exit
criterion so that it stops when done.

Order until the foundation holds (charter section 1 item 20):
stability and bug fixes, 3D, plugin separation; no feature sprints.

0. Disk (user request 11-09): after the security agent ends, `cargo
   clean`, add `[profile.dev] debug = "line-tables-only"` to
   `~/.cargo/config.toml` (incremental is already off there), rebuild;
   make `tests/2d-guard.sh` build the base into the shared target
   instead of `/home/user/target-base`, so the dependencies are shared;
   update charter section 3 "Disk" with the measured sizes.
1. Review round on 02cdeed..HEAD (running), answer its findings.
2. The two failing construct gates (backlog track A item 9).
3. Probe: a child session per run (reflection point 1).
4. Then 3D step 0 (plans/3d.md).
5. Remove inline comments that today's commits added (2d-guard.sh,
   tests/persona/, the security commits), per the user rule of 11-09.

## Done

- 11-10 11:05: pushed 1cdb2f6: H6 context switching with review fixes,
  3D step 5 with review fixes (order cache, 512 MiB texture cap, idle
  frame 0.5 ms). 3D core steps 0 to 5 done three days before plan.

- 11-10 10:10: pushed c5f277f (merge-gate green each time): 3D step 4
  with DnD security fixes (f5d293b chain), mc clock duration bug
  445cdfa, H5 HUD host with review fixes (too-long cards refused).

- 11-10 09:18: pushed 2d72eec through merge-gate (all green): Dependabot
  11 of 13 plus rmcp 3.9.0 advisory fix (#96 zbus 6 and #87
  serde-saphyr 2 wait: API migrations); 3D step 3 with review fixes
  (split caps 64 planes and 4 pieces per plane, context hit for nested
  and tilted contexts, projective pieces, clamp only plane points); H2
  HUD pictures and fonts with review fixes; H4a chat model with review
  fixes (epoch in agent.read).

- 11-10 07:50: merged and pushed 6ad7f67 through merge-gate.sh (all 7
  steps green): 3D step 2 with its review fixes, H4c act-stamped tokens
  with its review fixes (fixes #102), and the tiling focus race fix
  1275e15 (backlog 17; 4/4 failing runs under load before, 0/10 after).

- 11-09 evening: claude/hud-h1 with its review fixes merged as 1e824a9
  and pushed. Gate: 965 tests, clippy, 2D guard, xp.sh, gate-hud every
  cell as expected; the handover matrix printed one FAIL (no focus with
  xp before a cell, the focus flake of backlog item 17), yet the chained
  gate went on and pushed. A rerun of the matrix alone on the same
  commit: every cell as expected, rc 0. Why the chain did not stop is
  not found (friction 18).

- 11-09 evening: claude/hud-h3 with the review fixes (74e8348: the
  cancel deadlock, entries cut to 16 KiB and the ring to 4 MiB,
  redaction, a chat unread for 10 minutes expires, a `notice` role)
  merged as 7df49e3. Gate: 931 tests, clippy, 2D guard, xp.sh, matrix
  every cell as expected, gate-hud every cell as expected. Known gap:
  the line redaction checks only the first separator of a line.

- 11-09 17:28: HUD gate table merged (5e92531, branch claude/hud-h0):
  `tests/gate-hud.sh` measures 20 cells of the acceptance checks A1 to
  A15 that a script can verify headlessly. All 20 are `missing`, as the
  table EXPECTED says, because no HUD code exists yet; a forced wrong
  line fails the gate, and `--self-test` shows it without a session.
  Fixtures in `tests/hud/mine` and `tests/hud/fancy`.

- 11-09 15:17: ACP permission gate merged (103ce6f, branch
  claude/acp-gate; backlog item 0). A permission request of an ACP agent
  that runs or writes now takes only a key of `confirm_keys` or a HUD
  button; a spoken yes no longer answers it and an always word answers
  none (6fafc3f). Opus security review of the branch: no high finding;
  medium 1 and 2 (the screen showed too little of a request, and the
  keys were not named) and low 3 to 6 were fixed in follow-up commits:
  16a07ce (the whole title and raw input shown, secrets hidden, the
  keys named, a spoken yes sounds the error), 089dc54 and 42826c0
  (docs), and 84bd5eb (a request whose detail is too long to show whole
  is rejected, and the user hears why). Tests added with the fixes:
  `a_permission_request_needs_a_key_unless_it_only_looks`,
  `a_physical_sub_agent_question_takes_no_spoken_yes`,
  `an_always_word_does_not_answer_a_sub_agent_question`,
  `a_permission_request_is_physical_unless_it_only_looks`,
  `a_command_keeps_its_whole_title_for_the_user`,
  `a_physical_sub_agent_question_names_its_keys`,
  `a_request_too_long_to_show_is_rejected`. That they failed before the
  fixes is what the agent reports said; the reports are not kept in this
  log, so the red-first claim is not checked here. Gate before the push
  (PM report): 918 tests, 2D guard, xp.sh, handover matrix with every
  cell as expected. This entry was missing until the evening reflection
  found it (point 5); it was written afterwards from the commits.

- 11-09: construct adoption fixes squash-merged as 674a7a9 (gate:
  912 tests, 2D guard, xp.sh, construct windows and keys, matrix 158 s);
  the remote branch `claude/construct-adopt` could not be deleted (proxy).

- 11-09 15:00: XP start feedback merged (92e7539): a missing program
  says so at once; a slow start shows "Starting NAME" after 2 s and
  "Cannot open" after 30 s without a window; launchers that exit 0 are
  followed like any program. Two review regressions fixed before the
  merge. Gate: 909 tests, 2D guard, xp.sh, matrix gate green.

- 11-09 14:30: grants step 1 merged (52abe8f): confined callers cannot
  claim the indicator, answer, confirm, talk or mute; injected pointer
  presses are marked and ignored by the HUD answer buttons; docs name
  the one-hop routes of methods.privileged. Opus security review done.
  Gate: 895 tests, 2D guard, xp.sh, matrix gate green.

- 11-09 afternoon: merged persona driver (ece3c80), 3D step 1 (c90b2be),
  review fixes with guard hardening (62e7d94), XP split X2 (e5391bd),
  xp.sh cleanup and the handover matrix gate (df6dc69: 46 of 84 cells
  pass, every cell as expected). Gate on df6dc69: 891 tests, 2D guard,
  xp.sh, matrix gate green.

- 11-09: construct gates fixed and merged (aa17985): launcher window
  under a forking shell, two test script fixes; 3D step 0 merged
  (45d85f2): RenderStats and 2D path tests, review clean; 859 tests.

- 11-09: XP split X1 merged (4a13267): docs/xp-split.md, skeletons of
  chora-notify and chora-assistant, chora-draw as a shared library, D77
  (four repositories, amends D36); review: no high, two medium fixed;
  853 tests, 2D guard PASS. Panes as views of sources, D76 (bf81b35).

- 11-09: security fixes, all seven findings (fcf70e5 to ee61765 after
  the rewrite), 851 tests; history of the session branch cleaned of
  session files (ee61765); roadmap without plan links (4295193); guards
  `no-inline-comments`, `no-session-files`, `commit-message-length`
  (b9cd834, 4a845c2, 82ae39f); target shrunk from 20.9 to 3.4 GB; 2D
  guard PASS against 8742dee after all of today's code.

- 11-09: security check, attacker view (opus). Items (a), (b), (c) of
  charter section 1 item 11 HOLD: (a) `confined: true` in
  `crates/choractl/src/mcp.rs:108-111,352-355`, test
  `ipc::tests::a_confined_main_socket_connection_needs_the_grants_of_a_client_socket`,
  live probe refused `run_program`, `tiling.settings.set`, `focus_next`
  and `load_view` without grants; (b) every tool call passes
  `confirmed()` (`services/voice/src/chat.rs:990-995`), default rule for
  `run_program` (`config.rs:362-375`); (c) `Config::spawn` puts
  `sandbox_cmd` first (`config.rs:421-426`), `check_harnesses`
  (`config.rs:1056-1061`), tests
  `harnesses_run_in_the_sandbox_unless_unsandboxed`, `check_rejects_mistakes`.
- 11-09: 2D guard `tests/2d-guard.sh` (0153f0d), red test against a
  changed premultiply in `render.rs`.
- 11-09: 3D strategy `plans/3d.md`, revised after an opus review
  (9cde738); decision D75 and roadmap (dd59a2e).
- 11-09: baseline (07c6c09): 846 tests, most gates pass; `cargo deny`
  fixed (8589485, yanked async-executor).
- 11-09: driver for simulated users (4e372b4), audience research and
  everyday task matrix (b4b7520).

## Waiting for merge

(none)

## Running (11-10 17:00 Bangkok)

pm: session_016iS3kQmq6Q6Ce3hxtenYSR (persistent)

Workers, 11-11 16:30: all fix lanes of persona round 3 merged.
- session_016iS3kMuC2WXeztpuWDKsXe (opus): UX/UI loop 1, designer
  review of the whole feature set (KISS/POLS), report ux-loop-1.md.
Fable workers idle until Monday: core 016iS3kL1yG8R47DGySmB65H,
virtual 016iS3kNteKAJNer4zC9m3qk, HUD 016iS3kNBZGoJk1MQrWZ4pBV, demos
016iS3kN2vNDneETCgRK3tan.
Merged 11-11 late: ux-construct (67f93a9), F9 gate fix (d00a626), xp one balloon + polish (5acea88).
Merged 11-11 pm: HUD looks, demo scene, stale-frame, wm-pane, fix-chat
(cfcc239, 5bd349b), fix-cube (157f991), fix-basics (536aaee).

Local: K5 construct on 3D (agent afab03f7e183f96f0, /home/user/wt/k5),
plus construct wobble in 3D and construct-3d.png / construct-wobble.png.

Merge queue: demo (claude/demo) after review; floating windows in work (claude/virtual-2).

Merged today: merge-gate script, H4c, 3D steps 2 to 6, focus race, HUD
H1 to H7 and H4a/H4b, effects driver, give key, grab fix, flake fixes,
render fixes, D69 step 1, grants2 (2a1e2d2), hud-start (c81144a),
wobble3d E2 (3dc1eb6), virtual-2 steps 2-3 (7ad15de), core-death + mc-scale (e21a506), K5 (55383af), E3 give during a drag + flakes F5/F6 (108f1b2), D69 steps 4-8 (9e78360), xp cube in construct (67d51c0), pull-out (e2cff6b), K4 cube modes (a4e5459), CI agent sandbox (2f6c3e0), W12 + sandbox packaging (12b176d), demo capture clock + scenes (dd7a5d3), floating windows + cube items (0fa6c16), click rule (b8ac1ae), winit gate (a6b8444), handover matrix 85/85 + mc drag (f6c09e5). sccache on. Cap: four heavy lanes.

## Blocked

## Split reviews

split review: chora-xp 18258 2026-11-09 (planned: D73, roadmap track X1 to X7)
split review: voice 16933 2026-11-09 (one policy crate chora-voice-policy, the rest modules; design/split-review-voice.md)
split review: chora-mc 6704 2026-11-09 (no split; share the voice wire types in chora-kit; design/split-review-mc.md)
split review: chora-construct 6530 2026-11-09 (no plugin split; split main.rs into files after the adoption merge; D76 geometry as a chora-kit module at K9; the /proc launcher check to chora-kit; design/split-review-construct.md)

## Review findings

Round of 11-09 evening (sonnet bugs, KISS, leftovers, UX; opus security
done, below) on A = df6dc69..5e92531, B = hud-h1, C = hud-h3:

- C high (verified by the PM): `agents.rs` `cancel_running` holds the
  lock of the match scrutinee while `cancel()` locks again; cancelling a
  running helper deadlocks the voice main loop. Fix: read the state into
  a bool, drop the guard, then cancel; test with a running helper. Also
  lower-case the name as `cancel()` does.
- C medium: an open chat of a hung client mutes results forever; low:
  a caller-less `chat.open` gets the unknown-tool text.
- B medium: a `near` cycle parses; `row()` leaves helper states
  (idle, done, blocked) in English and ends in " · " for an empty task;
  `HudNotice` says "1 errors", shows a shell command and the internal
  name "chora-voice". Low: template fields per source undocumented in
  docs/hud.md; `when` values and size bounds not validated; KISS: a
  `match` instead of TABLE plus order test; duplicate text helpers in
  ask.rs; the jargon test misses "chora-voice" and "Look".
- Opus security view: B1 = the same deadlock (high; it blocks the stop
  path, principle 5). B2 medium: transcript entries uncut (16 MiB lines,
  OOM; an entry over the core MAX_LINE sticks the reader): cut entries to
  16 KiB and bound the ring in bytes. B3 low: transcript not redacted
  (`chat::hide` unused). C1 medium: SVG `<image href>` read by the
  default usvg resolver: resolver limited to `data:` or `hud::file`,
  `resources_dir` = folder (goes to H2). C2 medium: folder fonts could
  fake the card text: the card draws with host fonts only (H5). C3
  medium: ids of any characters, literal text anywhere, unbounded sizes,
  `chat.slot` lets a client place the card: ids `[a-z0-9_-]{1,32}`,
  upper bounds, the card always on top at a host-fixed place (H1 fix,
  H5). C4 low: `hud.yaml` read through `dir.join` and parsed after a
  failed scan, no entry count or depth limit, hard links, alias budget.
  C5 low: `{` in the file part of `picture`.
  A1 medium: any unconfined main-socket process (run_program with a
  token, an ACP agent shell if the sandbox mounts the socket) can claim
  the indicator and press Yes: only the HUD names of a security setting
  may claim; test the socket check of agent-sandbox.sh (grants step 2).
  A2 low: an injected press sets `last_press` and `pressed`, so it can
  mint a token and satisfy the focus press rule (into the H4c design
  check). A3 low: a missing `confined` field means unconfined: make it
  `Option<bool>`, `None` = confined.
- A medium: the XP start messages (launch.rs:62-73) are English only and
  name no next step; they move into the word table after H1 merges.
  Low: `program_name("'my app' x")` gives "my"; German voice phrases
  use the informal address against principle 7 (existing convention,
  backlog); gate-hud.sh header cites design files that are not in the
  repo.


Status marks added on 11-09 evening from the code at HEAD (5e92531) and
the commit messages; "fixed" means a commit named it, not that a new
review confirmed it. The review round on the later commits is due.

Security check of 11-09 (probes in the scratchpad, `sec/`):

1. medium: spoken questions hide what runs: `run_program` speaks only
   the file name (`config.rs:1283-1286`), `inject_input` asks "Shall I
   type or click?", privileged calls do not name the command. Fix: speak
   the whole argv when it has more than one word or a `/`, the typed
   text, and the keys and values of `settings.set`.
   Status: FIXED fcf70e5.
2. medium (if voice-mcp has `methods.privileged`): the privileged list
   is a hand-kept copy (`config.rs:395-410`); `methods.list` does not
   send the privileged flag (`methods.rs:262-267`).
   Status: FIXED 9967c60, 59c4bd2, 1788a06.
3. low-medium: injected keys reach grabs without an "injected" mark, so
   a client with `input.inject` can answer a confirm question meant for
   a physical key.
   Status: FIXED d443fe7.
4. low: a confined main-socket connection skips `check_paths`,
   `confine` and the claims check (`ipc/clients.rs:262,289`,
   `ipc.rs:1287`); shader compile errors leak file content.
   Status: FIXED e823dd9.
5. low: the event connection of `choractl mcp` is not confined
   (`crates/choractl/src/events.rs:70-74`).
   Status: FIXED 53803ce.
6. low: `load_view` has no confirm rule.
   Status: FIXED c56e26d.
7. docs: `services/voice/README.md:343-350` is out of date.
   Status: FIXED ee61765.

Review round of 11-09 on 02cdeed..d443fe7 (five views; bugs: clean,
187 voice tests and the new core test pass; security: nothing high or
medium):

8. medium (docs): `services/voice/README.md:346` still says the agent
   does not ask before privileged methods (same as security finding 7,
   open in the security agent).
   Status: FIXED ee61765 (README now says the agent asks).
9. low: `tests/persona/session.sh` runs `rm -rf "$dir"` on exit also when
   `$dir` is a caller-supplied `PERSONA_DIR`; delete only what it made.
   Status: FIXED 2eb2606.
10. low: `tests/2d-guard.sh` hard-codes `/home/user/wt/base2d`,
   `/home/user/target-base`, `/home/user/guard-cache`; make them
   overridable; the base cache ignores changes of the scenes.
   Status: OPEN (the paths are still hard-coded in
   `tests/2d-guard.sh`).
11. low: the privileged question is no longer a user `confirm` rule, so a
   user cannot switch it off per method; document it in `docs/voice.md`
   (or keep a rule).
   Status: FIXED 1788a06 (docs/voice.md names the flag and the
   question).
12. low: `said` and `spoken` in `services/voice/src/config.rs` duplicate
   each other; `privileged_rule` builds a throwaway `Confirm`;
   `client.rs` test doc names "finding 3 of the security review" (a
   private reference); `input.rs` `KeyAction::Grab` doc merges two
   sentences; `tools.rs` doc of `privileged_rule` contradicts own
   settings methods; `smoke.sh` has dead `cp` lines; `act.py type` text
   starting with `-` needs `--`; an all-move `inject_input` is spoken
   "Shall I type or click: ?"; `docs/voice.md:271-277` is one run-on
   paragraph.
   Status: OPEN, in part (`said` and `spoken` still both exist in
   `config.rs`; the other points were not rechecked).
13. low (security): `privileged_rule` fails open for a method not yet in
   the set within a turn; the spoken params are never cut (a long
   question).
   Status: PARTLY FIXED. Spoken values are cut at 60 characters
   (`spoken` in `config.rs`). The fail-open of `privileged_rule` for a
   method not yet in the set was not rechecked: OPEN.

Not verifiable here: `agent-sandbox.sh` is not in the repo (a writable
`.git` in the sandbox would let an agent plant git hooks).

## Checked and rejected
