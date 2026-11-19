# WM5 r1: kit::wm::stack and the scene orders (D89) with `held`

2026-11-19. Lane: cloud, opus, branch `claude/wm5` from 4548feb3 (train 59; the session branch had not moved). Head d14f5342.
Brief: briefs/wm5-r1.md.

## Verdict
Built; red first, then green. Merge gate once on d14f5342 (`--jobs 3`, run /tmp/merge-gate.p7daSF): every step
PASS (41 steps, "all steps passed"; tiling-audit, acts, matrix, virtual, mc-handover, e2e among them). acts: 76 pass,
8 known red, 0 blocking (known-red.toml unchanged). Independent review still to come (rules 7, 8).

## Commits (red first)
| commit | what |
|---|---|
| 3cae1523 | test: red cells. Scene: `Order` + prop `order` as a skeleton without sorting, 8 cells; core ipc: the lock order stays above every plugin; kit: `wm::stack` skeleton + 11 cells; mc: root in the background order; acts selftest I12: held window on top (5 cells) |
| 72920499 | feat(core): the root groups draw by their order; lock refused; `scene.boxes` names the order (+1 core boxes cell) |
| 5c4149a7 | feat(kit): `kit::wm::stack` (Rules: hold, held, modal, forget, order) |
| 780ab99c | test(tiling): the held window lies on top while it is dragged (red on 72920499) |
| 960a6552 | fix(tiling): relayout orders through `stack::Rules` (held = the dragged window) |
| 10505045 | refactor(mc): root in order background, `under_other_plugins` (D83) deleted |
| 263a272c | fix(acts): I12 checks the held window (`held: true` in a look's `windows`) |
| 59ea4adc | docs: api.md (order, boxes `order`), glossary (Order, Held window), architecture.md, decisions D89 extended, D83 retired |
| d14f5342 | chore(cut): copies row "root moved below the other roots by hand (D83)" max 0 |

## Red evidence (3cae1523)
- chora-scene: 6 of 9 FAILED, e.g. `the_core_draws_the_roots_by_order_whatever_the_order_of_creation` left ["bar","xp","drag","mc","tiling"] right ["mc","xp","tiling","drag","bar"]; `no_client_takes_the_lock_order`, `only_a_group_has_an_order`, `inside_an_order_the_client_orders_with_move`, `a_new_order_puts_the_group_on_top_of_its_new_order`, `a_surface_under_the_root_stays_in_the_normal_order` FAILED.
- core `ipc::tests::the_lock_order_stays_above_every_plugin`: FAILED (the create with order lock was accepted).
- kit `wm::stack`: 6 of 11 FAILED (held on top, modal above parent, modal of held, held modal, modal of modal, new parent).
- mc `the_root_of_mc_lies_in_the_background_order_below_every_other_root`: FAILED left Normal right Background.
- tiling `the_held_window_lies_on_top_while_it_is_dragged`: FAILED ["s1","s2"] (s2 raised later stayed on top).
- tests/test_acts.py I12: 2 FAILED (a window, a root of another look over the held window).
Green: chora 607, chora-scene 137, chora-kit 278, tiling 153, mc 141, xp 195, construct 198, assistant 238, notify 18 (all pass); test_acts.py 106 pass.

## Core lines changed (escape hatch named)
- crates/chora-scene/src/node.rs: `enum Order {Background, Normal, Held, Overlay, Lock}` (Ord), `Props::order` (default normal, left out when normal), `PropsUpdate::order`, `Props::apply`.
- crates/chora-scene/src/scene.rs:901 `link`: under `root` the index is clamped into the band of the node's order (`band` :919, `order_of` :939); `set_order` :944 puts a root group on top of its new order, called from `change_props` :750; `root_order` :999 for boxes.
- crates/chora-scene/src/scene/checks.rs:53: `order: lock` from a client is refused (Invalid) — the lock order belongs to the core, so no plugin root can draw above the lock; the session lock (ext-session-lock) and the safety layer stay drawn by the core above the whole scene as before (render.rs untouched). check_kinds: only a group takes `order`.
- crates/chora/src/ipc/boxes.rs:92: `order` of the root group in each scene box.
Name: the prop is `order` (D89 allows `layer` or `order`; "layer" names offscreen layers and layer shell already).

## kit::wm::stack and the ports
Rules { held, modals }: `order(look)` puts held on top, then each modal right above its parent where below it (modal of held goes with it; cycles end). 100 % covered.
- mc: D83 reorder gone; root created with order background (build.rs `root`).
- tiling: `relayout` orders via `stacked` (windows.rs) with held = `dragged`; tiling has no modal facts (the core sends no toplevel parent).
- xp: no change needed and none made: it orders by raise (`Move index None`), keeps no order list; its modal panels have no parent window. construct: depth order and `placed` (D84) stay its own; compiles unchanged.
- I12 invariant: `covers_held` (tests/acts/invariants.py): a window, or a node of another client in an order below held, over a window that its look reports `held: true`.

## Visible behaviour
Unchanged except: mc stays under every other root even when that root moves to index 0 (B5 residual rv6-1); a window that tiling drags lies on top of tiling's order while held. F29 (bar over fullscreen in tiling-audit) is not this cause: the bar is waybar, a layer-shell `top` surface the scene order does not touch; it stays a timing flake for the fullscreen act.
known-red.toml unchanged (no row turned green or red).

## Coverage
`cargo llvm-cov -p chora-kit --features chora-kit/testing` (without XDG_RUNTIME_DIR): kit::wm lines 98.9 % (stack 129/129, book 210/214, give 18/18, kept 36/37, state 44/44).

## Open, said plainly
- No look puts a window into the `held` order across looks yet: xp's and tiling's dragged windows stay in their normal roots, so a window dragged from xp over a later-built normal root of another look can still go under it. The band and the I12 rule exist; the move of the dragged group into a `<look>-held` root group needs the window transform made absolute (proposal 1).
- No look reports `held` in state.get yet, so the live I12 held rule has nothing to read; it is a selftest cell only.
- notify and assistant roots stay normal (visible behaviour unchanged); proposal 2.
- I12 "modal above its parent" at look level is only in tiling's port shape; xp has no window-parent facts.

## Refactor proposals, ranked
1. A kit helper `stack::hold_in(group, look)` plus xp/tiling ports: on a drag, move the window group into a root group `<look>-held` of order held with its absolute transform, back at the drop; reports `held: true` in state.get. Closes the cross-look half of I12. M.
2. chora-notify and chora-assistant bars/notices into order overlay; xp taskbar and start menu into an `xp-overlay` root. Ends build-order dependence of bars (I9, I12 bars). M.
3. Core: `surface.created` with the xdg parent of a toplevel, so `stack::Rules::modal` gets app dialogs in every look. S-M.
4. xp: replace raise-by-`Move index None` (wm/windows.rs:164, press.rs:60, panels.rs:77/482) by one kept order + `stack::Rules`, as tiling does; enables a copies row "Move index None outside kit". M.
5. chora-scene/src/scene.rs is 1250 lines; the root band (`band`, `order_of`, `set_order`, `root_order`) could move to scene/tree.rs next to link/move. S.
Cut check: no new breach (mc, tiling use only chora-kit and chora-scene).
