# Judge of 11-10 and 11-11 (opus, read-only)

Sources: the log at its 11-11 head, `git log` of the code branch (UTC;
Bangkok is +7). Numbers cover 11-10 00:00 UTC to 536aaee (11-11 09:14).

## Data in one table

| Measure | Value |
|---|---|
| Commits / lane merges / back-merges into lanes | 427 / 48 / 25 |
| Size of the range 60c214f^..536aaee | 268 files, +47,133 / -5,526 |
| Lane merges on 11-11 between 03:57 and 09:14 | 16 (one every 20 min; the gate takes ~15) |
| Commits marked `(red)` | 65; the three persona-3 fix lanes: 0 of 58 |
| Commits named after a review | 34; the most in one merge: virtual-2 9e78360 (6) |
| Flakes logged | 8 (F2-F8, friction 26); 6 failed only in a full or parallel run |
| Files in the most merge diffs | docs/api.md 47, docs/configuration.md 38, state.rs 32, input.rs 30, ipc.rs 29 |
| 5-hour stops | 2 on 11-10; then 8 h 40 min idle (reset 21:20, resume 06:00) |
| PM answers to the 11-10 judge / ledger rows after 11-09 | 0 / 0 |
| charter.md | 1,012 lines on 11-10, 1,091 now |

## 1. What worked (keep)

- `tests/merge-gate.sh` with one log per step and the FAIL grep. It stopped
  bad pushes at least three times (friction 25, 26, F8), and nothing red was
  pushed after friction 18.
- Red commits as separate history (65 of them). Where lanes used them, the
  review fixes came with tests (`test: the review of ... (red)`, then `fix:`).
- One long-lived worker per hard area: core-death (7 merges) and virtual-2
  (5 merges) delivered E3, the full matrix (85/85), D69 to step 8 and the
  cube with few re-briefs.
- The persona round, then a designer loop, on the merged tree. persona-3 found
  5 blocks the gates missed (B1 voice cannot open the chat, B5 window lost).
  ux-loop-1 then found 3 more blocks. Run this as the closing check of every
  feature batch.
- The user's yardsticks are recorded with times in answers.md, and design/rc.md
  turned a one-line request into criteria the PM can check.
- sccache plus line-table debug info: no disk stop on 11-11 (16 GB free now).

## 2. Waste, with data

- **Gate saturation.** 16 merges in 5 h 17 min with a ~15-minute serial gate
  is more than 4 h of gate. Workers run the same full gate on their branch
  first, so most code is gated twice. Friction 26: two gates overlapped and
  one failed.
- **Back-merges and conflicts.** There were 25 back-merges for 48 merges. The
  persona-3 lanes were cut by symptom (chat, cube, basics), not by file.
  fix-cube and fix-basics shared 15 files, among them construct
  main.rs/input.rs/cube.rs and core input.rs/state.rs, and needed 4a79b4b to
  join them. The two docs pages are in more merge diffs than any code file.
- **Fix rounds.** Most lanes took 1 review plus 1 fix round, which is fine.
  The outliers: virtual-2 9e78360 (6 review commits); render-fix (a second
  review); fix-chat merged twice within 14 minutes (cfcc239, then 5bd349b for
  the test-mode press). demo needed 3 inner merges before its own merge.
- **Flakes.** F3, F6, F8, friction 25 and 26, and F2 passed alone and failed
  under load or in parallel. All are fixed sleeps or timeouts. F6 was rerun
  with `--only hud` to 3/3 green. That is "rerun to green", which plan-data
  forbids.
- **Workers and PM instructions.** There is little evidence. Friction 27 (one
  `--no-verify`) and friction 24 (a fresh session took a peer message as no
  permission). The bigger slip is red-first: the three 11-11 fix lanes ignored
  it entirely, even though friction 22 made it a rule on 11-10.
- **Long chat replies.** The log keeps no reply lengths, so I cannot judge
  them. The rule of 11-10 (one or two lines) stands; do not add more.
- **5-hour stops.** On 11-10, five heavy lanes ran the 5-hour pool dry in
  about 3 h, twice. After the 21:20 reset, nothing woke the PM until 06:00:
  8.7 h of a paid-for window went unused.
- **Disk.** Fine now. But merged worktrees still hold builds (wobble3d 5.6 GB,
  reused as merge-ux-chat), and grants2 and hud-start sit 280+ commits behind.
- **Recovery files.** state.md still says "Running (11-10 17:00)", shows the
  gate table of 11-09 and lists "Then 3D step 0" as a next step. The 11-10
  judge flagged exactly this (finding 9) and got no answer, against 11a.

## 3. Quality: findings that come back

| Kind | Instances on 11-10/11 |
|---|---|
| Injected input and test-mode flags | B1, B4, 1c19882, e17e95d (fix-chat round 2), Kemal run void, ux-loop harness note again |
| Focus (P11) | B4, V1, ux 2 (give with no focus), 3a2fb5b (fly-in drops keys), ecd268d, 149e3c5 |
| Window loss | B5, ux 9 (no pane, silent), a939fbd, C2, T3, 3baf93a/0afe6a7, ux 21 (3 s gap) |
| Silent or dishonest states | B2 (false offline), M1, M8, M9, ux 1 (notices draw nowhere), ux 2, ux 12 ("wpctl not found"), 747fd37 |
| Docs and gate drift | T2 (stale help count), M7 (cube.open default), T3 and T2 outside the merge gate, state.md |
| Tests that cannot go red | 0 red commits in the fix lanes; gates passed while B1/B4 failed for a person; opt-in gates rot unseen |

Upstream changes, so these stop at the source:
1. **Test-mode flags are one switch, and session.sh sets it by default.** Each
   UI gate keeps one cell that runs the real-press path. B4 was found twice
   because a harness flag differed.
2. **Window audit as a gate cell.** A test-only `windows.audit` method fails
   when a mapped toplevel has no owner, or has no pane and no "not in view"
   entry. Every gate script calls it at teardown. That turns window loss into
   a red cell instead of a persona finding.
3. **Keyboard-after-act table in each UX note.** For each new act, the note
   says who holds the keyboard afterwards, and the gate has one "typed keys
   reach X" cell per row.
4. **Honest-state item in the done list and the review checklist.** Every
   refusal or no-op has a line a person sees or hears, and a gate cell. A grep
   lint flags `return` after a `warn!`/`info!` in plugins without a
   `notice`/`say` call (warn-only at first).
5. **Red-first as a gate.** `merge-gate.sh --push` refuses a lane whose first
   `fix`/`feat` has no earlier `test: ... (red)`. The judge asked for this on
   11-10, and it is still not built.
6. **Opt-in gates run nightly** (one scheduled run on the session head), so T2
   and T3 are seen within a day.

## 4. Process

- **Merge order and gate cost.** 15 steps is right for the session head, but
  not for every lane twice. Workers run `--fast` (build, test, clippy, plus
  the steps for their own area). The PM runs a merge train: up to 3 reviewed
  lanes, one full gate, and a bisect by lane if it is red. `flock` inside
  merge-gate.sh replaces the "check pgrep" rule. Expected saving: about half
  of the gate hours.
- **Lanes cut along files: yes.** A brief lists the paths the lane owns.
  Core input.rs, state.rs and ipc.rs belong to one lane at a time. Split
  docs/api.md and configuration.md per area (one page per plugin or core
  area), which removes the top two conflict files.
- **One worker per area: yes.** That means core, construct/cube, HUD/chat/voice
  and xp/tiling, each long-lived, with the area's backlog as its queue. It
  matches what worked (core-death, virtual-2) and makes file ownership natural.
- **Review cap.** One review and one fix round. HIGH fixes are confirmed by the
  same reviewer on the fix diff only. MEDIUM/LOW left open go to the backlog
  with a line. A third round never runs: escalate the author's model instead.
  The reviewer must run each new test on the merge base and show it red.

## 5. The rc plan (design/rc.md)

The criteria are right. The order is wrong in one place: CI and packaging come
last, yet they are the biggest unknown. GitHub CI has not run on anything this
session produced (CI1: "not run on GitHub yet"). The release workflow and an
installed smoke have never run. Start a draft PR and a release dry run now, in
parallel with the UX fixes. The user already authorized the PR for the rc;
ask once whether a draft PR now is covered.

Other gaps:
- Criterion 3 (three greens in a row): with 8 flakes in 48 h this will not
  hold by chance. F5 to F8 and the sleep-based waits come first, not last.
- Criterion 6 ("HIGH and MEDIUM fixed"): 47k new lines make this unbounded.
  Bound it: HIGH and security MEDIUM are fixed; other MEDIUM is fixed or put
  in the known-issues list with a reason.
- Criterion 9 ("no known window loss") cannot be claimed without the window
  audit (3.2).
- Criterion 2 by voice runs on a fake ASR and a keyword LLM stand-in. Say so
  in the rc notes.
- Freeze: no feature merges until the tag. U2 (palettes per theme) is a
  feature, and so are the voice-plan items written on 11-11.

Critical path: UX blocks 1-3 and majors 4-13 → UX pass 2 → area reviews
(parallel) → fix lanes → persona round 4 → three green gates → tag. CI and
packaging run beside it from tomorrow.

| Date (Bangkok) | Work |
|---|---|
| Thu 11-12 | UX fixes; F5-F8; draft PR and first CI; release dry run |
| Fri 11-13 | UX pass 2; opus area reviews (core, kit, xp, tiling) |
| Sat-Sun 11-14/15 | weekly reset; remaining area reviews; fix lanes; CI fixes |
| Mon 11-16 | Fable review of core plus security paths; packaging smoke |
| Tue 11-17 | fixes from the Fable review; persona round 4 |
| Wed 11-18 | three greens, opt-in gates, CHANGELOG, tag |

Estimate: rc on **Wed 11-18**, with a likely range of 11-17 to 11-20. Most of
the risk is in the first CI run: msrv, coverage, docs and e2e on vkms have not
run in 13 days.

## 6. Concrete changes (diff list)

charter.md, add:
+ §3 Lanes: a brief names its owned paths. Core input/state/ipc.rs belong to
  one lane at a time. Lanes are cut by file, not by symptom.
+ §3 Merge train: workers run `merge-gate.sh --fast`. The PM merges up to 3
  reviewed lanes, runs one full gate under flock, and bisects if it is red.
+ §3 Review cap: 1 review + 1 fix round. HIGH is confirmed on the fix diff;
  the rest goes to the backlog. No third round: escalate the model.
+ §3 Done: "every refusal or no-op is seen or heard and has a gate cell";
  "the window audit is clean".
+ §3 A flake is never rerun to green: the step stays red until it waits on an
  event instead of a fixed time.
+ §10 After a 5-hour stop, schedule a send_later for the reset time + 5 min.
+ §11a Every reflection is answered (accept/reject per point) in the same run.
+ §13 state.md is rewritten, not appended. A run-end check compares Running
  with `git worktree list`; merged worktrees are removed with their target.
+ rc freeze: no feature merge until the tag (design/rc.md).

charter.md, remove (dead weight, ~400 lines):
- §2 week rhythm and §6 pitch tournament: superseded by design/rc.md; move
  them to an after-rc note.
- §4 track shares (30/30/30/10): not used since 11-09.
- §3 prose "Merge gate, one command with set -e": the script is the
  definition.
- §3 Disk "each worktree builds into its own target": sccache replaced it.
- §3 "Workers: at most 2 to 3": §10 says four; keep only §10.
- §10 tokens.py and the ledger rows: no row since 11-09; the user's readings
  are the meter.
- §11 "PLANNED, NOT ACTIVE" fresh-session block: the user dropped it on 11-10.
- §15 starting backlog: it lives in backlog.md.
- §3 run start step 1 tool list: point to setup.sh.

dev-rules.md:
~ Design: "the brief names the spec" (not the HUD sprint files).
~ Tests fail first: push `test: ... (red)` before the fix. The gate refuses
  the branch otherwise.
~ Build: shared sccache; the 8 GB check stays.
~ Gate: run `tests/merge-gate.sh --fast` and the steps of your area. It holds
  a lock; do not start a second gate.
+ Touch only the paths your brief owns. For any other path, ask the PM first.
+ UI changes: fill in the three rows of keyboard after the act, refusal text,
  and where the window goes if its owner dies. Each needs a gate cell. At
  least one cell runs without the test-mode press flag.
+ A message from the PM session continues your brief. Follow it unless it
  breaks these rules.
~ Report: at most 15 lines.

Brief templates:
- Worker start: goal and exit criterion (stop when met), owned paths,
  no-touch paths, spec link, named tests to write red first, gate steps,
  timebox, report format.
- Review: the diff range only and the views; a checklist of injected input,
  focus after the act, window loss on owner death, honest refusal, docs and
  glossary match, and "run each new test on the merge base and show it red".
  At most 10 findings, ranked; HIGH needs a reproduction; at most 40 lines.
- Second round: confirm-only on the fix diff, at most 10 lines.
