
## r1 construct

- Branch claude/log-down-co at b974134e (one commit on 7a6c0eb8; origin/claude/autonomous-dev-session-3w-i33pg7 already merged, no change).
- Kit: crates/chora-kit/src/log.rs `init()` (stderr subscriber, RUST_LOG or info, once via Once, a host subscriber stays); called at the top of `runtime::run`, so each plugin gets it. Test: a_second_init_does_not_panic.
- construct: 42 eprintln in 18 files to 0 (`git grep -c eprintln -- plugins/chora-construct` empty). Levels: error 1 (the window manager of the cube failed), info 3 (cube manager text, watchdog took the shaders, layers stay out), warn 38. Text kept. The calls go through plugins/chora-construct/src/log.rs (macros log_warn!/log_info!/log_error! calling a function with target chora_construct), because tracing macros inlined raised clippy cognitive_complexity over 10 in 13 functions and an allow would raise the allow ratchet.
- Ratchet: eprintln 131 to 91 (tools/lint-ratchet.list); lint-ratchet-selftest ok.
- Log line: the kit subscriber line is in the gate test.log: `INFO chora_kit::log::tests: after the log started` (stderr, same writer the plugin has). A construct line was not triggered in any gate cell (all healthy, no warn); not shown for a construct line itself.
- Gate: cargo test -p chora-kit -p chora-construct ok; clippy --workspace --all-targets -D warnings ok; tests/merge-gate.sh --jobs 1 (full): all steps passed, 0 red.

## PM review r1 (09:28 UTC, 7a6c0eb8..b974134e, 27 files +119 -41, one commit)
- Checks: construct eprintln 0 at b974134e (git grep); main.rs:636 calls runtime::run, which calls chora_kit::log::init first, so construct gets the subscriber; init is Once and try_init, so a host subscriber stays; levels sane (1 error, 3 info, 38 warn); ratchet only falls (131 to 91).
- lc-1 LOW: messages keep the "chora-construct: " prefix while the target already names the crate; doubled in the journal. Drop at the next touch (backlog).
- lc-2 LOW (evidence): no gate cell shows a construct line on stderr; only the kit test line. Code path is direct (one function, one tracing call), so not a MAJOR; LOG-DOWN mc adds a test that captures one plugin line.
- lc-3 process: the lane used a bare `git stash` (unmarked; the stash stack is shared across worktrees). Lanes use a WIP commit or `git stash push -m <tag>` and apply by sha. Friction entry.
- Verdict (computed): 0 BLOCK, 0 MAJOR: MERGE-OK; joins t109.
