
## r1

Lane: cloud, sonnet, branch claude/kit-report from train 71 head 6aa0bd37. Brief briefs/kit-report-r1.md.
Head: **582ab506** (2 commits: de7c8410 pin tests, 582ab506 the move).

| measure | before (6aa0bd37) | after (582ab506) |
|---|---|---|
| `fn report` in plugins xp, mc, construct | 3 | 0 (one `report(name, err)` in `crates/chora-kit/src/runtime.rs`) |
| `tools/cut-check` copies lines | 0 (row missing) | 0 (new row "own report()", max 0; a planted `fn report()` in a plugin gives `FAIL: copies: own report(): 1 left, the row allows 0`) |
| `#[allow]`/`#[expect]` lines (crates, plugins, services) | 374 | 374 |
| `tests/cut-check-selftest.sh` | ok | ok |
| `tests/merge-gate.sh --jobs 3` | n/a | all steps passed (clippy, test, matrix, e2e included) |

- Step 1: kit `testing::{in_child, stderr_of}` run a test in a child of the test binary and return its stderr; one `report_line` test per look pins the exact line (`chora-xp: no such window`, `chora-mc: Overview: no such window`, `chora-construct: no such window`). Committed first, passed before the move, pass after it.
- Step 2: mc passes its `what: message` as `format_args!`; the printed texts are unchanged. CONTRIBUTING.md line of `runtime` names `report`.
- Step 4 (`testing::connect`) not done: the 14 "connects to its fake core" allows sit in 12 files of 6 parts, not small.
- Other `fn report` (crates/chora render x3, services/voice, helper.rs method) are different things and outside the row (`paths = ["plugins"]`).
