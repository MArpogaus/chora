# kit-connect

## r1

Lane: cloud, sonnet, branch claude/kit-connect from 103844da (kit-report 582ab506 + lint-reason 1f99a98f). Brief briefs/kit-connect-r1.md.
Head: **f7569c24** (3 commits: 66e5923d the helper and its test, 0e285439 the switch, f7569c24 one allow kept).

| measure | before (103844da) | after (f7569c24) |
|---|---|---|
| `#[allow]`/`#[expect]` attributes (crates, plugins, services; `#[` and `#![`, per attribute) | 413 | 401 |
| allows "the test connects to its fake core" in plugins | 13 | 0 |
| inline `Runtime::with(` in plugins | 13 | 0 |
| `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy` | n/a | all 5 passed |

- Step 1: the helper is `chora_kit::testing::FakeCore::runtime::<A>(&self, name)`: it connects to the fake core and returns a `Runtime<A>` (`Runtime::with(self.connect(name), name, Vec::new())`). Kit code is under the root clippy.toml, so the helper itself needs no allow; `FakeCore::connect` keeps its one existing allow.
- 13 plugin sites in 11 files switched (notify, assistant, construct x5 in tests.rs, state.rs, cube/tests_moved.rs x2, lifecycle.rs; mc x3; tiling x1; xp x2), plus 4 callers of `tiling_on` and 2 of `wm_on`. `tiling_on` and `wm_on` take a `Runtime` now instead of a `Conn`. Each site's pin of `pin_accepts_injected(false)` stays where it was. No test changed in what it asserts.
- Three of the 13 allows did nothing: `on_core` (construct tests_moved), the `mc` fn of mc and the mc test of the background order call only `FakeCore::connect` or other test fns, which the plugin clippy.toml does not ban; they went too.
- Net: 13 attributes removed, 1 added back (assistant `on_fake_core`, reason now "the test answers the requests of its fake core"), so 413 -> 401 (-12). Reason for the one kept: the gate grep tests/gate-0.8-plugin-runtime.sh exempts the string "output.list" of that test only inside an item with a `disallowed_methods` allow (same as the 5 `output.list` answers of construct and xp); the first gate run was red on it (`plugins/chora-assistant/src/main.rs:958`), the fix is f7569c24. None added outside the kit.
- Step 2: kit test `a_runtime_of_the_fake_core_reaches_the_core` (crates/chora-kit/tests/fake_core.rs, with `--features testing`): the runtime says `hello`, its `subscribe` reaches the core (`["hello", "events.subscribe"]` in the recorder) and it holds no grab. Committed with the helper (66e5923d).
- Step 3: tools/copies.toml row "own test connect (Runtime::with in a plugin)", `paths = ["plugins"]`, `pattern = 'Runtime::with\('`, max 0 (greppable: every inline connect is that one call). A planted `Runtime::with(c, "a", Vec::new())` in plugins/chora-xp gives `FAIL: cut-check: copies: own test connect (Runtime::with in a plugin): 1 left, the row allows 0`; tests/cut-check-selftest.sh ok.
- Hook note: a commit-message-length failure on the first commit (body line over 72), redone; no --no-verify.
