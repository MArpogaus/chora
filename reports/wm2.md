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

## Review r1
2026-11-19 UTC. Lane: cloud, opus, read-only on code; claude/wm2 head 751dfbcb, base dc4bc46d. Brief: briefs/review-wm2-r1.md.
Verdict: **FIX** (1 MAJOR open). Counts: BLOCK 0, MAJOR 1, MINOR 2, NIT 2.

Checks run (head 751dfbcb):
1) key place: `cargo test -p chora input::` 46 pass. Safety layer (VT, Ctrl+Alt+BackSpace/Escape/i) is decided in
   `press_action` before `grab_action`; the injected path is unchanged. A key a look binds reaches the look (I11 cell);
   a hovered manager's grab is held back and swallowed, no raw key to a hovered client. Rule matches docs/api.md.
2) red first: 93e50bfd is the oldest of the 9 commits and holds only cells + kit skeleton; cells not re-run at 93e50bfd
   (two extra core builds, not cheap); the report's red evidence is consistent with the diff.
3) `kit::wm::focus` (Focus, Holder, Policy) is generic and 167 lines; ShownFocus gone (row max 0 holds); mc and tiling
   go through kit. `cargo test -p chora-kit --features testing` all pass (223 lib + integration), chora-mc 136,
   chora-tiling 153.
4) visible behaviour: see rw-1 (a click does not always bring the keys back).
5) `tools/cut-check` ok, copies rows exact (no grown/lower report); `tools/size-ratchet` ok; region.rs 2520 lines
   (2521 at base), ratchet 2270.

Findings:
- rw-1, MAJOR, plugins/chora-mc/src/manage.rs:204, a click on mc's own window after another look took the keyboard
  sends no focus (compares `focus.surface`, ignores `lost`); with the new key place (767249d7) mc's bindings then stay
  swallowed and typing stays in the other pane, so "click does" (D55, docs/api.md "until one of its windows takes it
  back") fails for mc in panes; before, hover at least gave mc its grabs. Confirmed by a probe cell (click s1,
  `focus.changed` other, click s1: no Focus sent). Fix: call `self.focus.take()` and send when
  `self.focus.sends(Some(node))`, with that cell.
- rw-2, MINOR, plugins/chora-mc/src/main.rs:1017, `take_free_keyboard` sends `Command::FocusNode` around kit `Focus`,
  so at start and after a panel rebuilt the page the page node has the keyboard but `state.get` says holder null (the
  I3 mc cell covers only `keyboard_to`). Fix: send `self.focus.to_node(Some(page_id(&name)))`.
- rw-3, MINOR, crates/chora/src/input/place.rs:54, `key_place` takes the window owner from `focus_wanted` and ignores
  `layer_focus`: while a clicked layer surface (panel search field) has the keyboard, the place is the owner of the
  window that had it before, not the pointer (old) or nobody. Fix: skip the window owner while `state.layer_focus` is
  set, with a core cell.
- rw-4, NIT, plugins/chora-tiling/src/windows.rs:185, `forget_window` has two stacked doc paragraphs; the old one
  ("when the window comes back, tiling sends it again") no longer describes the policy. Fix: merge into one.
- rw-5, NIT, crates/chora-kit/src/wm/focus.rs:40, `surface`/`sent`/`lost` are pub and tiling (windows.rs:195) and mc
  (show.rs:159) write them directly, past `node`/`core`. Fix: a `Focus` method for "forgot, next is X" and private
  fields (fits proposal 3).
Known, not findings: I3 for xp open (core proposal 1); no mc book on this base.

## r2
2026-11-19 UTC. Lane: cloud, sonnet, branch claude/wm2, head 12305e5a (base 751dfbcb). Brief: briefs/wm2-r2.md.
Red first: fa6ba541, one cell per finding, each fails on 751dfbcb on its own assertion:
rw-1 `a_click_on_its_window_after_another_look_took_the_keyboard_sends_the_focus` (mc, one Focus sent, none on the second click),
rw-2 `a_free_keyboard_taken_by_the_page_shows_in_the_state` (state.get holder null), rw-3
`the_key_place_is_not_the_window_owner_while_a_layer_has_the_keyboard` (core, Grab instead of Swallow; the layer is mapped).
Fix: 12305e5a. rw-1 `focus.take()` and send when `focus.sends(Some(node))` (manage.rs); rw-2 `focus.to_node(Some(page_id))`
(main.rs); rw-3 `State::layer_has_keyboard()` and `key_place` skips the window owner while it holds (place.rs; the node
owner and the safety layer are unchanged); rw-4 one doc on `forget_window`. rw-5 not done (needs a Focus method and private fields, more than one line).
Green: `cargo test -p chora --lib input::` 47, `-p chora-mc` 138, `-p chora-tiling` 153, `-p chora-kit --features testing` 223 + integration.
`tests/merge-gate.sh --only keys,pane-keys,wm-pane,acts`: all four PASS (acts 53 s, wm-pane 22 s, pane-keys 8 s, keys 17 s). No full gate.
Pushed claude/wm2 (751dfbcb..12305e5a). Commit hooks: no-inline-comments and commit-message-length (subject <= 68) each rejected one attempt; fixed, no --no-verify.

## PM check r2 (07:26 UTC; PM not the builder)
Diff 751dfbcb..12305e5a (6 files, +98 -13); red fa6ba541 before fix 12305e5a.
- rw-1: manage.rs:204 take() then sends(): a click after another look took the keyboard sends focus again; cell a_click_on_its_window_after_another_look_took_the_keyboard_sends_the_focus. Closed. NIT: a click on the window that still holds focus now resends it (idempotent).
- rw-2: main.rs:1017 to_node keeps the Focus record; cell a_free_keyboard_taken_by_the_page_shows_in_the_state. Closed.
- rw-3: place.rs:54 skips the window owner while layer_focus is set; cell the_key_place_is_not_the_window_owner_while_a_layer_has_the_keyboard. Closed.
- rw-4: doc merged. rw-5 NIT open -> backlog.
Verdict (computed): no open BLOCK or MAJOR -> MERGE-OK 12305e5a; train 59.
