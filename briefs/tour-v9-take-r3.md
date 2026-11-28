# tour-v9 take r3 (F7, RC 1b, 11): a fresh v9 take on the t162 head (tour-rows r5, tour-fix r1-r2 in since take r2)

Model: sonnet, effort medium (a recording run with existing tools; no code change; routine)
Lane: cloud, claude/tour-v9-take-r3 at 3098926e (session head, t162 green). Cost at most 1 %; stop at 200 k context. No product code, no tool change, no intended.toml row: this round records and reports only.
Why: RC 1b "owner sees v9" and F7: take r2 counted 88 open, 10 regressions; TOUR-FIX r1, r2 and TOUR-ROWS r5 (regr 3 left: c5d, c5e, m8) merged since. Owner 11-17 10:59 "Be perfectionist and very critical about the met targets."
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`; `command -v foot ffmpeg waybar swaybg dbus-daemon` and `python3 -c 'import PIL'`. A failure or a missing tool: send_message the PM "tour-v9-take-r3 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/tour-v9-take.md "## r2" (the form), reports/tour-rows.md "## r5".
1) `python3 demo/tour/run.py --check` (all chapters): paste the summary; a red act: name it with its row, do not fix.
2) Full take (demo/tour/README.md "## Run"), all five chapters; joined MP4 at most 19 MB (re-encode with ffmpeg crf if larger; name the crf).
3) The v8 comparison with tests/tour-diff.py: counts per chapter and verdict (same, change, regression, open), take r2 vs this take; every regression act with its check line; say for c5d, c5e, m8 whether each is still a regression.
4) Contact sheet (demo/tour/sheet.sh) of the stills.
Done when: --check summary, the MP4, counts r2 vs r3, the sheet, all pushed.
Report: reports/tour-v9-take.md "## r3" on claude/session-log; commit the MP4 and the sheet as media/tour-v9.mp4 and media/tour-v9-sheet.png on claude/tour-v9-take-r3 (on claude/session-log only your report); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the counts.
Before ending: `pgrep -fa 'chora --headless'` empty. Never end a turn to wait; poll any long command in the same turn.
Rules: push claude/tour-v9-take-r3 and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
