
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

## r1 PM review (2026-11-19 20:57 UTC)
Diff 6aa0bd37..582ab506, 18 files, +115 -41; builder sonnet lane, reviewer PM. Identity ok (author and committer); test commit de7c8410 before the move.
- OK F2: one chora_kit::runtime::report; the three plugin copies are gone (3 -> 0); tools/copies.toml row "own report()" max 0, a planted copy fails (cut-check selftest).
- OK: printed lines unchanged (chora-xp: ERR; chora-mc: PAGE: MSG; chora-construct: ERR), pinned by stderr_of tests.
- NIT: after the move the report_line tests call the kit function, not the plugin's call site, so they pin the format, not that a failed page reaches it. Fine for a refactor guard.
- Open, not in scope: step 4 (testing::connect, 14 allows in 12 files) to the backlog.
Verdict (computed): no BLOCK or MAJOR -> MERGE-OK; 582ab506 to train 73.
