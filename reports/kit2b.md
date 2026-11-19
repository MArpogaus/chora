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
