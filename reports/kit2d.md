# kit2d r1 (F2): xp on kit::actions

Branch claude/kit2d (from 1d1aa5fd, merged claude/wm1d 18231aa1; the session branch did not move). Commits:
c316198e (red cell), f7bcc021 (kit `Host::power`), 09b1c797 (the port), 888dfb60 (coverage cell). Head 888dfb60.

## Done
- xp loads `settings::Common<Look>` (`#[serde(flatten)]`, `test_accept_injected_keys` moved into it). `Look` is xp's own enum:
  `Files, Desktop, Tap, Start, Switch(Step), Carry(Turn), Cube` (bare `cube` is the pointer turn, as in construct).
  The alias is `config::KeyAction` (`startmenu::Action` exists).
- The 23 hard-coded keys of `KEYS` are the defaults in xp.yaml (`bindings:`); `KEYS`, `enum Key` and the key match are gone.
  System keys go through `kit::actions::perform` with `Acting` (a `Host` over `&mut Wm` that collects the scene commands
  of the hooks); xp's own actions are `Wm::look`. Ctrl+Alt+Left/Right is `{cube: left|right}` (the System action); no new
  System variant. Carry, Switch, Files, Desktop, Start, Tap, Cube stay xp actions (item 3).
- The four cube keys keep their Super+Ctrl stand-in as `wm::FALLBACKS` (only for the default binding; a moved key has none).
- `acts_in_xp` + `Settings::refuse`: bindings to `terminal`, `run`, `give`, `drag`, `desk` are dropped with one warning
  (no new feature, as construct r2). `check` parses every binding and refuses two spellings of one key.
- `settings_changed` now grabs again (`start_grabs` / `grab` / `end_grabs`), so `settings.set` and `settings.reload` move
  keys at once; before, xp had no keys in its settings.
- kit: `Host::power(&mut self) -> Result<bool, _>`, called by `perform` before the shared script; default `false`, so mc
  and construct are unchanged. xp opens its Turn Off dialog there (the old `Key::Power`). Without it xp could not
  keep the dialog, because `System::script()` takes Power first.
- help.rs builds its rows from `bindings` (`LINES` keeps the old row order); copies.toml: two new rows at 0
  (`const KEYS: [` and `pub enum Key {` in plugins/chora-xp; xp had no action row before). docs/configuration.md (row
  `bindings` and the table of defaults), docs/api.md, xp.yaml name the key and its defaults.

## Cells (red first)
- Red in c316198e: `the_default_binds_every_key_that_xp_handled_on_1d1aa5fd` (23 keys and action of each, against the
  empty table: `bindings` was missing). Green in 09b1c797.
- New in 09b1c797: `one_binding_gives_the_same_system_action_in_every_look` in xp (same yaml and table as mc and
  construct); `the_default_bindings_all_act_in_xp`; `a_system_action_without_meaning_in_xp_is_dropped_with_a_warning`
  (desk, terminal, drag, run, give); `a_binding_that_is_no_key_fails`; `set_moves_a_key_and_the_next_start_reads_it`;
  `a_key_does_the_action_of_its_binding` (Super+F1, Super+/, power dialog); the old key tests of wm.rs and help.rs now
  read the default bindings. kit `each_system_action_reaches_its_hook` gains Power.
- Honest: the cross-look cell for xp was never red (kit already reads the file; the cell is green with the port, the
  same note as kit2b rk-8). The kit `power` hook has no separate red commit (the cell does not compile before it).

## Coverage
`RUSTC_WRAPPER= cargo llvm-cov -p chora-kit --features chora-kit/testing --summary-only` (needs `XDG_RUNTIME_DIR` set
to a dir of mode 700): actions.rs 89.7 % lines, actions/perform.rs 91.2 % (was 90.2), settings.rs 92.6 %, kit 90.45 %.

## Merge gate on 888dfb60
`tests/merge-gate.sh --jobs 1`, one run, log dir /tmp/merge-gate.qHHqyr (lane container only): all 41 steps PASS ("all steps
passed"): red-first, pre-commit, gate-lib, build 149 s, test 92 s, clippy, ci-check, 2d-guard, package-*, xp 85 s, matrix
271 s, virtual, xp-faces, xp-paperclip, xp-fields, xp-question, xp-notepad, hud, layout 183 s, tiling-audit 321 s,
tiling-look 550 s, chat-keys, doors 329 s, acts 52 s, last-words, cube 104 s, pull, placed, construct-hud, out, winit,
wm-pane, pane-keys, walk, keys, pixels, mc-paths, mc-sofa, mc-handover, e2e 296 s. Known-red was not touched.

## Behaviour changes to know
- Help and Support shows Super+/ beside Super+F1 in the row "Open this help" (it had no row); the chords of a row now
  go in alphabetical order of the binding (the table is a `BTreeMap`), the rows keep their order.
- Lock and Screenshot keys run through `perform` (`choractl run` with the activation token, as in mc and construct)
  and not through `session::run`; a missing script still gives the balloon "Cannot do that".

## Refactor proposals, ranked
1. `Settings::refuse` + `acts_in_*` is now the same 12 lines in construct and xp (and mc refuses in `check`): one
   `Host::acts(&System) -> bool` default in kit and one generic `refuse` helper (kit2b proposal, still open).
2. A per-binding stand-in (`FALLBACKS`) is xp-only; construct and tiling have the same problem for held keys. A
   `fallback: Option<String>` beside the action in `Common` would drop the const, but it is a feature (not in this round).
3. NAMES / WORDS / OLD in kit actions.rs (kit2b proposal 2) is still three hand lists.
4. xp `Acting` collects commands because the hooks return `Result<(), _>`: a `Host` with a hook result type `Vec<Command>`
   would not need the wrapper, but mc and construct would pay for it; leave until tiling (KIT2c) shows the shape.
5. Tooling: `cargo llvm-cov` of kit fails without `XDG_RUNTIME_DIR` (one kit test needs it); the pre-commit clippy is
   cold for about 6 minutes per commit in a fresh container.
