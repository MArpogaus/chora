# WM2 r1: kit::wm::focus and the core grab fix

2026-11-19 06:55 UTC. Lane: cloud, opus, branch `claude/wm2` from dc4bc46d. Head 751dfbcb. Brief: briefs/wm2-r1.md.

## Verdict
Built; the cells are red first, then green. Merge gate once on 751dfbcb (`--jobs 3`, run /tmp/merge-gate.6Lfqsk):
every step PASS, "all steps passed" (red-first, pre-commit, build, test, clippy, ci-check, acts, pane-keys,
keys, wm-pane, cube, pull, doors, matrix, virtual, tiling-look, mc-*, e2e among them). acts: 48 pass,
4 known red (5 before: the I3 row of mc went), 0 blocking. Independent review still to come (rules 7, 8).

## Commits (red first)
| commit | what |
|---|---|
| 93e50bfd | test: red cells. Core: I11 and the stale pointer; kit: skeleton of `kit::wm::focus` + 5 cells; mc: page node holds the focus |
| 767249d7 | fix(core): the key place; rule moved to `crates/chora/src/input/place.rs`; docs/api.md |
| 4b3190c1 | feat(kit): `kit::wm::focus` (Focus, Holder, Policy) |
| 1d07ef79 | refactor(tiling): ShownFocus (focus.rs) deleted, kit Focus; tiling's model is the Policy |
| d8fc17ce | refactor(kit): Policy sees the look (mc has no book on this base); holder follows the core focus |
| 3d2d778f | fix(mc): focus through kit; page node holds it; chat open = lost; mc Policy; known-red row I3 mc removed |
| c962eb5b | chore(cut): copies rows |
| 781f8957 | docs: glossary "Key place" |
| 751dfbcb | refactor(mc): `focused_dock`; region.rs back under its ratchet (2270; a first gate failed ci-check on +2 lines) |

## Red evidence (93e50bfd)
- `input::tests::a_key_that_a_look_binds_skips_its_focused_probe_and_reaches_the_look`: FAILED "the look gets
  Alt+y: []" (the probe got only Alt: no stray key, but the look was starved).
- `input::tests::the_keys_follow_the_keyboard_not_a_stale_pointer`: FAILED (Swallow instead of Grab).
- `wm_focus.rs`: 5 of 5 FAILED on the skeleton (two looks in two panes, own node, empty focus, policy, holder).
- mc `state::tests::mc_holds_the_focus_of_its_page_node`: FAILED (holder null; known red net2-a2).
All green from 767249d7 / 4b3190c1 / 3d2d778f on; `cargo test -p chora` 596 pass, chora-tiling 153, chora-mc 136.

## Cause and core fix (sh-24, the line)
input.rs:494-501 at dc4bc46d (`key_place`, the 461-519 rule before mv3): the place of a key was only the claim
under the pointer. With the keyboard in a pane manager's window and the pointer resting on another claim, the
grab of the focused manager was held back and swallowed: keyboard-only use followed a stale pointer, and the
person's key was lost. The stray raw key itself was already closed by da5a9c00 (the swallow); I11 now also
asserts the look gets its key. New rule (place.rs `key_place`): the manager a click chose (D55, while the focus
holds), else the owner of the keyboard focus (node, then window; a live claim holder), else the claim under the
pointer. Only managers whose claims all lie on virtual outputs wait for the place (unchanged). The safety layer
(Ctrl+Alt+BackSpace/Escape/i, VT keys) is decided in `press_action` before the rule; untouched.

## kit::wm::focus
`Focus {surface, sent, lost}`: hear/moved (`focus.changed`), gave, lose, take, sends, sent, to (Command::Focus),
to_node (Command::FocusNode), holder (Window | Node | Foreign | Nobody: what the core focuses, else lost),
state (the `state.get` focus), after(gone, why, policy). `Policy::focus_after(gone, why)`: the look is the
policy. Deviation from 2.2: no `view`/book argument (mc has no book on this base; the look reads its model).
tiling: history (window focused before). mc: the scene window that shows (after a dock), else the page node.
construct and xp: no change needed (no API break).

## Copies (tools/copies.toml)
`Command::Focus outside kit::wm` 20 to 18 (allow `crates/chora-kit/tests/wm_`); new: `ShownFocus` max 0,
`mc: Command::Focus` 2, `tiling: Command::Focus` 1, `Event::FocusChanged outside kit` 8. cut-check ok.

## Coverage
`cargo llvm-cov -p chora-kit --features chora-kit/testing` (without XDG_RUNTIME_DIR): kit::wm lines 98.3 %
(book 97.86, focus 98.63, give 100, kept 97.30, state 100; 6 of 359 lines missed).

## Visible behaviour
Changed only where the bugs were: a pane manager whose window or node has the keyboard gets its keys while the
pointer rests elsewhere (before: swallowed). Hover alone over another pane no longer moves the keys; a click
still does (D55 choice, pane-keys PASS). state.get: mc names itself holder while its page node has the keyboard;
tiling and mc name a window of theirs that the core focused even when they did not send it (I3 reads the core).

## Open, said plainly
- I3 "two panes" for xp is not closed: the core sends no event when a node of another client takes the
  keyboard, so xp (holder "xp" without a window, wm/state.rs:41) still claims the focus while the construct's
  walk node has it. Needs the core proposal 1 below; xp is not ported (brief: compile fixes only).
- I11 is a core cell, not an act: the act harness has no two-look pane session with a `--report` probe yet.
- No new act cells (real-press / keyboard-after-act) beyond the acts that ran: the change is core + model.

## Refactor proposals, ranked
1. Core: `focus.changed` also on node focus, with `node` and its owner (crates/chora/src/state.rs:1035,
   scene focus_node_command scene.rs:1012), so every look sees a foreign node take the keyboard; closes I3 for
   xp in panes. S-M.
2. xp and construct onto `kit::wm::focus`: xp `focused`/`core_focus`/`focus_away` (wm.rs:347-349,
   wm/state.rs:39-52) is the same model; construct plugin.rs:40. Lowers `Event::FocusChanged outside kit`. M.
3. `Focus::send(next) -> Option<Command>` so tiling's transaction (windows.rs:99-104, 135) builds no
   Command::Focus; tiling row to 0. S.
4. mc keeps `core_focus` beside kit Focus for the chat check (main.rs:686, 984): a `Focus::core()` reader. S.
5. Move tiling's ShownFocus cells (main.rs:1617, 1659) into crates/chora-kit/tests/wm_focus.rs. S.
6. `key_place` scans all nodes per key press (place.rs, claimant_at); cache the place per focus/pointer change. S.
Cut: no new breach (plugins use chora-kit and chora-scene only).
