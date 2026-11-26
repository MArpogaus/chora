# review h-rf2 r1 (rc 3, F6; harness): seeded review of claude/h-rf2-r1 before its merge

Model: sonnet, effort high (shell harness review against a fixed rule; the builder never reviews itself, rules.md 7)
Red on head: none (a review; the lane's red 36a41daa precedes its fix a2b080ab)
Lane: cloud, read-only on code: claude/h-rf2-r1 at a2b080ab, review 4d8bbe36..a2b080ab (4 files, +309 -3: tests/red-first.sh, tests/red-first-selftest.sh, tests/merge-gate.sh, CONTRIBUTING.md). One round. Stop at 120 k context; cost at most 0.4 %; at most 8 findings. No merge gate (the lane ran it: EXIT 0).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-h-rf2-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/h-rf2-r1.md (rule and done criteria) and reports/h-rf2.md "## r1" on claude/session-log; `git diff 4d8bbe36..a2b080ab`.
Checks, each with its command and files:
1) Red first: `tests/red-first-selftest.sh` fails at 36a41daa and passes at a2b080ab.
2) Rule: a fix whose paired #[test] passes on the parent is rejected (selftest case, and one hand-made case in a temp repo, not committed); a gate-cell test prints "gate cell, not run" and does not fail; `No test: <reason>` opts out; the cap at 10 fix commits prints its line.
3) False reds: run `tests/red-first.sh` with its range option on 3be27e32..4d8bbe36 (no gate run); every commit it flags is named with its test; a valid commit flagged red is MAJOR (a gate that blocks a valid commit, routines.md).
4) Gaps: the rule misses 689bff45 (gate cell): is that reported honestly in the report and CONTRIBUTING; worktree and target cleanup after the run (no leftover worktree, `git worktree list`); time on the range under 5 min.
5) Shell quality: set -euo pipefail, quoting, shellcheck if installed.
Known, not findings: rtr4-*, sh-37..sh-45.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rhrf2-N, severity, file:line, text, fix in one sentence). Append to reports/h-rf2.md under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): verdict, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
