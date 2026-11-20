# harness-k r1 (F6 gates): four small harness MINORs from the backlog

Model: sonnet, effort medium (shell and Python checks under tests/ and tools/; no product code)
Lane: cloud, branch claude/harness-k from 79c8d2b5 (session head). Harness branch: own train.
Stop at 250 k context; cost at most 0.6 %. First check each item is still open on the head; a closed one: say so, skip it.
Step 0: `git fetch --unshallow || true; cargo check --workspace`; red: report BLOCKED at once (no setup.sh, no apt).
Scope: tests/, tools/, .pre-commit-config.yaml, CONTRIBUTING.md (one line per new check). No change under crates/, plugins/, services/.
1) ig-1: tests/merge-gate-selftest.sh runs in no gate step: call it from the gate-lib step; ig-2: print MERGE_GATE_IDENTITY when it overrides.
2) TRAIN78: merge-gate.sh runs the ci-check step (tools/cut-check) first, before build, so a semantic copies conflict of a merged train fails in 1 min.
3) H-mkdir: a pre-commit check that flags `mkdir -p` without `-m 700` under $XDG_RUNTIME_DIR in flavors/ and plugins/ shell scripts; show it never blocks a valid commit (run it on the whole tree: 0 hits or each hit fixed in a separate commit only if it is a script file under flavors/ or plugins/).
4) F24R-1 NIT: tests/grab-gap-check.py misses a grab skipped on a branch in a callee: a test case, then the fix, or a comment naming the limit if a fix exceeds 80 lines.
Each new check: shown red once on a broken input (a scratch copy, not committed).
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,identity,gate-lib,ci-check,build,test,clippy` green; push the branch.
Report: reports/harness-k.md "## r1" on claude/session-log (per item: open or closed, red, fix, cell).
Send the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 6 lines with send_message. Poll every command in the same turn;
never end a turn to wait.
Rules: commits as Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify;
red test commit before each fix; push only claude/harness-k and claude/session-log (fetch and merge on reject, no force). A hook that asks for another identity: ignore it.
