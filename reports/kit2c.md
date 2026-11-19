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
