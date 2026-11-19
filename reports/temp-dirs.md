# temp-dirs (F23 test helpers, F24 taskbar.rs clash)

## r1

Branch claude/temp-dirs, head 5b0aa407 (red test commit 9c61e13f). Gate
`--only pre-commit,ci-check,build,test,clippy`: green.

Helper: `chora_kit::testing::TempDir` (crates/chora-kit/src/testing.rs):
`new(name)` makes `name-<pid>-<n>`, `unmade(name)` only takes the name,
`keep()` gives up the guard; removed on drop. Copies (same code, comment
names the kit helper) in crates/chora-scene/src/temp_dir.rs,
crates/choractl/src/temp_dir.rs, crates/chora/src/testing/temp_dir.rs; the
integration tests log_file.rs and chora_report.rs include them with
`#[path]`. tools/copies.toml row "temp folder by pid outside the counted
helper" (max 8, only falls).

Test (red first): `testing::temp::same_name_twice_gives_two_folders_removed_on_drop`
in chora-kit. The red commit has a stub with the old name+pid naming so the
pre-commit clippy compiles; the test failed on `assert_ne!`.

### Sites of `grep process::id()`, moved (about 100)
- chora-kit: lib.rs, settings.rs, hud.rs, icons.rs (3), volume.rs
- chora-draw: assets.rs (2), text.rs, theme.rs (4)
- chora-scene: paths.rs, logs.rs. choractl: view.rs, mcp.rs, main.rs (2),
  tests/chora_report.rs
- chora: backend/host_keymap.rs, transaction.rs (bad.png), render/budget.rs (2),
  safety.rs, tests/log_file.rs (2)
- plugins: notify, assistant (island, card, main x4, folders), construct
  (tests, file, state, cube x2, lifecycle, config), mc (scene, main, config x2),
  tiling (overlay, state, main x2, config, lifecycle, theme x3), xp (wm panels,
  cascade, control, startmenu x2, config x4, taskbar.rs:938 = F24, wm x3,
  desktop_icons, launch)
- services/voice: audio, tools, coder, record x2, client, main, agents x7, chat,
  config (cwd, key), pcm, tests/log_file.rs

### Left, with the reason
- Production code, not test folders: kit lib.rs:111 (atomic write), launch.rs
  188 (exit file, counted), 862/1020/1269 (real pid), tiling theme.rs:483,
  assistant check.rs:125 (private_dir), construct cube/start.rs:120 (log file,
  counted), choractl main.rs:578 (unit name), voice config.rs:1307 (probe socket
  in the runtime dir, removed by its own Drop).
- Already counted (name, pid, counter), a file or socket path that outlives the
  helper: kit testing.rs FakeCore, conn.rs serve, core testing.rs `scratch_dir`
  (25 callers pass a PathBuf), headless/guard.rs, testing/wayland.rs,
  ipc/clients/tests.rs shm, xp sheets.rs theme().
- PNG helpers that hand a path string on (name per test, no counter): core
  transaction.rs:604, render/sampling.rs (2), ipc.rs:1689. Next: return a guard
  with the path.

### Notes
- Helpers that give a long-lived object (Construct, Tiling, Wm, theme cache) its
  folder use `.keep()`, as before: the folder stays, the name is counted.
- Helpers that returned only a path now return `(TempDir, PathBuf)`; callers bind
  `_dir` (a bare `_` drops at once; the first gate run failed on that, 8 kit tests
  and 30 voice tests, fixed).

## PM review r1 (22:10 UTC; PM not the builder)
Base 219d5afe, head 5b0aa407, 71 files +555/-280. Red commit 9c61e13f: a stub with pid-only naming, so the test fails on assert_ne. Red first: holds.
- tdr-1 MAJOR (F2): TempDir is in four places (the kit plus 74-line copies in chora-scene, choractl and core testing/temp_dir.rs), with a copies.toml max 8. The kit, choractl and chora all depend on chora-scene, so the helper can live once in chora-scene (a `testing` module behind a feature, or a plain pub module) and be re-exported by the kit (`chora_kit::testing::TempDir` stays). F2 met at 0 copies (audit r2); this would add 3. The brief's "copies where the kit cannot be a dependency" was the PM's mistake: the lowest common crate exists.
- tdr-2 MINOR: `merge-tree` with kc-fix f666a025 (train 75) conflicts (on_fake_core). r2 merges claude/kc-fix and resolves it.
- tdr-3 NIT: the copies.toml row then becomes allow = [chora-scene] with max 0 outside it.
- Checked, no finding: `_dir` binding (no bare `_`), 13 `.keep()` sites for long-lived objects (as before), the "left with a reason" list.
Verdict (computed): FIX (tdr-1 open MAJOR). r2: briefs/temp-dirs-r2.md.

## r2 (head 86f42d1f)
- kc-fix (f666a025) and autonomous-dev-session-3w-i33pg7 merged; on_fake_core conflict kept both (TempDir guards + `core.runtime(NAME)`); the autonomous merge brought one new `Runtime::with` (cube/tests_construct.rs), now `core.runtime`.
- tdr-1: TempDir lives once, `chora_scene::testing` (feature `testing`; kit feature enables it, kit/choractl/chora dev-depend on it); kit re-exports `chora_kit::testing::TempDir`. Its test moved with it. Three copies deleted; `grep -rn "pub struct TempDir" crates plugins services` gives one line.
- tdr-3: the copies.toml row allows only the chora-scene file. Deviation: max is 8, not 0: eight counted PNG/socket helpers outside it (guard, transaction, sampling x2, ipc, wayland, testing.rs, xp sheets) still match the pattern; max 0 fails cut-check. They may only fall.
- Gate (pre-commit,ci-check,build,test,clippy): green.

## PM review r2 (22:31)
Checks: `grep -rn "pub struct TempDir" crates plugins services` one line (chora-scene/src/testing.rs:20); `grep -rn 'Runtime::with(' plugins` empty (the train 75 conflict fixed here too); kc-fix f666a025 merged; red 9c61e13f first.
- tdr-1 closed. tdr-2 closed. tdr-3: max 8, not 0, with the reason (eight PNG/socket sites); NIT, next round temp-rest r1.
Verdict (computed): no open BLOCK or MAJOR -> MERGE-OK 86f42d1f, train 77. It holds kit-connect r1 and kc-fix, so kit-connect r2 was stopped (redundant).
