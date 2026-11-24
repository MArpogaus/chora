# tour-base r4 (F7, rc 11): a check the candidate stops measuring is not `same` (rtb3-1 MINOR)

Model: sonnet, effort medium (one verdict branch in tests/tour-diff.py, mirror of the rtb2-2 fix; routine)
Lane: cloud, branch claude/tour-base-r4 at d40142e4 (session head, t127b, contains tour-base r3). One area: tests/tour-diff.py, tests/test_tour*.py; no product code, no intended.toml rows. Stop at 150 k. Cost at most 0.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `python3 -m pytest tests/test_tour.py -q`, `python3 -I tests/test_tour_diff.py`. A failure: send_message the PM "tour-base-r4 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/tour-base.md "## Review r3" (rtb3-1 with its fix line, line 380).
1) Red commit first (one commit, tests only, message cites rtb3-1 only): `verdict("a", [], True, [], ({"I6": "green"}, {"I6": "not measured"}))` is not `same` (a regression), also for a red base; a `check` row naming I6 absorbs it. Paste the failing lines.
2) Fix commit: where the candidate is "not measured" and the base is not, the check joins `flipped` (absorbed only by a `check` row), as rtb2-2 does for the other side. KISS: no new verdict, no new field.
3) `python3 demo/tour/run.py --check` if it runs headless here: paste the summary and the tour-diff counts per verdict (unchanged counts expected; a change: name the acts).
4) Full gate: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no --only), detached: `setsid nohup bash -c '(tests/merge-gate.sh --jobs 2 > /tmp/gate.log 2>&1; echo EXIT $? >> /tmp/gate.log)' >/dev/null 2>&1 </dev/null &`, poll in the same turn; paste the last 20 lines. A red cell you did not cause: name it with its log line. Before ending: `pgrep -fa 'chora --headless'` empty.
Never end a turn to wait; poll any long command in the same turn.
Done when: red then fix shas with pasted lines; --check counts or the reason it did not run; full gate green or its foreign reds named.
Report: reports/tour-base.md "## r4" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/tour-base-r4 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
