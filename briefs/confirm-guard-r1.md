# confirm-guard r1 (RC shared bar "asks before risky acts"; sh-26 MAJOR): the confirm card guard in mc and construct

Model: sonnet, effort high (a safety guard in two plugins through the kit; small diff, needs red tests)
Lane: cloud, branch claude/confirm-guard from 79c8d2b5 (session branch, train 81). One round, stop at 250 k. Cost about 0.8 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace`; a failure or a missing tool: send_message the PM "confirm-guard r1 BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/shared.md sh-26; vision-card.md (chora-assistant: asks before risky acts). On the branch: services/voice/src/core.rs (BUTTON_DELAY 400 ms and its tests near 2479), crates/chora-kit/src/ask.rs and helper.rs, mc agent.rs (confirm card, sanitize near 693), construct hud.rs (confirm card, sanitize near 1074).
Items:
1. A confirm card in mc and in construct ignores Yes (click, key, voice word) for 400 ms after it shows, as voice does (BUTTON_DELAY). Put the guard once in chora-kit (ask.rs or helper.rs) and use it in both; no copy (F2). Red test first (own commit): a kit unit test plus one act or gate cell per look that answers Yes at once and asserts no run; then the fix (own commit).
2. Verify the sanitize half: a prompt with a bidi override (U+202E) and a zero-width char shows escaped in both cards; a test that pins it (red only if it fails).
3. findings/shared.md sh-26 row `fixed <fix sha> (red <sha>) test:<names>` on claude/session-log.
Do not touch crates/chora input.rs, mc navigation or construct key code (other lanes). Poll each run in the same turn; never end a turn to wait.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,gate-lib,doors,construct-hud,mc-paths,mc-sofa,chat-keys`; all PASS. A red cell you did not cause: rerun it alone once and name it.
Done when: red commits before fixes, gate cells pass, reports/confirm-guard.md "## r1" on claude/session-log (per item: red sha, fix sha, test, gate cell); head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/confirm-guard and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
