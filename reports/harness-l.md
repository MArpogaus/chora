
## r1

Branch claude/harness-l, head 75dfa4b9 (from 355cdbe5). Gate `--only pre-commit,identity,gate-lib,ci-check,build,test,clippy`: green, pushed. Step 0 (unshallow, cargo check) green.

| item | open? | red | fix | cell |
|---|---|---|---|---|
| R15 allow window | open | ff5ef811: `#[allow(clippy::needless_return)] /* c */ let z = 2;` then a connect call passed (hidden) | cead612e: window closes on any line with `)]` | guards.sh runtime_case, 6 cases; whole plugins/ tree green |
| R15 negated need | open | ff5ef811: `if ! command -v pgrep pkill jq >/dev/null; then` refused | cead612e: optional `if/elif/while/until` and `!` prefix | kill_case, +4; quoted tool names already passed; `if ! command -v pgrep; then pkill chora; fi` still refused; all tests/*.sh green |
| R12 table | open | n/a (table of cases) | ce9242c3: runtime_case helper and tables for plugin-runtime and no-foreign-kill, incl. quoted substitutions "$(...)" | guards.sh |
| R14 hook | open | scratch: bare `exit 1` guard script refused by the old entry | 520dc775: tests/guards-hook.sh accepts a red guards.sh only when tests/guards.sh is the only staged file and every added line is a `*_case` call; no removals | hook_case, 4 cases; red commits ff5ef811 and 33a88995 went through it |
| H-move | open | 33a88995: comment moved between functions refused | 75dfa4b9: lines whose text is removed anywhere in the staged diff are skipped | move_case, 4 cases; a copy still fails |

Notes: the R14 change could not have its own red-first commit (the old hook refuses a red guards.sh), so test and fix are one commit. The accept rule needs single-line `*_case` additions: a new helper function goes in a green commit first (done for runtime_case and move_case). H-move limit: an edit that restores a text removed elsewhere in the diff passes (documented in the script header). The R12 tables cover the two guards of R15; mkdir-runtime, injected-optout and clippy-config-sync keep their existing cases.

## PM review r1 (06:17 UTC; PM not the builder; diff 355cdbe5..75dfa4b9, 7 files, +109 -10)
Checks: red first (ff5ef811 before cead612e; 33a88995 before 75dfa4b9): met. R15 fail closed on any `)]`, case "code after the attribute" red: met. Negated need accepted, `if ! command -v pgrep; then pkill` still red: met. R12 tables incl. quoted names: met. R14 hook accepts only `*_case` additions, 4 cases incl. another file and another line: met. H-move moved passes, copied and edited fail: met. Gate PASS per handback.
- rv-1 NIT: guards-hook accepts any `*_case` line, including one whose argument runs a command substitution; it is our own test file, so low risk.
- rv-2 NIT: no-inline-comments collects removed lines from every file in the diff, so a bare `//` removed anywhere lets a new `//` through.
- rv-3 NIT: R14 test and fix share 520dc775 (the old hook refused a red commit; documented in the handback).
Verdict: MERGE-OK (no MAJOR). It gets its own train (t86) after t85.
