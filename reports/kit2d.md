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

## Review r1

Reviewer: review lane (sonnet, high), read-only on code; claude/kit2d head 888dfb60 against base 18231aa1. No merge gate run.
Verdict: **FIX** (0 BLOCK, 1 MAJOR, 1 MINOR, 2 NIT).

Checks:
1. Red first: c316198e (cell) comes before 09b1c797 in `git log`. Run at c316198e, the cell fails on the assertion (left `{}`, right the 23 keys), not on a parse error. OK.
2. Defaults: the 23 keys of the old `KEYS` (wm.rs at 18231aa1) are the 23 of `HANDLED` and of xp.yaml, same action each (Help and HelpAlias to `keys`, Turn(-1/1) to `{cube: left|right}`, Carry to `{carry: ..}`, Switch(1/-1) to `{switch: next|previous}`); none lost, none added. `cargo test -p chora-xp`: 204 passed.
3. kit hook f7bcc021: 13 lines in perform.rs, default `Ok(false)`, called only for `System::Power`; mc and construct unchanged. `cargo test -p chora-kit --features testing`: all green (224 lib); `cargo test -p chora-tiling -p chora-mc`: 140 and 152 passed. OK.
4. Visible changes: Super+/ beside Super+F1 and chords in alphabetical order are POLS-safe (one more row entry; the Start menu row now reads "Ctrl+Escape, Super"). Lock and Screenshot through `perform` match mc and construct; a missing script still gives the balloon. Unfit bindings (`desk`, `terminal`, `drag`, `run`, `give`) are dropped with one named warning in `refuse` (called by `complete`). `settings_changed` regrabs (`start_grabs`, `grab`, `end_grabs`), and a failed regrab is rolled back by the runtime (runtime.rs:581). One gap: rw-1.
5. No new System variant (actions.rs untouched); xp-only keys stay in `Look`; no keysheet. OK.
6. `tools/cut-check` ok; both new copies rows count 0 (grep rc=1). docs/api.md and docs/configuration.md name `bindings` and the defaults (rw-3, rw-4).

Findings:
- rw-1, MAJOR, plugins/chora-xp/src/wm/keys.rs:125 (`Event::InputButton` arm): every button binding starts the cube drag, whatever its action; the event carries `binding` and the arm ignores it. `Super+BTN_RIGHT: close` turns the cube and never closes; `Super+x: cube` (a key, not a button) does nothing, silently. Fix: look up `self.rt.action(&binding)` in the arm and call `grab_cube`/`release_cube` only for `Look::Cube` (run `self.key` for another action on press), and have `refuse` or `check` drop with a warning a `BTN_*` binding to anything but `cube` and `cube` on a non-button key.
- rw-2, MINOR, plugins/chora-xp/src/wm.rs:815 (and help.rs:166): `fallback(text)` keys on the spelling of the binding, not on its action; `Ctrl+Alt+Left: close` also takes `Super+Ctrl+Left` when the core refuses the first, against docs/configuration.md ("a key that `bindings` moves has no such stand-in"). Fix: give a stand-in only while the action is the default one of that key (compare the action, or the pair).
- rw-3, NIT, docs/configuration.md:2234: the `bindings` row of xp lists `{volume: up|down|mute}` but not `{volume: {set: N}}`, which xp.yaml names and the code accepts. Fix: add it.
- rw-4, NIT, docs/api.md:468: the paragraph now says twice which looks are on `bindings` ("xp acts on `cube` ... mc, construct and xp are on it ...") and the lines are not rewrapped. Fix: one sentence per fact, wrapped as the rest.

## r2

Lane: build (sonnet, high), claude/kit2d. Head 188b642d. The session branch moved, so a7ba4daa merges it (conflicts in `plugins/chora-xp/src/wm/state.rs` and `docs/api.md`: kept the port's `kit::wm::state` shape and added `texts` from NET3).

Red first: fb02ad6d adds four cells. Run at fb02ad6d, all four fail on an assertion, none on a build error: `a_button_bound_to_close_closes_and_turns_no_cube` (wm.rs), `a_button_and_the_cube_go_together_or_are_dropped` (config.rs), `a_moved_cube_key_has_no_stand_in` (wm.rs), `a_moved_cube_key_shows_no_stand_in` (help.rs).

Closed in 188b642d (fix and docs in one commit, since a commit-message hook refused the first try):
- rw-1: `Event::InputButton` goes to `Wm::button`, which acts on `self.rt.action(&binding)`: `cube` grabs at the press and releases at the release, another action runs `self.key` at the press. `Config::refuse` drops with one warning a `BTN_*` binding to anything but `cube`, and `cube` on a key that is no button.
- rw-2: `fallback(binding, action)` gives the stand-in only while the action is the default one of that key (`default_of`); `grab_keys` and `help::rows` pass the action.
- rw-3: docs/configuration.md names `{volume: {set: N}}`, and the new refusal of rw-1.
- rw-4: docs/api.md: one sentence per fact, rewrapped.

Checks: `cargo test -p chora-xp` 209 passed; `cargo test -p chora-kit --features testing` all green (224 lib). `tests/merge-gate.sh --only xp,keys,acts,hud,cube` (after `cargo build --workspace --locked`, which the subset does not do): xp PASS 90 s, hud PASS 69 s, acts PASS 66 s, cube PASS 107 s, keys PASS 18 s. A first run without the build failed in `xp` at "chora-xp starts nested" (no binary), not a code finding. No full gate.
