# kitcov

Coverage of chora-kit (`cargo llvm-cov`, lines), lane kitcov, brief F5.

## r1

Target: chora-kit at least 80 % lines, unit tests only. Branch
`claude/kitcov` at `ea468e3`, on the session head `dd2a0aa`.

Result: every chora-kit file is at or above 80 % lines (none was under
80 % after the round). Total lines 90.75 % -> 94.55 %. No production code
changed; the commit only adds test code.

### Step 0

- `cargo check -p chora-kit --features chora-kit/testing`: passes.
- `cargo llvm-cov --version`: cargo-llvm-cov 0.9.1.

### Coverage, before and after

Command: `env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-kit --features chora-kit/testing --summary-only`.
The before table was measured before any change. Files marked "was under
80 %" were below the target and got tests.

| file | lines before | lines after |
|---|---:|---:|
| `actions.rs` | 89.74% | 89.74% |
| `actions/perform.rs` | 91.94% | 91.94% |
| `ask.rs` | 97.54% | 97.54% |
| `chat.rs` | 96.18% | 96.18% |
| `clock.rs` | 84.88% | 84.88% |
| `conn.rs` | 63.89% (was under 80 %) | 97.50% |
| `cube.rs` | 88.28% | 88.28% |
| `desktop_entry.rs` | 89.10% | 89.10% |
| `effects.rs` | 82.63% | 82.63% |
| `float.rs` | 98.15% | 98.15% |
| `geometry.rs` | 99.47% | 99.47% |
| `give.rs` | 88.81% | 88.81% |
| `glass.rs` | 97.06% | 97.06% |
| `helper.rs` | 77.36% (was under 80 %) | 100.00% |
| `hud.rs` | 97.10% | 97.10% |
| `icons.rs` | 76.00% (was under 80 %) | 98.60% |
| `lamp.rs` | 100.00% | 100.00% |
| `launch.rs` | 90.79% | 90.79% |
| `launcher.rs` | 76.15% (was under 80 %) | 98.80% |
| `lib.rs` | 93.85% | 93.85% |
| `modes.rs` | 100.00% | 100.00% |
| `notify.rs` | 92.66% | 92.66% |
| `runtime.rs` | 68.65% (was under 80 %) | 93.14% |
| `settings.rs` | 92.61% | 92.61% |
| `shared.rs` | 92.19% | 92.19% |
| `testing.rs` | 83.97% | 90.08% |
| `testing/script.rs` | 95.21% | 95.21% |
| `volume.rs` | 92.34% | 92.34% |
| `wm/book.rs` | 98.13% | 98.13% |
| `wm/focus.rs` | 98.63% | 98.63% |
| `wm/give.rs` | 100.00% | 100.00% |
| `wm/kept.rs` | 97.30% | 97.30% |
| `wm/state.rs` | 100.00% | 100.00% |
| `wobbly.rs` | 99.37% | 99.37% |
| `words.rs` | 98.52% | 98.52% |
| `xdg.rs` | 81.67% | 81.67% |
| **TOTAL** | **90.75%** | **94.55%** |

### Tests added (34)

- `conn.rs` (8, new `tests` module): `software` follows the `hello` answer,
  a confined caller gets no activation token, `answer` sends the result or
  the error, a rejected transaction is handed back by `try_apply` and only
  logged by `apply`, `query` returns nodes, `connect` reports `NoSocket`,
  `next_event` keeps the order and reports the close, `next_event_timeout`
  gives a queued event or none.
- `runtime.rs` (17): `from_person` and `hears_pointer` judge each event
  with an `injected` flag (they were only scanned as source text before);
  the plugin trait defaults; `settings.*` and `state.get` answer
  `METHOD_NOT_FOUND` without settings; the `state.get` export and
  `privileged`; a long error cut before a split character; a method call is
  answered and a refused answer only logs; `serve` hands on events until the
  close; `--output` filters other outputs' events; `outputs`/`all_outputs`;
  `announce` and `subscribe` call order; `grab` replaces its action and a
  refused grab holds nothing; a new set of grabs keeps the give keys and ends
  the others; `end_grabs` rejects an unparseable binding; no give keys means
  no grab; `run` comes back after a failed session.
- `icons.rs` (5): a theme gives the wanted size, else the largest PNG up to
  64 px; a candidate ranks SVG above PNG, skips symbolic icons and large
  PNGs; an absolute path names its own file and bad names name nothing; an
  answer is kept and the fallbacks use it; the oldest sender copy is removed
  when more than 16 are held.
- `launcher.rs` (3): `ask` pipes the rows and returns the trimmed line, and a
  fuzzel that cannot start answers nothing; `choose` runs a chosen row and
  gives up after the rounds; `run` with a closed list starts nothing.
- `helper.rs` (1): `hear` follows `voice.state`/`voice.remove` and forgets the
  helpers when `voice` disconnects; other clients and events do not count.

Results: `cargo test -p chora-kit --features chora-kit/testing`: 344 passed,
0 failed, 0 ignored. `cargo clippy -p chora-kit --all-targets --features
chora-kit/testing -- -D warnings`: clean.

### Deviations from the brief

- The `conn.rs` event tests use a small test-only server (`pushing_core`,
  `pub(crate)` in `conn::tests`) instead of `FakeCore`. `FakeCore` answers
  requests but never sends events, and `next_event` needs an event. The
  `runtime.rs` `serve` test reuses that helper.
- `connect_needs_the_socket_in_the_environment` returns early when
  `CHORA_SOCKET` is set in the test run. Edition 2024 makes `set_var` unsafe
  and the crate denies `unsafe_code`, so the test cannot set the variable.

### Findings

None. No test found a bug, so no `#[ignore]` was added.

### Left uncovered on purpose

- `Runtime::start` and `Runtime::connect`: they read `CHORA_SOCKET`; covered
  only through `Conn::connect_at` and `Runtime::with`.
- The `run` give-up path: it needs 30 s of failing sessions.
- The double-failure log in `apply` (`the old settings failed to apply
  again`): needs a plugin whose old settings fail too.

### Hooks

The pre-commit `injected-optout` check flagged `{ .. }` patterns on events
with an `injected` field in `runtime.rs`; those patterns name the field
(`injected: true`) now. The commitizen hook needed a conventional subject:
`test(chora-kit): ...`.

### Gate

`tests/merge-gate.sh --only build,ci-check,pre-commit` on `claude/kitcov`:
pre-commit PASS, build PASS (150 s), ci-check PASS (75 s). The gate's
pre-commit step needs `origin/claude/autonomous-dev-session-3w-i33pg7` in the
clone, which was fetched for the run. Full gate not run; the train runs it.
