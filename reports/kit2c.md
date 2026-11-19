# kit2c r1 (F2): tiling on kit::actions; kit6 rw-1, rw-2, rw-3

Branch claude/kit2c (from 1d1aa5fd, merged claude/kit6 ad88ebaf; the session branch had not moved). Head c29a3e9c. Commits: 854981fe (merge of kit6; conflicts in mc acts.rs and construct keys.rs resolved to the kit2b side, `launch::start` in perform.rs), f58da09f (red), 2e12231c (port), e5eea8a4 (red rw-1, rw-2), c29a3e9c (fix rw-1, rw-2, rw-3).

## Done
- Tiling loads `settings::Common<Own>`. `Own` keeps the windows, layouts and tags (and `reload`, and the internal `swallow`); Run, Close, Give, Drag, Volume and the key list are kit `System`, done by `kit::actions::perform` with `Tiling` as a `Host`. `Host` gained a hook `run` (default: start with the shared launcher); tiling overrides it with `launch::run` so that its start still does not wait for choractl.
- `show_keys` is a serde alias of `System::Keys` (read in every look, as `help`); the shipped tiling.yaml says `keys`. The method `show_keys` is unchanged.
- Bindings tiling has no action for (terminal, launcher, chat, lock, power, screenshot, desk, cube) are dropped with one warning each (`Settings::refuse`, as the construct); before they were parse errors.
- wm.rs `act` takes `&Own`; `Effect` is only Relayout and Reload. `give_focused` and the `is_finite` check of `inc_master_width` are gone (a YAML `.nan` is refused by the parser before the check: error text "non-finite float").
- rw-1: wrapper runs the command in `( )`; `exit N` and a failed `exec` write the status. rw-2: `becomes`/`gone_by_polling` poll `alive(pid)` or the unit state every second when pidfd_open fails, the poll is interrupted, or the unit is active without a pid; the wait for the unit to empty is now a poll too. rw-3: `chora-launch-exit-`; copies.toml pattern follows. D104 states the wrapper design and that rw-1 is closed.
- tools/copies.toml: row "tiling: a shared action of its own" at 0 (there was no tiling action row before). No xp `bindings` (KIT2d).

## Cells, red first
- Red f58da09f: golden `the_configs_of_29e2cd77_load_to_the_same_table` (tests/configs/29e2cd77/tiling.{yaml,table} = the shipped default, tiling-user.{yaml,table}; recorded on the old code, green before and after; the table keeps the old spelling `show_keys`), `one_binding_gives_the_same_system_action_in_every_look` (the shared file loads in tiling to the lines of system-bindings.table for the actions tiling has; RED: unknown variant), `a_system_action_without_meaning_in_tiling_is_dropped_with_a_warning` (RED). Both go through `serde_json::to_value(config)["bindings"]`, so they ran on the old type.
- Red e5eea8a4: tests/launch.rs `exit 1` back in the two cells plus `exec false` (3 FAILED at 5 s, "no change within 5 s"); kit unit cells `becomes` and `gone_by_polling` against stubs that answer false (4 FAILED).
- Green in 2e12231c and c29a3e9c. Also: kit `show_keys_is_the_old_spelling_of_keys`, `a_look_takes_over_the_start_of_a_command`.
- Not testable here, said as it is: the real pidfd_open failure and the unit active with pid null (no systemctl to fake without a process-wide PATH change); the polling function and `becomes` are tested with closures, the switch to them is two lines in `watch`.
- tiling 155, kit 248 + tests, construct, mc, xp all ok; `clippy --workspace --all-targets -D warnings` clean; tools/cut-check ok (18 reported, as before); size-ratchet ok.

## Coverage
`RUSTC_WRAPPER= cargo llvm-cov -p chora-kit --features chora-kit/testing`: actions.rs 89.7 % lines, actions/perform.rs 91.1 %, launch.rs 90.8 %, kit total 90.8 %. Uncovered: the shared-script branch of `perform` (would lock the screen), the pidfd and systemctl calls of `watch`.

## Merge gate on c29a3e9c
One run, `tests/merge-gate.sh --jobs 1`, all steps PASS (log dir /tmp/merge-gate.CpkYpH in the lane container only, copy /tmp/claude-0/gate-kit2c.log): red-first, pre-commit, gate-lib, build 138 s, test 85, clippy 14, ci-check 73, 2d-guard 121, package-*, xp, matrix 270, virtual, xp-faces, xp-paperclip, xp-fields, xp-question, xp-notepad, hud 64, layout 180, tiling-audit 305, tiling-look 524, chat-keys, doors 327, acts 52, last-words, cube 113, pull 65, placed, construct-hud 236, out, winit, wm-pane, pane-keys, walk, keys, pixels, mc-paths, mc-sofa, mc-handover, e2e 290. Door acts and tiling gate steps unchanged; the known-red list was not touched.

## Behaviour
Unchanged: same table for the configs of 29e2cd77, same acts. Visible: `keys` is also read as the tiling spelling, a tiling file with a binding it cannot act on loads with a warning instead of falling back to the defaults, `settings.set` of such a binding answers Ok and drops it (construct's rk2-1, same open point).

## Refactor proposals, ranked
1. `Host::Error` (S): tiling's `ClientError` is the only real error; the hooks could log and return (), then `Infallible` in mc and `let Ok(()) =` go. perform.rs.
2. `acts_in_construct`, `acts_in_tiling` and mc's refused set are three hand lists (S): a `Host::acts(&System) -> bool` default next to the hooks, and `Settings::refuse` as one kit helper over `Common` (config.rs of tiling and construct, ~15 duplicate lines).
3. NAMES, WORDS, OLD of actions.rs grew a fourth hand entry (`show_keys`) (S): derive from `System`'s schema, one unit test as the cheap step (kit2, kit2b proposal).
4. `settings.set` should turn the `refuse` warnings into INVALID_PARAMS (S, rk2-1), now for two looks.
5. tiling main.rs is 2.3k lines (L): the Host impl, `turn` and grab_all could move to an `acts.rs` as mc has; keys.rs `words` could read kit's words once KIT3 exists (M).
6. `launch::run` and `launch::start` differ only in waiting for choractl; with `Host::run` in place, tiling could take `start` if its stall (tens of ms on the event loop) is accepted (S, decision).
7. launch.rs `watch` has pidfd, poll and unit paths in one function (M): a `Follower` trait with the system and a fake would make the real fallback testable (needs an injectable systemctl).
Cut check: tiling uses chora-kit, chora-scene, chora-xp is named once in a test (main.rs:1661, known); no new breach.

## Review r1
Seeded review of claude/kit2c c29a3e9c (base 1d1aa5fd; kit6 ad88ebaf already MERGE-OK), one round, read-only on code. Looked at the merge 854981fe (`git show --remerge-diff`) and f58da09f, 2e12231c, e5eea8a4, c29a3e9c.
Verdict: MERGE-OK (0 BLOCK, 0 MAJOR, 0 MINOR, 2 NIT).

Checks:
1. Red first holds. At f58da09f (worktree, old tiling) `a_system_action_without_meaning_in_tiling_is_dropped_with_a_warning` and `one_binding_gives_the_same_system_action_in_every_look` FAIL, the golden `the_configs_of_29e2cd77_load_to_the_same_table` is green (recorded on the old code, as said). At e5eea8a4: 4 kit unit cells FAIL (`becomes` x2, `gone_by_polling` x2) and the 3 cells of tests/launch.rs (`exit 1`, `exec false`, editor chain) FAIL at 5 s; the other 2 launch cells are green. Stubs compile.
2. Tiling is on kit::actions: `Config.common: Common<Own>` (flatten), `Action = kit Action<Own>`, `impl Host for Tiling`; no `Action::(Run|Close|Give|Drag|Volume|ShowKeys)` left in plugins/chora-tiling (the new copies.toml row). The old table, parser and `give_focused` are gone. The two 29e2cd77 configs (default, user file) load to the recorded tables; `show_keys` loads as an alias of `keys`. An unfit binding goes through `Settings::refuse` (dropped, one warning naming the key) and File::load runs it (the cross-look test goes through File::load and the key is gone); before it was a parse error and the defaults. The file text says so in configuration.md. `cargo test -p chora-tiling`: 155 green; `cargo test -p chora-kit --features testing`: 248 + all integration tests green (the XDG_RUNTIME_DIR line in the log is a notify test, passes).
3. The cross-look test of kit2b gains tiling (`one_binding_gives_the_same_system_action_in_every_look`, same system-bindings.yaml/.table as mc and construct, with the kept list). I did not run doors/acts/tiling gate steps (no merge gate, per brief); the lane's gate log says they are unchanged and the known-red list is untouched, and the diff touches no gate script.
4. rw-1: `wrap` is `( {command}\n); status=$?; ...`, so `exit N` and a failed `exec` write the status; cells use `exit 1`, plus a new `exec false` cell. rw-2: `main_process_gone` returns `None` for pidfd_open failure (other than ESRCH) and a poll error (EINTR); `watch` then calls `gone_by_polling` (once a second, `becomes` sleeps `min(every, deadline-now)`, returns at the deadline: no busy loop); the pidfd is an OwnedFd dropped on return. A unit active with pid null: `pid.is_none_or` is true and `becomes(!unit_active)` polls each second to the deadline, then no Gone (the balloon closes by the 30 s rule, not as running forever). rw-3: `chora-launch-exit-` in launch.rs and the copies.toml pattern; D104 in docs/decisions.md states the subshell and says the rw-1 open point is closed.
5. tools/copies.toml: the tiling row (Action::Run etc., paths plugins/chora-tiling, max 0) holds; `tools/cut-check` ok (18 reported, the same set; fuzzel.ini and main.rs:1661 known); `tools/size-ratchet` ok (3 files over 1500, not grown). The merge 854981fe resolves the two conflicts (mc acts.rs, construct keys.rs) to the kit2b side with `launch::start` in perform.rs; nothing lost.

Findings:
- r1-1, NIT, crates/chora-kit/src/launch.rs:636, in the polling fallback a failing `systemctl` (unit_main_pid gives None) counts as "unit names another main process", so the program is "gone" at the first look; the pidfd path has the same rule, but it asks once, the poll asks every second. Harmless with the non-zero-status rule of Exited; fix: in `gone_by_polling` take `main_pid(unit)` as "other" only when it is `Some(p)` with p != pid, and treat None as not gone.
- r1-2, NIT, crates/chora-kit/src/actions/perform.rs:112 (override plugins/chora-tiling/src/main.rs:527), the doc of `Host::run` promises the launcher in front of `fuzzel` and a `Failure::NotStarted`, tiling's override does neither (same as before the port: raw command through `launch::run`, no failure). Fix: one doc sentence "a look may start the command its own way, as tiling does so that its event loop does not wait for choractl".
Known, not findings: the real pidfd_open failure and unit-active-without-pid are tested only through the seam (`becomes`, `gone_by_polling`); xp `bindings` is KIT2d; `settings.set` of an unfit binding answers Ok (rk2-1).
