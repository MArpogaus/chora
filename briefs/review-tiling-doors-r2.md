# review tiling-doors r2 (rc 2: ti-55, ti-56 MAJOR, L2 false green): seeded review of claude/tiling-doors-r2 before its merge

Model: opus, effort high, because the diff changes a look's door (bar word, launch client name in the core connection) and the persona checker that decides rc 2; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/tiling-doors-r2 head fa676de8, base 721be592 (12 files, +228 -59). One round.
Step 0: `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-tiling-doors-r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green, EXIT 0).
Read briefs/tiling-doors-r2.md (PM decisions, done criteria), reports/tiling-doors.md "## r2", findings/tiling.md ti-55, ti-56, reports/p2-r9-tiling.md "## r9" on claude/session-log; `git diff 721be592..fa676de8`; captures media/2026-11-25/tiling-doors-r2/.
Checks, each with its command and files:
1) Red first: b0ec5f10 (tests only) before 9d416c21, 2ae1e3ff, 3142aea3 (`git log --oneline --stat 721be592..fa676de8`). a8d2ce57 (launch name with pid) and 45914b6b: is the cell looks-word-parallel in a commit before the fix or with it? A behaviour fix with no red test before it is MAJOR.
2) ti-55 (flavors/tiling/waybar/{config.jsonc,looks.sh}, plugins/chora-tiling/src/{looks.rs,main.rs}): word "Looks" / "Looks: xp" / "Looks: xp, mc"; refresh <= 2 s; no core answer -> "Looks". POLS for a beginner: does the word lead to the window move? Bar width with three looks (the lane notes the left island reaches the clock): MAJOR only if it overlaps or truncates at 1280 wide. `cargo test -p chora-tiling`.
3) launch_conn name chora-tiling.launch-PID: any other place (core, tests, docs, chora-kit) that matches the old exact name? Does a 2 s poll leak connections or log lines in the core (watch the leak cell's assert)?
4) ti-56: HINT_TIME 8 s; once per 10 s and not while the key list shows unchanged; the test injects the clock (no sleep).
5) L2 (tests/persona/check.py, test_check.py): passes only for a changed value on disk AND the running value after reload; a digest-only change fails; "not measured (no config)" cannot turn a real run green. `python3 -I tests/persona/test_check.py`.
6) Beyond the brief: tests/gate-doors.sh, tests/test_acts.py, tests/doors/tiling.sh keep what they asserted (no weakened check, no new sleep); docs/configuration.md matches the code.
Known, not findings: rth-1..6, rtd1-*, rv9j-1..5, rtm1-1..5, Super+q unbound (decision), persona r10 not run (next lane).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtd2-N, severity,
file:line, text, fix in one sentence). Append to reports/tiling-doors.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Never end a turn to wait; poll every command in the same turn.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
