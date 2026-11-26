# review h-rf2 r2 (rc 3, F6; harness): seeded review of claude/h-rf2-r2 before its merge

Model: sonnet, effort high (shell harness review against a fixed rule; the builder never reviews itself, rules.md 7)
Red on head: none (a review; red c44aaa752 before fix 23dc8ec22; red 1e1d7ab7d before fix 0e4d426ab)
Lane: cloud, read-only on code: claude/h-rf2-r2 at 59ba5c21e35ae1ecefe5b421ce9e148456208287, review a2b080ab..59ba5c21e (4 files, +218 -19: tests/red-first.sh, tests/red-first-selftest.sh, tests/merge-gate.sh, CONTRIBUTING.md); r1 range 4d8bbe36..a2b080ab for context. One round. Stop at 120 k context; cost at most 0.4 %; at most 8 findings. No merge gate (the lane ran it: all PASS, docs SKIP).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-h-rf2-r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read reports/h-rf2.md "## Review r1" (rhrf2-1..6) and "## r2" on claude/session-log; `git diff a2b080ab..59ba5c21e`.
Checks, each with its command and files:
1) Red first: `tests/red-first-selftest.sh` fails at c44aaa752 (on its parent's script) and passes at 59ba5c21e; same for 1e1d7ab7d / 0e4d426ab.
2) rhrf2-1..6 closed as the report says: one selftest case each; rhrf2-1 mixed commit no longer blocked; rhrf2-4 a build error outside the added test is "not run", not red.
3) False reds: `tests/red-first.sh` range mode on 3be27e32..4d8bbe36 gives rc 0, "2 proven red, 6 not run"; each not-run line has the named reason; a valid commit flagged red is MAJOR (a gate that blocks a valid commit, routines.md).
4) Cleanup: SIGTERM during a run leaves no worktree (`git worktree list`) and no /tmp/red-first-run.*; old traps restored after return; time on the range under 5 min.
5) Shell quality: quoting, traps, `set -u` in a sourced library; shellcheck if installed; sleep ratchet unchanged (tools/sleep-ratchet).
Known, not findings: rtr4-*, sh-37..sh-45, lint-ratchet note "allow is 359, listed 360".
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rhrf2r2-N, severity, file:line, text, fix in one sentence). Append to reports/h-rf2.md under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): verdict, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
