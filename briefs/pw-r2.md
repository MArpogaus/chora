# pw r2 (P2 prerequisites): fix review r1 rw-1, rw-2 (MAJOR), rw-3, rw-5, rw-8

Model: sonnet, effort medium (two test-harness fixes with a plan; routine)
Lane: cloud, branch claude/pw from 3ba07d9b (r1). One round, stop at 250 k. Cost about 0.7 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "pw r2 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Read on claude/session-log: reports/pw.md "## Review r1" (rw-1 to rw-8). tests/ and docs only; no product code.
1. rw-1 red first (own commit): a persona-selftest line runs shared/screenshot.sh in the session environment with the
   grim stand-in and fails when the file lands outside $PERSONA_DIR. Fix in tests/persona/session.sh: HOME-free
   XDG_PICTURES_DIR=$dir/pictures plus an xdg-user-dir stand-in that prints it; no write to the host's home.
2. rw-2: one green (sound end state) and one red (its breach) test in tests/test_acts.py PersonaContracts for each of
   X2, L1, L2, L3, M2, C2, C3; then reports/pw.md's claim holds for 18 of 18 (count them in the report).
3. rw-3 T6 counts only notices that are new against `before` (test: an old notice alone is red); rw-5 delete dead
   Run.env_file, Ctx.since; rw-8 POWER pattern includes `loginctl terminate` (test line).
rw-4, rw-6, rw-7: backlog lines in the report, no code.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,gate-lib,acts,doors` green, plus
`bash tests/persona-selftest.sh` and the round.sh smoke on 4 looks. Before the final gate merge
origin/claude/autonomous-dev-session-3w-i33pg7. Poll it in the same turn; never end a turn to wait.
Done when: 1-3 hold with the commands that show them; gate green; reports/pw.md "## r2" on claude/session-log
(each claim with its command or cell); head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/pw and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
