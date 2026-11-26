# review tiling-focus r2 (rc 2; rtf1-1, rtf1-2 MAJOR): seeded review of claude/tiling-focus-r2 before its merge

Model: opus, effort high, because the diff changes the shared focus rule in the kit (Windows::hand_kept, Focus::takes_given, D119) and in two looks, where a wrong step sends keys into a window on an output the person does not see; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/tiling-focus-r2 head 712a01f5, base 6261d3e0 (r1 head; 15 files, +472 -87). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it whole: EXIT 0, 66 PASS; the train runs it again).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-tiling-focus-r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/tiling-focus-r2.md (done criteria), reports/tiling-focus.md "## Review r1" (rtf1-1..5) and "## r2" on claude/session-log; `git diff 6261d3e0..712a01f5`.
Checks, each with its command and files:
1) Red first: 08f3b514 holds the failing tests and comes before 7407db98; run `cargo test -p chora-kit -p chora-tiling -p chora-xp` at 08f3b514 (the named tests red) and at 712a01f5 (green).
2) rtf1-1, rtf1-2 fixed: the give keys of tiling and xp's Control Panel rows keep the keyboard with the giver (next window, else its root node); the receiver yields through `Focus::takes_given`. Look for a path where no look holds the keyboard, or keys reach a window on an output the person does not see (give keys, menu rows, a refused give: rtf1-4).
3) Automation, voice and MCP gives (no stay) of the focused window still go with the window; rtf1-5 as D119 now says. D119 and docs/api.md "Window managers" match the code.
4) Default `hand_kept` for mc and construct: is the filed gap (giver does not keep) safe for the RC (no stray keys), or a MAJOR? Name the matrix cells that show it.
5) The new matrix result `kept` (tests/gate-handover-virtual, matrix EXPECTED): does it assert the holder, not a sleep; tools/sleep-ratchet, size-ratchet, lint-ratchet on 712a01f5; cut-check.
6) Merge with the session head 6daaaab8 (t147: co-drop r1 changed construct focus): `git merge-tree --write-tree 6daaaab8 712a01f5` clean? A conflict or a clash of rules is a finding.
7) Captures media/2026-11-25/tiling-focus-r2/: before and after match the report.
Known, not findings: FLAKE-CUBE r1, FLAKE-SOFA, FLAKE-PERSONA-NOTEPAD r1, core flake words.rs:594; construct list.rs give menu (filed).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtf2-N, severity, file:line, text, fix in one sentence). Append to reports/tiling-focus.md under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
