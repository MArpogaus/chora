# kit2 r1 (F2): kit::actions with settings::Common, pilot mc

Branch claude/kit2 (from de057ab3). Commits: ab8ba65e (red: kit cells, mc golden), 0f8e0ed1 (kit::actions, settings::Common, mc on them), eb12e379 (From<System>/From<Look>, size ratchet).

## Done
- `chora_kit::actions`: `System` (Run, Terminal, Launcher, Keys, Chat, Lock, Power, Screenshot, Volume, Close, Give, Drag, Desk(Step), Cube(Turn)), `Action<L>` = System or the look's own enum (untagged on the wire, one spelling), `System::script()` for the shared scripts. Desk and Cube are values only.
- `chora_kit::settings::Common<L>`: `bindings` and `test_accept_injected_keys`; a look flattens it into its settings (flatten and deny_unknown_fields work together in serde: an unknown key is still refused).
- mc: own `Action` enum split into `Look` (next/previous scene, scene, toggle, solo) and kit `System`; `Config.common`. Its copy of the shared actions and of the two common keys is gone. A System value mc has no act for (terminal, launcher, close, desk, cube) is refused in `check` naming the binding, as before (an unknown action was a parse error).
- Error text: a wrong value of a shared action reports the shared side (`{volume: sideways}`), a wrong name reports the look side.

## Cells (red first)
- kit `tests/actions.rs`: one YAML binding gives one System action (15 spellings), a look name goes to the look, round trip through YAML, mistakes named, scripts. Red in ab8ba65e (parse stub), green in 0f8e0ed1.
- mc golden `the_configs_of_29e2cd77_load_to_the_same_table`: `tests/configs/29e2cd77/{mc,mc-docs-bindings}.yaml` against `.table` (key, JSON of the action), recorded on the old code (green before and after, table unchanged).
- mc `a_binding_set_through_the_settings_is_saved_and_read_back` (settings.set merge, save, reload through the flattened Common and the untagged schema); `a_system_action_without_meaning_in_mc_is_refused`.
- Local: cargo test -p chora-kit --features testing, -p chora-mc, clippy --workspace -D warnings all green.

## Merge gate
Not a full pass; stated as it is.
- Run 1 (0f8e0ed1): ci-check FAIL, size-ratchet: region.rs grew 2271 to 2280. Real; fixed in eb12e379 (net 0).
- Run 2 (eb12e379, --jobs 3): red-first, pre-commit, gate-lib, build, test, clippy, ci-check, 2d-guard, package-*, virtual, xp-*, hud, matrix, layout, chat-keys, tiling-audit, acts (door acts), last-words, cube PASS. `pull` FAIL "pane of the pulled window is not in front of the face at 1:1 or nearer: 170 px wide for 200": the known load flake F21 of reports/harness-r3.md (168/193 px, tests/gate-construct-pull.sh:150 reads once, sleep 1); construct untouched here. The gate stops at the first failure, so the steps after pull did not run.
- Run 3 (`--only pull,doors,mc-paths,mc-sofa,mc-handover`, serial): all PASS (doors 324 s, pull 63 s, mc-paths 66 s, mc-sofa 27 s, mc-handover 10 s).
- Not run on this head: tiling-look, construct-hud, out, winit, wm-pane, pane-keys, walk, keys, pixels, e2e. None touch mc; kit change is additive (new module, one new type in settings). The PM runs the full gate at merge.
- Logs: /tmp/merge-gate.YlnO3E (run 2), /tmp/merge-gate.rMETol (run 3), in the lane container only.

## Behaviour
Unchanged by design: same spellings, same table, same acts (door, drag, volume claim). mc.yaml untouched.

## Refactor proposals, ranked
1. Port construct next (KIT2b): its `Action` has Terminal/Launcher/Close/Chat/Lock/Power/Screenshot/VolumeUp/VolumeDown/Mute. Old spellings `volume_up`, `volume_down`, `mute`, `help` need serde aliases for one release (C10 promise); do it in `Action<L>`'s System side, not in the look.
2. Move `System` performing (run_shared, volume, give, chat) into `actions::perform(&mut Runtime, &System, &dyn Notice)` once a second look is on it; today only mc performs, so the match lives in mc/acts.rs. Doing it with one user would guess the seam.
3. `names_system` duplicates the variant list by hand; derive it from the schema or a strum-like const when a variant is added (a unit test could compare it to `System`'s schema).
4. `Action<L>` goes through `serde_json::Value`; fine for bindings, but a generic `Content`-free path would give line numbers in file errors. Low value.
5. keysheet (KIT3) should take its words from `System`, so mc keys.rs `words()` and `volume_order` go; wait for the other three looks.
6. xp has no `bindings`; it gains `Common` with defaults equal to today's hard-coded keys (KIT2d).
