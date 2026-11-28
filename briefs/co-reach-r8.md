# co-reach r8 (rc 2 construct: review r7 FIX, rcrr7-1 MAJOR and the wait's MINORs of the same diff)

Model: opus, effort high because the fix changes a timed wait in construct's mode state (who runs the face manager, a second press, a fly elsewhere, a non-blocking answer): a state-machine design, not routine
Red on head: rcrr7-1 on 008714ba: xp already running, Super+Alt+2 from a pane: open_cube refuses "a client xp runs already (code 1)", nothing is given (review probe probe_xp_runs_give_from_pane); dc532d60 gave it to xp.
Lane: cloud, claude/co-reach-r8 at 008714ba (co-reach r7 head, review FIX). Cost at most 1 %; stop at 250 k context. One area: plugins/chora-construct (overlay, input, pull, hints, reach_tests) and tests/gate-construct-cube.sh, docs/cube-modes.md; no other plugin, no core, not demo/tour/.
Why: design/rc.md row 2 construct (co-79); the review r7 (reports/co-reach.md "## Review r7") found the key now refuses where it gave before. These MINORs are in the same unmerged diff, so they go with the MAJOR (rules.md 6 does not hold back a fix of the change under review). Owner 11-17 10:59 "Be perfectionist and very critical about the met targets."
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`. A failure: send_message the PM "co-reach-r8 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/co-reach.md "## r7" and "## Review r7", findings/construct.md co-79, docs/cube-modes.md "One key, one step".
Each id: a red test commit first (unit test or gate cell failing on 008714ba), then the fix commit.
1. rcrr7-1 MAJOR: with the face manager already running (manager pane or client), Super+Alt+2 from a pane gives to it as on dc532d60; the bar tells what the key does there; the cube_window branch only when no face manager runs.
2. rcrr7-2 MINOR: the wait ends when the person moves on (flown into another pane, the noted window no longer the cube's surface).
3. rcrr7-3 MINOR: a second Super+Alt+2 while the cube starts keeps the same put.
4. rcrr7-4 MINOR: every unput end after a told key tells a notice (20 s: "XP did not place the window in time", or the reason).
5. rcrr7-5 MINOR: no blocking state.get in the poll: send, match the answer on a later poll (or back off after a timeout); the frame loop never waits on xp.
6. rcrr7-6 MINOR: cube-gate cells: Super+Alt+2 in a pane with xp unable to start and with xp running, each asserting a told reason or the give.
7. rcrr7-7 NIT: docs/cube-modes.md one sentence on how the wait ends and what is told.
After: the 8 paths of the r7 table (report section 4) each 5/5 again by hand with the told key, plus a 9th: xp running, key from a pane.
Run: the full gate, no step filter: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no --push) detached with setsid nohup, poll it in the same turn; paste the last 20 lines. A red cell you did not cause (sh-37..sh-47, doors helper-none): name it with its log line, rerun it alone and the steps after it. Never end a turn to wait. Before ending: `pgrep -fa 'chora --headless'` empty.
Done when: seven ids each with red then fix commit (the NIT without red); 9 paths 5/5; gate EXIT 0 in one run (or a named foreign flake and every later step run); reports/co-reach.md "## r8" on claude/session-log (base, head, diffstat, per id, path table, gate line); captures under media/2026-11-28/co-reach-r8/ (each under 500 KB).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): head sha, ids closed, counts, gate result.
Rules: push claude/co-reach-r8 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report and media folder); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
