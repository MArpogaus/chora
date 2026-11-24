# review guard-4 r1 (F4, F5, rc 3): seeded review of claude/guard-4 before its merge

Model: sonnet, effort high (harness logic: a ratchet and a load-scaled time bound; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/guard-4 head 9aa6a556, base cae62f50 (9 files, +215 -13). One round. Stop at 120 k; cost at most 0.5 %; at most 8 findings. No full merge gate (the lane ran it green); run only the cells and selftests named below.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-guard-4 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/guard-4-r1.md (done criteria), reports/guard-4.md "## r1", findings/shared.md sh-37, reports/audit-f.md "## r6" (F4 W9); `git diff cae62f50..9aa6a556`.
Checks, each with its command and files:
1) Red first: 97928c9b before 427b33fa, 6b2dc70c before a9e8e887/9aa6a556 (`git log --oneline cae62f50..9aa6a556`); each red selftest fails at its red commit and passes at head (`bash tests/sleep-ratchet-selftest.sh`, `bash tests/gate-lib-selftest.sh`).
2) Sleep ratchet (tools/sleep-ratchet, tools/sleep-ratchet.list): the count is the audit r6 command; it only falls; `--update` cannot raise it; the fail message names list value, tree value and top files; wired in ci-check (tests/merge-gate.sh), .github/workflows/ci.yaml and tests/ratchet-raise.sh; a valid commit is never blocked. Check that the "two halves" and comment wording do not hide real sleep lines from the count (grep the tree for sleeps the count misses).
3) sh-37 bound (tests/gate-lib.sh load_scaled_limit, cpu_probe_ms; tests/gate-mc-scale.sh): does the scaled bound still catch a real slowdown of settings.set (a 2x regression of the core with an idle CPU must fail)? Is the probe cheap, deterministic enough, and bounded above (no limit that grows without cap under heavy load)? Run `bash tests/gate-mc-scale.sh` twice idle and once beside `cargo build --workspace`; report times and bounds.
4) No new sleep line, no changed timeout elsewhere, no foreign file touched.
Known, not findings: the lane flake 9316 ms did not reproduce (the red is a selftest with observed numbers).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rg4-N, severity, file:line, text, fix in one sentence). Append to reports/guard-4.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
