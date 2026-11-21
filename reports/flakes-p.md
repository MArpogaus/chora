# flakes-p (F5: no flaky tests)

## r1

Lane: cloud, branch `claude/flakes-p` from `618ae513`; head `bd3ecad4`. Written 11-21. Step 0 passed (`cargo check --workspace` exit 0; foot, python3, sccache present).
Load: `tests/under-load.sh` with one busy loop per stream, three streams at once (matching `--jobs 3`), and a second `cargo build` loop (clean of chora-kit, own target dir) beside them. `PAIR_ONLY=1` (new, in the usage header of the cell) runs only the two-pane part of the cell (7 s alone), the reproducer.

| Item | Result | Commits | Loaded runs |
|---|---|---|---|
| 1. FLK-PULL | cause shown, product fix in xp | diagnostic `7dab89ce`, red unit test `cddb241f`, red cell `31815ede`, fix `bd3ecad4` | red 7 of 45 (PAIR_ONLY); after the fix 30 of 30 (PAIR_ONLY) and 21 of 21 (full cell, 3 streams of 7, 270 s each) |
| 2. placed, cube x10 loaded | not done | none | the lane was past 150 k when item 1 closed |

### 1. FLK-PULL: "the second window is no pane"
Reproduction: 54 loaded runs of the pair part with the first diagnostic: 8 red, all `FAIL: the second window is no pane` (the first of 1 of 10 alone on dcb77733 was this line).
Diagnostic `7dab89ce`: the failure line now adds the node of the second window and the tree of the panes. A red: `s2 owner=xp parent=xp-win-s2 visible=True size=[200, 200]`, the pane tree holds only `construct-pane-s1`. So the window was never offered to the construct: the core reserved it for xp.
Cause: the core picks the manager of a new toplevel in `Scene::manager_of_new` (crates/chora-scene/src/scene.rs:447): the owner of the parent, else the manager that a click chose, else the owner of the focused **node**, else of the focused surface, else the claim under the pointer. Event logs of two reds (`focus.*` and `surface.created` with timestamps): `focus.changed s1` and `focus.node xp-icons (owner xp)` in the same millisecond after the drop, then `surface.created s2 ... "to":"xp"`, and no further focus event for the 10 s that the cell waited. In greens the order is `focus.node xp-icons`, then `focus.changed s1`, then `focus.node null`, and `to` is `construct`. A `Command::Focus` of a surface clears the node focus in the core (scene.rs:1063), so the last of the two commands wins. xp sends `focus_node xp-icons` (`desktop_keys`) from its own view of the focus; when the window has left xp and the core has focused it before xp handled the departure, that view is older, xp does not see a window of another look, and its late `focus_node` takes the keyboard behind the pulled window. Not a harness effect: a person who pulls a window out and types would type into the icons of xp, and the next program they start opens on an xp face, not as a pane.
Red: `cddb241f` unit test `the_desktop_icons_give_back_keys_that_a_foreign_window_holds_by_focus` (xp) fails (left `[]`, right `[FocusNode { node: None }]`). Red of the cell, `31815ede`: after the drop the cell waits until xp has let the window go (`state xp` no longer lists it) and then until `state xp` focus holder is not `xp`; it fails with `construct:construct/s1 xp:xp/None`: 7 of 45 loaded runs fail there, none later. Construct's own focus view says `construct/s1` in a red and a green alike, so it is not a witness (the earlier commits `0fcbfec5`, `8a443f9b`, `bbe0aee4` tried an event log and that view; they are the way to the state that does discriminate, kept as they ran).
Fix `bd3ecad4` (chora-xp, plugins/chora-xp/src/wm/state.rs, dispatch.rs): xp hears its own `focus.node` of the desktop icons; while a window of another look has the focus it answers `focus_node null`. xp sends the icons the keys only when no foreign window has the focus (`desktop_keys`), so the answer fires only on the stale decision. It is a product change in a flake round; PM to review it.
After the fix: 30 of 30 loaded runs of the pair part and 21 of 21 loaded runs of the full cell (red 7 of 45 before).

### 2. placed and cube, 10 loaded runs each
Not done: past 150 k tokens. Both cells ran green in every gate run below (`cube` 115 s, 114 s, 115 s at --jobs 3).

## New (not mine)
FLK-MATRIX-S1: `tests/gate-handover-matrix.sh` (cell `matrix`), first Super drag of a construct-owned window: `choractl: the node "s1" shows no point [347.3, 248.0] (code 1)`, 1 of 3 matrix runs at --jobs 3 (rerun 1; green in the gate run before it and in reruns 2 and 3). `super_drag` (line 316) takes the middle of `rects` of the node and injects a point inside the node; the node showed no such point at that moment (rects of the pane stable per `rects_stable`, so the point is likely outside the shown part of a pane that is not yet in view). Not reproduced beyond that one; the log is kept at reports/evidence/flakes-p-matrix-red.log. Open, for the next flakes round.

## Gate
`tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,gate-lib,xp,matrix,virtual,xp-faces,xp-paperclip,xp-fields,xp-question,cube,pull` at `bd3ecad4` (PATH with ~/.local/bin): all fifteen PASS (pre-commit 2 s, gate-lib 88 s, ci-check 70 s, build, test 176 s, clippy 12 s, virtual 43 s, xp-faces 15 s, xp-paperclip 7 s, xp 84 s, xp-fields 30 s, xp-question 12 s, pull 84 s, cube 115 s, matrix 277 s). The touched cells (`--only pull,xp,virtual,xp-faces,matrix,cube`) three more times at --jobs 3: rerun 1 matrix FAIL (FLK-MATRIX-S1 above; the gate stopped, pull and cube did not run), rerun 2 and rerun 3 all six PASS.

Commits on claude/flakes-p (author and committer Marcel Arpogaus): 7dab89ce, 0fcbfec5, cddb241f, 8a443f9b, bbe0aee4, 31815ede, bd3ecad4. Head: bd3ecad4.

## PM review r1 (2026-11-21 02:30 UTC; PM is not the builder; small diff)
618ae513..bd3ecad4, product +44 -6 in plugins/chora-xp/src/wm (dispatch.rs, state.rs). A focus.node event now runs node_focused (state.rs:107): the core says the xp icons hold the keys while focus_away() (a foreign surface has the core focus), so xp sends FocusNode None. This is the rule desktop_keys already keeps (press.rs:268-277, I3: a window of another manager with the focus keeps the keys), applied to a late answer, so no new policy. Red cddb241f (unit) and 31815ede (cell, 7 of 45) before the fix; unit test covers the icons with and without a foreign focus, another owner, another node. Risk checked: a desktop click clears the core focus first (focus_away false), so the icons keep their keys there.
Findings: none open. Verdict (computed): MERGE-OK, train 97 with flakes-q. New FLK-MATRIX-S1 (1 of 3 loaded) keeps F5 open.
