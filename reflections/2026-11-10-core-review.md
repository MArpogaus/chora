# Core review of 2026-11-10: the merged render and input core as one piece

Scope: `df6dc69..a03be22` in `crates/chora`, `crates/chora-scene`,
`crates/chora-kit` (9,604 inserted lines, 39 files), read at a03be22.
Read-only: nothing was built, run or changed. Every finding below comes
from reading the code and the Smithay revision cf43a02 that it runs on;
none is confirmed by a test. Each "Test" line names the red test that
would confirm it. Line numbers are those of a03be22.

Counts: 1 high, 6 medium, 9 low. **Fix first** marks what should be fixed
before any further core feature (K5, E1 to E3, virtual outputs).

What holds, checked and found sound:

- The lock screen. `surface_under` and `node_hit` answer only the lock
  surface (`render.rs:1036`, `:1085`); `lock_changed` ends the node, plane
  and button grabs and the pointer grab, and the drag and drop grab of
  Smithay cancels on `unset`, which clears `client_drag`
  (`lock.rs:226`, Smithay `dnd/grab.rs:464`). No popup grab, drag, act or
  layer keyboard starts while locked.
- `client_drag` cannot stick: every way the grab ends (`set_grab` of
  another grab, `unset_grab`, the release) calls `dropped` or `cancelled`.
- An injected press never starts a drag, never stamps a token, never
  sets `pressed`; an injected move or press cancels a running drag.
- Client-controlled numbers: transforms and meshes are finite, depth is
  at most 64, nodes at most 16,384; NaN and infinite results of products
  fall out at `plane_inverse`, `is_finite` and the clamps of `Bounds`. I
  found no panic on client input in the new code.
- The split is bounded (64 planes, 4 pieces per plane) and the draw and
  the hit test share one order.

## High

### H1. An interactive move of a window in a plane pins the pointer to it for good — Fix first

`input.rs:646` (early return), `:654-658` and `:673-680` (the counter),
`:750-781` (`start_move`), `:956-966` (`planar_press`).

`press_button` counts the buttons of a plane press in `plane_grab.held`
and ends the grab when the count reaches 0, but only on the path after
`grab_button`. `start_move` turns the press into a button grab `move`
and does not end the plane grab. The release then goes to `grab_button`,
which returns `true`, so `press_button` returns at `:646` before the
decrement. `plane_grab` stays with `held == 1`. Every later press makes
it 2 and every release 1, so it never ends.

Scenario: a window with a title bar of its own (GTK, Firefox) sits in a
rotated node or a 3D context, under a window manager that grabbed `move`
(tiling, xp). The person drags the title bar. After the release,
`move_pointer` takes `planar_press` first (`:586`): all motion, clicks
and scrolling go to that one window, mapped through its plane, wherever
the pointer is. Clicks on other windows and on panels reach the pinned
window instead; `click_focus` still moves the keyboard by the real
surface, so pointer and keyboard disagree. Only a lock, the emergency
reset, a popup grab or a client drag ends it. This is the path K5 (own
windows as panes) uses for every pane.

Fix: end `plane_grab` in `start_move` (and wherever `button_grab` is
set), and stop counting buttons: after each button event end the plane
grab when `pointer.current_pressed()` is empty. Better, see M6.

Test: `planes.rs`, window under `{"rotate": 40}`, wm with `input.grab
move`; press, `xdg_toplevel.move` with the press serial, release, move
the pointer onto a second window; assert that the second window gets
the enter and `plane_grab` is `None`.

## Medium

### M1. A plane larger than 4096 texture pixels shows cut and stretched, so clicks land beside what is shown — Fix first (before K5)

`mesh.rs:436-441` (`pixel_size` clamps each side to 4096),
`render.rs:627-637` (`node_texture`), `:644-660` (`layer_texture` renders
at `scale` into the clamped size), `:594-606` (`texture_scale` never
goes below the step).

The texture of a plane is clamped to 4096 pixels a side, but its content
still renders at the full scale. A window of 2560 logical pixels on a 5K
output at scale 2 (5120 pixels), or a 5120 wide window at scale 1, shows
only its first 4096 pixels, stretched over the whole quad. The hit test
(`plane_hit`, `render.rs:1290`) maps the pointer with the true size. So
the person sees one widget under the pointer and clicks another, up to
20 % of the width away, and the right fifth of the window is never
visible but takes input. The clamp is older than this range (2D layers
had it), but windows became planes today, and a window is the one kind
of content that is routinely this large.

Fix: when a side exceeds 4096, lower the render scale of the texture so
that the whole content fits (`scale * 4096 / side`), and pass that scale
to `layer_texture`, the tracker and the passes.

Test: a 5000 by 100 window under a rotation at scale 1; capture; the
pixel at the projection of (4900, 50) shows the color the window drew
there.

### M2. Input on what the texture budget left out: four gaps and two false blocks

`render.rs:1262-1269` (`left_out`), `:1140-1186` (`hit_cut`),
`:1304-1307` (mesh planes), `:709-712` (node textures), `mesh.rs:234`.

`plane_hit` and `context_hit` block the pointer on a plane that the last
frame left out. The rule has holes at the seams:

1. A node with a mesh under a plane returns before the check
   (`render.rs:1304`): a left-out mesh plane takes input.
2. A layer with a plain transform (rounded rect, clip, effect group, a
   3D context that is itself a layer) goes through `hit_cut`, which has
   no check: its windows take clicks while nothing of the layer shows.
   Older than this range, but the new rule makes the two paths differ.
3. The keyboard stays with a window whose plane was left out; the person
   types into a window that shows nothing.
4. A press that started before the plane was left out keeps its pin
   (`planar_press`), as a 2D click grab does; noted only.
5. False block: `node_textures` spends the budget under the id of the
   source node (`render.rs:709`). When the mirror of a window does not
   fit, the id of the window lands in `left_out`, and the window itself,
   which drew, takes no input.
6. False block: `left_out` asks all outputs (`.values().any`); a plane
   left out on one output takes no input on another that shows it.

The budget is one number per frame for all owners, front first
(`mesh.rs:234`), so one plugin with large layers in front pushes the
layers of every other owner out.

Fix: record per output and per frame the set of textured nodes that
drew (or failed), keyed by the node that draws, not the source; consult
it in one place (`hit_cut`, before the kind match) for every layer,
plane and piece. Decide and document rule 3 (move the focus away, or
tell the owner with `layers.dropped`, which exists).

### M3. A drag and drop can start from a key press with no button down

`handlers.rs:334-345` (`drag_allowed`), `:462-490` (popup `grab`),
`state.rs:802` (`track_press` runs for keys too).

`drag_allowed` asks for `last_real_press == serial`, and `last_real_press`
is any real key or button press. Smithay asks only that the pointer is
grabbed with that serial. A popup grab may start from a key serial
(`pressed` accepts `last_input`, `handlers.rs:510`), and its start data
names the root window with button 0. So an unconfined client can, on any
real key press it receives: open a popup with a grab on that serial,
then `start_drag` with the same serial. The drag runs with no button
down. Until the next click the pointer enters no window (confined
clients are filtered, the others get drag enter), and the release of
that next click drops the data on whatever unconfined window is under
it, for example text onto a terminal. Each window the pointer crosses
can already read the offer. The client is trusted under D33, so this is
defence in depth, but it is exactly "data moves without a real act".

Fix: in `drag_allowed` require a held button of a real press: keep
`last_real_button: Option<Serial>` beside `last_real_press`, compare
with it, and refuse when `start_data.button == 0` or
`pointer.current_pressed()` is empty.

Test: `popups.rs`; real key into the window, popup grab with the key
serial, `start_drag` with it; assert `!client_drag`.

### M4. The act of a token is bound to nobody, and any click on any node that is no window is an act

`state.rs:812-823` (`note_act`), `input.rs:857-859` (`press_node`),
`input.rs:138`, `:142-145`, `:718-720`, `ipc.rs:913-932`,
`protocols.rs:367-379`.

The decision (h4c, rule 2) said "one act, one stamped token". The code
stamps every token that any unconfined connection mints within 2 s of
the act, and marks the act used only when a layer takes the keyboard.
An act is: any grabbed key or button (also one that a confined client
grabbed, for example push-to-talk), any key to a focused node, and any
real click on any rect, image or shader node, also a wallpaper or a
title bar drawn by a window manager. So after such a click, for 2 s any
main-socket process may mint a stamped token, and for 10 s, as long as
the person presses nothing, any unnamed client with an on-demand layer
surface that holds it takes the keyboard. The person did not ask for a
keyboard change with that click. No path in the tree lets a confined
client get such a token today (`choractl mcp` mints on its confined
connection; `methods.call` now carries `confined`, which no plugin
reads yet). The rule therefore holds only by the discipline of every
unconfined minter, not by the core, which is what principle 11 and
#102 asked the core to promise.

Fix: record with the act who received it (the connection of the grab,
the owner of the node) and stamp only a mint of that client name; let
`chora_kit::run` mint on the connection of the plugin and hand the token
to `choractl run`. Count a press on a node as an act only when its owner
subscribed to that node's presses, or let the owner claim the act. Make
plugins refuse `MethodCalled { confined: true }` for anything that
mints, in the kit runtime, not per plugin.

### M5. Step 5 moved 2D layers onto the cached path, outside the plan, with no guard that could see a stale frame

`mesh.rs:340-365` (`draw` with the kept tracker), `:370-400` (`next`),
`:946-970` (`apply_passes` reuse), `:178-198` (`evict_over_cap`),
`:146-152` (keys without the output).

Section 3 of `design/3d.md` says per plane damage applies only to
planes and 2D layers get it later with their own move of `base_2d`.
The code caches every layer: a rounded rect, a clip group and an effect
now render only on damage and reuse the result of their passes. That
is a gain, but it is a 2D path change that the 2D guard cannot judge:
it compares single frames of static scenes, and the failure of a cache
is a stale frame in a sequence. Weak points I can name:

- Versions count per slot from 0. When the cap evicts slot 0 of a node
  and keeps its pass slots (slot 0 is touched first, so it is the
  older), the new texture gets version 1 again; `signature` and
  `Seen.version` then match the old ones, and GL may reuse the texture
  name. A changed content can show the old pass result, with no damage.
  Needs more than 512 MiB of layers, so rare, but silent.
- Eviction takes "least recently used" among textures that the last
  frame used; over the cap, the same textures are freed and allocated
  every frame, each time with a full render.
- `steps`, `passed` and `ids` are keyed by node, not by output. Two
  outputs with different scales or cameras flip the step, rerun the
  passes and bump the commit on every frame, so "a static context
  renders no pass" fails on two monitors.

Fix: one counter for versions in `Layers`, never reset; evict only
textures the current and the last frame did not use, else refuse the
layer (it lands in `left_out`); key `steps`, `passed` and `ids` by
output or by texture size. Add guard scenes that run sequences: change
a child of a rounded rect, of a clip group and of an effect layer and
compare frame N with a fresh render.

### M6. Design: five pointer states with hand-written precedence — Fix first, with H1

`state.rs:238-246`, `input.rs:545-606`.

`node_grab`, `button_grab`, `plane_grab`, `client_drag` and the grab of
Smithay all say who owns the pointer. `move_pointer` orders them by an
if-chain, `relative_motion` (`input.rs:438-449`) ignores two of them,
and every place that ends one must remember the others: `lock_changed`,
`reset`, `start_move`, the popup `grab`, `dnd_requested`. H1 is one
forgotten place; step 4 already had to patch two. The next feature that
takes the pointer (E3, the grab transfer) will add a sixth.

Fix: make the plane press a `PointerGrab` of Smithay that replaces the
click grab at the press (it owns the mapping and ends in `unset`, as the
drag grab does), and fold `node_grab` and `button_grab` into one enum
`PointerMode`. Then one `unset_grab` ends everything and the lock, the
reset and the move need no list.

## Low

- **L1. Confined clients cannot drag at all, and no drag shows an icon.**
  `handlers.rs:349-360`, `:304`. A Flatpak browser cannot reorder its
  own tabs; a person drags with no feedback (principle 4). Allow a drag
  whose target is the client of the source; draw the icon surface at the
  pointer. Whether confined clients take part belongs in a grant, not in
  a constant of the core.
- **L2. A drag from an open menu leaves the menu without a grab.**
  `handlers.rs:324-326`. `set_grab` ends the popup pointer grab and the
  keyboard grab, but the popup stays mapped and no click outside closes
  it. Dismiss the popup grab (`ungrab(All)`) when a drag starts.
- **L3. The order of a context costs on the input path.**
  `space.rs:81-101`, `:334-343`, `render.rs:1200-1205`. Each hit test
  builds the context list and a fingerprint of 19 words per plane before
  it finds the kept order; a pointer motion does it at least twice
  (`hover`, `surface_under`). `sort` runs the whole BSP sort after each
  added piece. `MAX_SPLIT` is 64 where the plan said 1,024. Key the kept
  order by a scene generation, and check the piece limit once at the end
  or every 16 pieces.
- **L4. A collapsed plane reaches the splitter with a NaN normal.**
  `space.rs:304-313`. A node with scale (1, 0) in a context passes the
  edge test of `Polygon::from_points`, whose normal is then 0/0. I found
  no panic in plane-split 0.18 on that path by reading, only a wrong
  order. Refuse a polygon whose normal is not finite. Not verified by a
  run.
- **L5. Piece ids follow the place in the order, not the plane.**
  `render.rs:406`, `mesh.rs:146`. A piece gets the index of the whole
  order as its id, so a reorder gives old pieces new ids (full damage)
  and `ids` keeps every index a node ever had until the node goes: up
  to planes times pieces entries. Number the pieces per plane and prune
  ids that a frame did not use.
- **L6. Image steps cost a third more memory for every image.**
  `images.rs:398-403`, `:245-262`. The halvings count toward the same
  512 MiB, so a quarter fewer images fit than before, also images that
  never draw scaled. Build the steps at the first scaled draw, or raise
  nothing and say so in the docs.
- **L7. 2D changes that the guard does not cover.** `node.rs:251-262`: a
  node with a size and a negative or zero own scale is now a layer and
  draws mirrored (wanted, #48), which changes old scenes.
  `images.rs:216-250`: every scaled image samples a halving and snaps
  its edges. `handlers.rs:304`: drag and drop now runs in 2D scenes too.
  The 2D guard has no image, no mirrored node and no drag. Add them.
- **L8. `depth` and `facing` overwrite uniforms of the same name.**
  `render.rs:439-456`. A shader that already has a uniform `depth` gets
  another value inside a context, silently. Reserve the names in
  `check_shapes` or document them as reserved.
- **L9. Kit and plugin.** `chat.rs:225-268`: 500 entries are kept, but
  the text of one entry has no limit and wraps whole before the cut to
  40 lines; cap the bytes of an entry at `apply`. Tiling
  `main.rs:1329-1333`: after `gave`, tiling sends no focus until one of
  its windows takes it; when the receiver never focuses the window, the
  remaining windows wait for a click or a binding. Clear `lost` when the
  next `focus.changed` names no surface of another owner within a
  bounded time.

## Design notes

- The core grew from 25.4 to 31.8 thousand lines in a day. The split
  into `space.rs` and the scene walk is clean. `render.rs` now holds the
  draw, the hit test, the plane math and the window boxes in 1,700
  lines; the hit test wants its own module with one entry point, which
  would also give M2 its one place.
- Policy that went into the core: who takes part in drags (L1), what
  counts as an act (M4), and the keyboard rule of on-demand layers. The
  first belongs in grants; the other two are core promises and should
  be exact rather than broad.
- `Orders` needs a `Mutex` and a `Cell` inside `Assets` because the hit
  test has `&State`. That is a sign that the kept order belongs to the
  scene generation, not to the render assets.
- Not checked here: the HUD host and the chat field against principle 5
  beyond the core seams (the confirm keys are key grabs, so a layer that
  takes the keyboard cannot take them; a layer surface of the top layer
  still draws above `screen:NAME` and so above the question card, by
  D43, which is older than this range).
