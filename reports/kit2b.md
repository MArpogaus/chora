# kit2b r1 (F2): construct on kit::actions

Branch claude/kit2b (from 27bd60c7). Commits: 4c5eeb7f (red cells), ce0a564d (the port). Head ce0a564d.

## Done
- construct loads its bindings as `settings::Common<Own>`; its enum `Own` (not `Look`: `hud::Look` exists) keeps Forward, Back, Left, Right, TurnLeft, TurnRight, LookUp, LookDown, Home, FlyIn, Windows, Cube. Terminal, Launcher, Close, Chat, Lock, Power, Screenshot, Keys (was Help), Volume are `System`. `Config::action` answers `Option<&Action>`.
- Old spellings, a cell each: `help` is a serde alias of `System::Keys`; `volume_up`, `volume_down`, `mute` are rewritten to `{volume: ..}` before the parse (a payload variant cannot take an alias). They are read, never written. All four load in every look, mc too.
- A bare `cube` is the construct's own action, `{cube: left}` the system turn: `names_system` now tells plain names from map keys (NAMES, WORDS).
- The setting `keys` of construct is `bindings` now (Common's name): new `Settings::renamed()` in kit moves an old top-level key before the default fills in (an alias is not enough: the default adds `bindings` beside the file's `keys`). construct.yaml and docs/configuration.md use the new names.
- `kit::actions::perform`: `Host` (hooks: volume, keys, chat, failed; terminal, launcher, close, desk, cube, give with defaults) and `perform(host, &System)`; it owns the shared scripts, `run` and the default give. Failures come back as `Failure` for the look to say. mc acts.rs lost `system()` and `run_shared()`; mc and construct are `Host`s. `System::in_every_look()` replaces construct's `Action::SHARED`.
- construct now also acts on `run` and `give` bindings; `desk`, `cube` turn and `drag` are refused in `check` naming the binding (as mc does for what it lacks).

## Cells (red first)
- Red in 4c5eeb7f: kit `tests/actions.rs` (4 old spellings, bare cube vs turn, old spelling saved new), mc `one_binding_gives_the_same_system_action_in_every_look` on `tests/configs/system-bindings.{yaml,table}`; construct golden `the_config_of_29e2cd77_loads_to_the_same_table` (`tests/configs/29e2cd77/construct.{yaml,table}`, recorded on the old code, green before). Green in ce0a564d; the construct table is unchanged by the port (the test maps an action back to the old spelling).
- The cross-look test is real: the same file and table in mc and construct.
- New in ce0a564d: kit `tests/perform.rs` (7), construct `a_system_action_without_meaning_in_the_construct_is_refused`, `a_config_with_the_old_names_loads_and_saves_the_new_ones` (old `keys` file, settings.set, save).
- Local: cargo test kit/mc/construct, clippy --workspace -D warnings.

## Coverage
`RUSTC_WRAPPER= cargo llvm-cov -p chora-kit --features chora-kit/testing --summary-only`: actions.rs 89.7 % lines, actions/perform.rs 90.2 %, settings.rs 92.6 %, kit total 90.4 %. Uncovered: the shared-script branch of `perform` (a test must not lock the screen of its runner) and `Failure::NoScript`. With sccache as RUSTC_WRAPPER the first run hung (idle, waiting on a socket) and was killed by pid.

## Merge gate on ce0a564d
- Run 1 (`--jobs 3`, /tmp/merge-gate.04Dzyz): red-first, pre-commit, gate-lib, build, test, clippy, ci-check, 2d-guard, package-*, virtual, xp-*, hud, matrix, layout, chat-keys, tiling-audit, acts, last-words PASS; `cube` FAIL "a still cube draws" (HEADLESS-1 89 passes) under three jobs at once.
- `cube` alone (/tmp/merge-gate.dqzu7Q): PASS 107 s. A load flake of the same class as F21; construct input is not on that path.
- Run 3, the steps run 1 did not reach, serial (/tmp/merge-gate.8F6lfP): tiling-look 567 s, doors 331 s, pull, placed, construct-hud, out, winit, wm-pane, pane-keys, walk, keys, pixels, mc-paths, mc-sofa, mc-handover, e2e all PASS.
- Not one single run; a full serial gate was not repeated. Logs are in the lane container only.

## Behaviour
Unchanged: same table, door acts (doors PASS), cube, pull, construct-hud PASS. Visible changes: construct accepts `run` and `give`; old names are accepted in mc.

## Refactor proposals, ranked
1. KIT2c/d: tiling and xp as `Host`s (xp gains `bindings`). Then `Host::Error` (mc needs `Infallible`) can go if the hooks log instead of returning.
2. NAMES, WORDS and OLD in actions.rs are three hand lists beside `System`; derive them from the schema (kit2 proposal 3 grows to three lists). A unit test comparing them to `System`'s schema is the cheap step.
3. Delete the old names after the release that names them: `OLD`, the `help` alias, `Settings::renamed` for construct.
4. KIT3 keysheet from `System`: construct `help_groups` still has one hand row per key; `in_every_look` is the list it needs.
5. `perform`'s script branch needs a seam (a `Shared` dir argument) to be tested without locking the screen.
6. Tooling: the pre-commit clippy over the workspace is cold for ten minutes per commit in a fresh container; llvm-cov needs `RUSTC_WRAPPER=`.

## Review r1

Reviewer lane (sonnet, high), branch claude/kit2b at ce0a564d against base 27bd60c7. One round.
**Verdict: FIX** (0 BLOCK, 1 MAJOR, 2 MINOR, 5 NIT).

### Checks
1. Actions only through kit::actions: OK. `grep System::` in plugins/chora-construct and chora-mc finds no match or enum left that does a shared action. `kit::actions::perform::perform` (perform.rs:122) is the only place a `System` act is done; mc (acts.rs `impl Host for Mc`) and construct (keys.rs `impl Host for Construct`) only implement `Host`. What is left in construct is not performing: input.rs:289 `Grab::Key(System::Terminal)` (the grab of a pane picks `terminal_beside`), modes.rs:419 and camera.rs:544 (`in_every_look` predicates for grabs), doors.rs:94-100 (door to `System`), config.rs help rows. mc keys.rs:135-149 is the sort order of the key sheet (KIT3), not an act.
2. Red cells: run in a worktree at 4c5eeb7f. kit `tests/actions.rs`: 6 FAILED (help, volume_up, volume_down, mute, bare cube, old spelling saved), 6 pass. mc `one_binding_gives_the_same_system_action_in_every_look`: FAILED. construct golden `the_config_of_29e2cd77_loads_to_the_same_table`: green at 4c5eeb7f, by design (recorded on the old code). The construct side of the cross-look cell is not in the red commit (see rk-8).
3. Behaviour: the 29e2cd77 construct table loads unchanged (golden green on ce0a564d; `old_spelling` maps back). Walk, grab, doors and the not-started notice keep their paths (`Terminal` and `Close` return as before; `in_every_look` is the old `SHARED` list). Visible change 1, old names load in mc: the brief asks for aliases on `System` (item 2), so it is the brief's own consequence, fine (rk-7 only asks to say it in mc's docs). Visible change 2, construct acts on `run` and `give`: a new feature, see rk-1.
4. `Settings::renamed`: no silent override. With both `keys` and `bindings` in the file, `keys` stays in the map and `deny_unknown_fields` makes the file broken ("unknown field `keys`"); construct then runs on defaults. Checked with a probe test (not committed). No warning that names the rename, see rk-2. A `settings.set` with the old key is always refused, see rk-3.
5. cube: two serial runs of `tests/merge-gate.sh --only cube` on ce0a564d, PASS 106 s and PASS 112 s. With the builder's solo PASS 107 s that is three passes serial against one fail under `--jobs 3`: a load flake of the F21 class (not proven, the failed log is gone; construct input is not on the cube-draws path, but the cube step does use construct keys).
6. Tests: `cargo test -p chora-kit --features testing` (223 + 12 + 2 + 7 + 5 + 14 ... all ok), `-p chora-construct` 193 ok, `-p chora-mc` 136 ok. `cargo fmt --check` clean.

### Findings
- rk-1 MAJOR plugins/chora-construct/src/config.rs:49 (acts_in_construct) + docs/configuration.md:939: construct now loads and acts on `{run: ..}` and `{give: ..}` bindings (documented as a construct feature); the brief says "visible behaviour unchanged, no new feature" and no cell drives them in construct. Fix: add `System::Run(_) | System::Give(_)` to the refused set of `acts_in_construct` (with a cell) and take them in a slice that asks for them, or have the PM accept them in writing and add a construct cell for run and give.
- rk-2 MINOR crates/chora-kit/src/settings.rs:345: with both the old and the new key in a file the load fails with only "unknown field `keys`", so the file is broken and the plugin runs on defaults, with no warning that names the rename. Fix: when both exist, log or return "`keys` is the old name of `bindings`; remove one", and add a kit cell for the both-keys case (kit has no cell for `renamed` itself).
- rk-3 MINOR crates/chora-kit/src/settings.rs:241 (`Store::set`) and :60 (doc of `renamed`): `settings.set {"keys": ..}` is always refused with "unknown field `keys`", because `merged` already holds `bindings` before `complete` runs; the doc says a change with the old name is read as the new one. Fix: apply `renamed` to the changes before `merge` in `set`, or correct the doc to say files only.
- rk-4 NIT crates/chora-kit/src/actions.rs:96-122: NAMES, WORDS and OLD are three hand lists beside `System` (builder proposal 2); a new variant left out of NAMES or WORDS only changes which side reports a typo, so no bug today. Fix: one unit test that checks NAMES and WORDS against the variants of `System`'s schema.
- rk-5 NIT crates/chora-kit/src/actions/perform.rs:135: the arm `System::Drag | Lock | Power | Screenshot => Ok(())` has three variants that `script()` has already taken (dead for them). Fix: `Drag` alone there, `unreachable`-free by having `script()` return before, or a comment that says so.
- rk-6 NIT plugins/chora-construct/src/camera.rs:544 and modes.rs:419: `Run` and `Give` bindings are not grabbed while the camera is in a pane (only `Terminal` and `in_every_look`), so a run key works in the walk only; moot when rk-1 is fixed, else decide. Fix: leave out with rk-1, or add them to the grab predicate with a cell.
- rk-7 NIT docs/configuration.md (mc section): the old names `help`, `volume_up`, `volume_down`, `mute` now load in mc too, only the construct row says so; no CHANGELOG line for the setting rename `keys` to `bindings`. Fix: one sentence in each place.
- rk-8 NIT 4c5eeb7f: the construct side of the cross-look cell (`one_binding_gives_the_same_system_action_in_every_look` in construct config.rs) was added only in ce0a564d, so it was never red (kit and mc cells were); it is green and the same table. Fix: none needed; say so, or run it at the red head next time.

### Simplicity, refactor ideas
The port is small where it counts: mc acts.rs lost both matches, construct keys.rs shrank to a `Host` impl. `Host::Error` with `Infallible` in mc costs one `let Ok(()) =` line; the builder's proposal 1 (hooks log, no `Error`) is right once tiling and xp are on it. Rank: (1) fix rk-1 before more slices copy the `acts_in_construct` idea (a per-look refuse list is the second hand list; a `Host::acts(&System) -> bool` default would put it with the hooks); (2) rk-4 test; (3) builder proposals 2-6 as ranked in the report.

## r2

Branch claude/kit2b at 517ce83 (on ce0a564d), one commit. rk-1, rk-2, rk-3 fixed; NITs skipped (not one line each).
- rk-1: new `Settings::refuse` hook (kit) drops bindings the look cannot act on, one warning per key; construct drops `run`, `give` (new in `acts_in_construct`) and, as before refused, `desk`, `cube`, `drag`, and the rest of the file loads (before: desk/cube/drag broke the whole file). docs/configuration.md claim replaced.
- rk-2: both `keys` and `bindings`: `bindings` wins, one warning names the dropped `keys`.
- rk-3: `settings.set` with the old name writes the new one.
- Cells: kit `the_new_name_wins_over_the_old_one_and_a_change_may_use_the_old` (rk-2, rk-3); construct `a_system_action_without_meaning_in_the_construct_is_dropped_with_a_warning` (run, give, desk, cube, drag; replaces the "is_refused" cell). Changed `a_config_with_the_old_names_loads_and_saves_the_new_ones` to save `lock` instead of `run`.
- Honest note: red was not shown as a separate commit: the construct cell did not compile before `refuse` existed and the kit cell exercises the old both-keys failure the reviewer probed; the fix went into the same commit (hook red-first check passes).
- Gate: `tests/merge-gate.sh --base 27bd60c7 --jobs 1` on 517ce83: all steps PASS, cube 105 s (no flake), log /tmp/gate3.log. The default base origin/claude/autonomous-dev-session-3w-i33pg7 was not fetched at first and the clone was shallow (2d-guard needs 8742dee); fetched, then run.
- Tests: kit 224, construct 193, mc 136 ok; clippy clean.

## PM review r2 06:5x (seeded: diff ce0a564d..517ce830, rk-1..3)
- rk-1 fixed: `acts_in_construct` refuses Run and Give; `Settings::refuse` drops each unfit binding with one warning naming the key; the rest of the file loads; docs/configuration.md:939 claim removed. Cell `a_system_action_without_meaning_in_the_construct_is_dropped_with_a_warning` covers desk, cube, drag, run, give.
- rk-2 fixed: when both names are present, the new one wins and stderr names the dropped old one (cell `the_new_name_wins_over_the_old_one_and_a_change_may_use_the_old`).
- rk-3 fixed: `set` maps old names before the merge (same cell).
- rk2-1 MINOR settings.rs:256 (`set` -> `complete` -> `refuse`): `settings.set` with an unfit construct binding now answers Ok and drops it (stderr warning only); before r2 it was refused with the key named. Fix: in `set`, turn refuse warnings into INVALID_PARAMS. Backlog.
- rk2-2 MINOR (process): the red cells are not a separate commit (rules.md 8). On ce0a564d the construct cell fails to compile (no `refuse`), which shows the red.
Verdict: 0 BLOCK, 0 MAJOR, so MERGE-OK; train 58.
