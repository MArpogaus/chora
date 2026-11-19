# wm7b r1 (F3): the xp cube hosted on construct — handed over at 250 k, not done

Lane: opus high, cloud, branch `claude/wm7b` from f4e4f4d9. Head 1ae3f243 (pushed). No gate run yet.
Commits: 5322c586 `test(acts)` red (modifier drag, kill without restart, restart, pager without construct);
e450d672 `fix(core)` (+test, red first: a button grab of one client ends another client's lone-modifier tap,
so Super+drag on a face no longer opens xp's Start menu at the release; found by the new act, I4 xp-menu);
8ca85eab `fix(xp)`; 1ae3f243 `fix(construct)`.

## Items (cell: tests/acts/run.py construct on 1ae3f243's build, before the last act fix)
1. (a) PASS act "a drag with the modifier on the front face turns the cube" (construct cube/hand.rs; xp grabs no
   Super+BTN_LEFT while hosted or flat). `turn_to` uses `chora_kit::cube::quarters_to` (copy gone).
2. (b) a2 PASS act "the pager turns the cube" (xp pager on faces calls `cube.turn {to}`; row wm7a-a2 removed).
   rw-5 was in r2 already. a1 OPEN: I3 still breaks in "the key of the step back stands back": the core
   focuses construct-look, xp says it holds the probe window (xp ignores focus.changed {surface: null}).
   Row wm7a-a1 stays, but its match no longer fits: acts BLOCK on I3. xp no longer claims the focus without a
   window on virtual-only outputs.
3. (c) m2 PASS act "construct killed mid-turn leaves the windows with a flat xp within 3 s" (xp has no
   PDEATHSIG, stderr to a followed file; on the host's disconnect xp takes the screens, face i -> desk i).
   PASS act "construct started again reopens the cube with the windows on its faces" (construct reopens around
   the running xp; xp goes back on the role event); m1 covered there (start card not beside panes).
   Last run's I1 FAIL ("construct answers no state.get") is fixed in 1ae3f243 (s.looks), not re-run.
4. m3 (rw-8) done: `last_word` + `tell_end`, unit test in role.rs. m4 OPEN (MANAGER still the constant "xp").
5. Flat fallback: xp on virtual outputs without a host switches desks flat, key list "Switch to the desktop on
   the left/right" (help.rs unit test). OPEN: act "without construct the pager of xp switches the desk" got
   desk 4 for a click on button 2 (not diagnosed). xp's own cube on a screen stays (deviation: removing it
   is a visible change to the xp flavor and large; proposal below).

## Not run / open for the next round
- `tests/gate-construct-cube.sh` section 6 still expects "the xp of the cube ends with a construct that dies"
  and "a restarted construct removed the outputs": must change to the new behaviour (it will FAIL now).
- No `tests/merge-gate.sh --only ...` run, no kit coverage run, docs/api.md (input.key: tap ended by a button
  grab; xp hosted/flat) and configuration.md (cube_manager outlives the construct; xp pager on faces) and the
  decision record line: not written.
- `cargo test -p chora-construct -p chora-xp -p chora-kit --features chora-kit/testing`: all ok on 1ae3f243
  minus the last two clippy edits (clippy --workspace clean after them). Size: cube.rs shrank (launch,
  manager_runs, remove_left_outputs moved to cube/start.rs).

## Refactor proposals
- xp/cube.rs + wm/desktops.rs turn rendering (~500 lines): drop xp's own 3D cube now that the cube lives in
  construct; flat desk switch on screens (owner call: visible change to the xp flavor) (L).
- xp focus: one `focus_away` that also counts a focused node of another client (wm/state.rs:41) (S).
- construct `Host.child: Option<Child>` plus `end_manager`: a small `Manager` type with child, tail, command (S).
- tests/acts: `click_box` (construct.py) belongs in harness.py next to `click` (S).
Cut: core touched (input.rs, one rule); no plugin-to-plugin name added (xp finds the host by the role).
