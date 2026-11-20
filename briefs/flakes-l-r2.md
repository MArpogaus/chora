# flakes-l r2 (F5: no flaky tests): the six gate flakes of trains 82-87 and flake-series r2

Model: sonnet, effort high (routine flake fixes: reproduce under load, then bounded waits on state)
Lane: cloud, branch claude/flakes-l from the session head adf71956 (train 86). One round, stop at 250 k. Cost at most 1.2 %.
Step 0 (no setup script): `git fetch --unshallow || true; cargo check --workspace; command -v foot python3 sccache`; a failure: send_message the PM "flakes-l r2 BLOCKED: <line>" and stop (no apt). A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: backlog.md lines FLK-LAY1, FLK-HUD1, FLK-HUD2, the tiling-look ink-2560x1600@2 FLAKE line; reports/flake-series.md "## r2". Known, not yours: launcher-click red with fuzzel 1.9.2 (the PM runs 1.11.1).
Order (by hits; each: count reds of 20 under load before, cause, fix, count after):
1) tiling-look ink-2560x1600@2 "the focus border starts in 0 0 0 255, not the accent" (4 hits, passes alone).
2) acts xp act:session tests/acts/looks/xp.py:167 ValueError on an empty mixer read (2 hits).
3) gate-hud: FLK-HUD1 context-picks-folder timeout on figure_left; FLK-HUD2 voice-state-rate timeout on hud-state Listening.
4) layout FLK-LAY1: construct HUD light-1280x720, Super+T no third terminal.
5) tiling-audit fullscreen paper-2560x1600@2: the bar does not hide within 20 s (1 hit).
6) acts selftest persona sound: I5 overlap on the mc header (1 hit).
Load: `--jobs 3` beside `cargo build --workspace` in a second target dir (4 cores). A product fault (wrong first frame, lost notice) is a finding: red test commit first, then the fix. Tests: bounded waits on state (until_ok of the gate lib), no retry of a whole cell, no longer timeouts as the fix, no skips. A flake not reproduced in 20: say so, add the state wait only where the read is racy by code, name the line.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,gate-lib,tiling-look,acts,hud,layout,tiling-audit` green, then the touched cells twice more under load. Merge origin/claude/autonomous-dev-session-3w-i33pg7 before the final gate. Poll in the same turn; never end a turn to wait.
Done when: per flake red count before and after (0 of 20 after) or a named finding with red test and fix; gate green; reports/flakes-l.md "## r2" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/flakes-l and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
