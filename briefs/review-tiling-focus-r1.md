# review tiling-focus r1 (rc 1, 2; ti-57): seeded review of claude/tiling-focus-r1 before its merge

Model: opus, effort high, because the diff amends a focus rule across looks (D119) in the kit wm core where a wrong rule sends keys into a client the person cannot see; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/tiling-focus-r1 head 6261d3e0, base 79ed970f (27 files, +729 -47). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it whole: EXIT 0, 66 PASS; the train runs it again).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-tiling-focus-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/tiling-focus-r1.md (done criteria), reports/tiling-focus.md "## r1" and reports/p2-r11-tiling.md "## r11" on claude/session-log; `git diff 79ed970f..6261d3e0`.
Checks, each with its command and files:
1) Red first: 69feb0e0 holds the failing test and comes before 990d57cf; run that test at 69feb0e0 (red) and at 6261d3e0 (green).
2) Rule: D119 (docs/decisions.md) and docs/api.md "Window managers" say the same thing as the code (crates/chora-kit/src/wm/): a menu give (--stay) leaves the keyboard with the giver (next window, else desktop root); the receiver yields. Look for a path where no look holds the keyboard, or one where keys reach a client on an output the person does not look at.
3) The narrowing: a give without stay (Super+Alt+N, drag, automation, voice, MCP) still moves the keyboard. Judge against POLS and the shared RC bar ("no stray keys into clients"): MAJOR if a person-driven key give sends keys to another output unseen; else a note. Same for xp as giver (does not keep it yet).
4) Other looks unchanged: mc, construct, xp alone; gate-handover-matrix cells; tests/gate-looks-move.sh polls, no new sleep; tools/sleep-ratchet, size-ratchet, lint-ratchet on 6261d3e0.
5) Captures media/2026-11-25/tiling-focus-r1/: holder before and after, key list opens and closes after the move.
Known, not findings: the core flake words.rs:594 a_backtrace_into_a_full_pipe (untouched), FLAKE-CUBE r1, FLAKE-SOFA.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtf1-N, severity, file:line, text, fix in one sentence). Append to reports/tiling-focus.md under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
