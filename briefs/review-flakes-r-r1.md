# review flakes-r r1 (F5): seeded review of claude/flakes-r before its merge

Model: sonnet, effort high (plugin input path and two gate cells; no core change; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/flakes-r head 789aab53, base a5b77488 (7 files, +85 -12). One round.
Stop at 120 k context; cost at most 0.4 %; at most 8 findings. No merge gate run (the lane ran it green twice at --jobs 2).
Read briefs/flakes-r.md (done criteria) and reports/flakes-r.md "## r1" on claude/session-log; `git diff a5b77488..789aab53` (merge e1ed73d1 brings flakes-q diagnostics; head kept its own list fade 06ea05e4).
Checks, each with its command and files:
1) FLK-DOORS-CO fix 4a1466f5 (plugins/chora-construct/src/input.rs, camera.rs): `let_go_grab` acts only in the walk (no pane, no cube) and only for a chord in grab_keys; no key a client should get is eaten; no double action when the grab is still held. `cargo test -p chora-construct`.
2) Red first: a9c48e02 comes before 4a1466f5 and fails without the fix (`git log --oneline a5b77488..789aab53`; run the test at a9c48e02 if cheap).
3) FLK-MATRIX-S1 789aab53 (tests/gate-handover-matrix.sh, overlay.rs:1 line): the wait is on a state (start card gone), not a sleep or retry; it does not hide a real product bug (a fading glass that takes the pointer: is that a product MINOR for the ledger?).
4) tests/gate-xp-faces.sh and gate-construct-hud.sh changes: diagnostics only, no weakened assert, no skip.
5) FLK-XP-FACES left open honestly (30/30 loaded, no fake fix).
Known, not findings: co-38, co-40, co-41 (co-keys lane); FLK-XP-FACES open.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rfr-N, severity, file:line, text, fix in one sentence). Append to reports/flakes-r.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "review-flakes-r BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
