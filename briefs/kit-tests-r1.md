# kit-tests-r1 (F5: every moved module has unit tests through kit testing)

Model: haiku, effort high (tests only, no product change; the PM reviews)
Lane: cloud, branch claude/kit-tests from 79c8d2b5 (session branch, train 81). One round, stop at 250 k. Cost about 0.6 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "kit-tests BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on the branch: crates/chora-kit/src/testing.rs (FakeCore), crates/chora-kit/tests/wm_book.rs and fake_core.rs (the style to copy), crates/chora-kit/Cargo.toml.
Items, in this order (tests only; a test that finds a bug: commit it red with #[ignore = "kit-tests: <bug>"] and name it in the report, do not fix product code):
1. Harness: `cargo test -p chora-kit --test perform` does not compile (chora_kit::testing is feature-gated). Add the missing `[[test]] required-features = ["testing"]` entries for every test file under crates/chora-kit/tests that uses chora_kit::testing; check `cargo test -p chora-kit --all-features` and `cargo test -p chora-kit` both pass.
2. Unit tests through FakeCore for the modules with none: src/wm/give.rs, src/wm/kept.rs, src/conn.rs (beyond its one test), src/wm/state.rs (edges: empty, one window, removal of the focused one). At least the public functions' happy path and one edge each.
3. Coverage: if `cargo llvm-cov` exists, report chora-kit line coverage before and after; if not, say so (no install).
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7 first, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,gate-lib`; poll it in the same turn; never end a turn to wait. A red cell you did not cause: rerun it alone once and name it.
Done when: item 1 compiles, new tests pass 3 runs in a row, gate cells pass, reports/kit-tests.md "## r1" on claude/session-log (per item: sha, test names, gate cell).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/kit-tests and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
