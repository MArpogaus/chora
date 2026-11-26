# tour-fix r1 (F7, rc 1, 11; rtr4r-1, rtr4-3, rtr4-4 MAJOR): fix the tour so take r3 can run

Model: opus, effort high, because the construct chapter acts and the v8 replay decide which v9 differences count (F7 judgement, not routine)
Red on head: rtr4r-1 (5 whole-frame c5d rows at demo/tour/intended.toml:4377-4405 on 972f9415); rtr4-3 acts 9, 10 red; rtr4-4 launch.py:274 breaks the v8 replay
Lane: cloud, branch claude/tour-fix-r1 at 972f9415 (claude/tour-rows-r4 head, review r4 FIX). Area: demo/tour/ (intended.toml, launch.py, chapters/construct.py), tests/tour-diff.py, tests/test_tour_diff.py; no product code. Stop at 250 k. Cost at most 1.2 %.
Why: reports/tour-rows.md "## r4" and "## Review r4" on claude/session-log; findings/shared.md rtr4-3, rtr4-4, rtr4-5.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo build --workspace`. A failure: send_message the PM "tour-fix-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1. Red first, its own commit: tests/test_tour_diff.py fails on 972f9415: no pixel row box covers the whole frame (rtr4r-1), and a row citing a merge commit takes its files from `git diff --name-only <sha>^1 <sha>` (rtr4r-3, 19348b95 must be rejected).
2. rtr4r-1: drop the five c5d rows; c5d joins rtr4-3.
3. rtr4-3: the construct chapter follows the drop that now steps in (75d4623e): acts c2b c3 c3b c3c c3d c4 c4b c5d c5e say and do what v9 shows (step out where the act needs the cube); acts 9 and 10 green.
4. rtr4-4: launch.py ensure_notify must not break the v8 replay (mc dark, handover); rtr4-5: tool-version.sh from the build repo, not TOUR_REPO.
5. Re-take v9 (head) and v8 (733967aa, TOUR_BASELINE=1); counts of tests/tour-diff.py; every regression left is a filed finding (rtr4-1 m2 m2b m3, rtr4-2 m8, or new rtf1-N with both stills).
Out of scope (backlog): rtr4r-2, rtr4r-4 (row causes and the v8 agenda page).
Run: `python3 -m pytest tests/test_tour_diff.py tests/test_tour.py -q` and the full gate, no step filter: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 --allow-skip docs` (no --push) detached with setsid nohup, poll in the same turn; paste the last 20 lines. A red cell you did not cause: name it with its log line, rerun that step alone and then the steps after it. Never end a turn to wait. Before ending: `pgrep -fa 'chora --headless'` empty.
Done when: red commit before the fixes; acts 9, 10 green; v8 replay runs through mc dark and handover; re-take counts in the report; gate EXIT 0 (or a named foreign flake and every later step run); reports/tour-rows.md "## fix r1" on claude/session-log (base, head, diffstat, counts, gate line); stills under media/2026-11-26/tour-fix-r1/.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): head sha, red commit, counts, findings left (ids and severity), gate result.
Rules: push claude/tour-fix-r1 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report and media folder); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
