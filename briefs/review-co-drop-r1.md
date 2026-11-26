# review co-drop r1 (rc 1, 2; rcr5-2, rcr5-1): seeded review of claude/co-drop-r1 before its merge

Model: opus, effort high, because the diff changes a focus rule in the 3D look (D55, D119) where a wrong step sends keys into a window on a face the person cannot see; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/co-drop-r1 head 75d4623e, base 7d9269ae (6 files, +365 -18). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it whole: run 2 EXIT 0, 66 PASS; the train runs it again).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-co-drop-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/co-drop-r1.md (done criteria), reports/co-drop.md "## r1" and reports/co-reach.md "## Review r5" (rcr5-1, rcr5-2) on claude/session-log; `git diff 7d9269ae..75d4623e`.
Checks, each with its command and files:
1) Red first: 730cc738 holds the failing tests and comes before 75d4623e; run `cargo test -p chora-construct drop_tests` at 730cc738 (red) and at 75d4623e (green); read the pull gate cell 2a (super, plain, side).
2) Rule: a drop onto the front face steps in as the person and activates the window (like put_in_face); a drop onto another face keeps the keyboard with the construct; a failed place is told and stays stood back (pull.rs given_from_a_pane, names::not_placed). docs/cube-modes.md says the same. Look for a path where no look holds the keyboard, or keys reach a window on a face or output the person does not see.
3) D119 (docs/decisions.md as on the session head df3d1ccf) and the construct give menu (list.rs, rtf1-2 class): does this diff agree with the rule TILING-FOCUS r2 makes (a person give keeps the keyboard with the giver)? A note, not a finding, unless this diff contradicts it.
4) No other look changed; tests/gate-construct-pull.sh polls, no new sleep; tools/sleep-ratchet, size-ratchet, lint-ratchet on 75d4623e.
5) Captures media/2026-11-25/co-drop-r1/: before and after match the report.
Known, not findings: FLAKE-CUBE r1, FLAKE-SOFA, FLAKE-PERSONA-NOTEPAD r1 (persona-selftest.sh:198 no click on Notepad), core flake words.rs:594.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rcdr1-N, severity, file:line, text, fix in one sentence). Append to reports/co-drop.md under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
