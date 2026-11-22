
## r1 construct

- Branch claude/log-down-co at b974134e (one commit on 7a6c0eb8; origin/claude/autonomous-dev-session-3w-i33pg7 already merged, no change).
- Kit: crates/chora-kit/src/log.rs `init()` (stderr subscriber, RUST_LOG or info, once via Once, a host subscriber stays); called at the top of `runtime::run`, so each plugin gets it. Test: a_second_init_does_not_panic.
- construct: 42 eprintln in 18 files to 0 (`git grep -c eprintln -- plugins/chora-construct` empty). Levels: error 1 (the window manager of the cube failed), info 3 (cube manager text, watchdog took the shaders, layers stay out), warn 38. Text kept. The calls go through plugins/chora-construct/src/log.rs (macros log_warn!/log_info!/log_error! calling a function with target chora_construct), because tracing macros inlined raised clippy cognitive_complexity over 10 in 13 functions and an allow would raise the allow ratchet.
- Ratchet: eprintln 131 to 91 (tools/lint-ratchet.list); lint-ratchet-selftest ok.
- Log line: the kit subscriber line is in the gate test.log: `INFO chora_kit::log::tests: after the log started` (stderr, same writer the plugin has). A construct line was not triggered in any gate cell (all healthy, no warn); not shown for a construct line itself.
- Gate: cargo test -p chora-kit -p chora-construct ok; clippy --workspace --all-targets -D warnings ok; tests/merge-gate.sh --jobs 1 (full): all steps passed, 0 red.
