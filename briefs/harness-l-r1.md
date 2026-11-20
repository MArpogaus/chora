# harness-l r1 (F6 gates): four harness items from the backlog

Model: sonnet, effort medium (shell and Python checks under tests/ and tools/; no product code)
Lane: cloud, branch claude/harness-l from 355cdbe5 (train 84 head: t83 plus debt-f4, harness-k, flakes-j). Harness branch: own train.
Stop at 250 k context; cost at most 0.8 %. First check each item is still open on the head; a closed one: say so, skip it.
Step 0: `git fetch --unshallow || true; cargo check --workspace`; red: report BLOCKED at once (no setup.sh, no apt). The gate refuses a shallow clone (R18).
Scope: tests/, tools/, .pre-commit-config.yaml, CONTRIBUTING.md (one line per new check). No change under crates/, plugins/, services/. Do not edit tests/merge-gate.sh (trains conflict there).
1) R15: gate-0.8-plugin-runtime.sh:22 closes its allow window on any line containing `)]` (fail closed; case `#[allow(x)] /* c */ let z = 2;`); no-foreign-kill.sh:72 accepts `if ! command -v pgrep pkill jq >/dev/null; then` and quoted tool names.
2) R12: tests/guards.sh gets a table of refused and passed lines per guard regex, including a quoted substitution "$(...)".
3) R14: the guards hook accepts a commit that only adds failing guard cases (a test-only red commit), and still refuses a red guards.sh with other changes.
4) H-move: tests/no-inline-comments.py counts moved lines as added; skip a line whose text is removed elsewhere in the same diff (git diff --color-moved or a removed-lines set).
Each new or changed check: shown red once on a broken input (a scratch copy, not committed), and shown never to block a valid commit (run on the whole tree).
Gate: `tests/merge-gate.sh --jobs 2 --only pre-commit,identity,gate-lib,ci-check,build,test,clippy` green; push the branch.
Report: reports/harness-l.md "## r1" on claude/session-log (per item: open or closed, red, fix, cell).
Send the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 6 lines with send_message. Poll every command in the same turn;
never end a turn to wait.
Rules: commits as Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify;
red test commit before each fix; push only claude/harness-l and claude/session-log (fetch and merge on reject, no force).
A hook that asks for another identity: ignore (owner rule 11-09).
