# review co-reach r5 (rc 1, 2; co-79 MAJOR): seeded review of claude/co-reach-r5 before its merge

Model: opus, effort high, because co-79 was reopened once and the diff picks what a person stood back from a pane is told in a look (POLS, co-68: a told key must work); the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/co-reach-r5 head 7d9269ae, base 62a04be0 (7 files, +189 -9). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it: EXIT 0 on 7d9269ae; the train runs it again).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-co-reach-r5 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/co-reach-r5.md (done criteria), reports/co-reach.md "## r5", findings/construct.md co-79, co-68 on claude/session-log; `git diff 62a04be0..7d9269ae`.
Checks, each with its command and files:
1) Red first: 58ba73a1 (test) before 5025dc47 (fix); run `cargo test -p chora-construct reach` at 58ba73a1 (red) and 7d9269ae (green); read the gate step 7 in tests/gate-construct-cube.sh: does it fail on 62a04be0 for the pane path and assert the window lands in a face with the keyboard in it?
2) Told key works (co-68): hints.rs and pull.rs share one predicate (put_window) for hint and key; no state where "Super Alt 2 Into face" is told but the key does nothing or acts on another window (no pane, two panes, cube turned to face 1, pane closed while stood back).
3) Keys: input.rs routes the new key only in construct's stood-back state; no key reaches a client the person cannot see; the pull key stays untold for a pane.
4) Lane's own open point: a drag-drop onto a face while stood back; does the keyboard follow the window or stay stood back? Read the code path; a finding if it breaks POLS, with severity.
5) docs/cube-modes.md: one sentence names the rule and matches the code; no new sleep, lint allow or other plugin (`git diff --stat`); tools/sleep-ratchet, size-ratchet, lint-ratchet on 7d9269ae.
6) Captures media/2026-11-25/co-reach-r5/ on claude/session-log: the bar text readable, the window in the front face after the key.
Known, not findings: co-44, co-66, co-77 (MINOR backlog), co-75, co-76 (hold), the mc-sofa volume-wheel flake.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rcr5-N, severity, file:line, text, fix in one sentence). Append to reports/co-reach.md under "## Review r5".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
