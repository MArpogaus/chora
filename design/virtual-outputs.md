# Virtual outputs (D69)

Design of 2026-11-10, read on the code under `/home/user/chora` (`crates/chora`,
`crates/chora-scene`). File references are `path:line`. Nothing here is built yet.
It serves K7, K8 and K11, and the cube of `3d.md` and `wobbly-3d.md`.

## 0. What exists and carries the design

- The headless backend already renders an output into a `GlesTexture` with an
  `OutputDamageTracker` (`backend/headless.rs:60-66`, `:302-346`). That is a virtual output
  without an owner and without a reader.
- A shader node reads another node as a texture through a uniform `{"node": ID}`
  (`chora-scene/src/node.rs:137-155`, `render.rs:692-735`).
- All window managers share one scene. A window manager with `--output NAME` reads the work
  area of its output in scene coordinates and sets its claim there. `chora-xp` takes several
  `--output` and handles `output.removed` (`plugins/chora-xp/src/main.rs:75-78`,
  `wm.rs:1240`).
- The pointer becomes a `Spot {output, local, scene}` (`output.rs:320-332`); the hit test,
  the hover, the node press, button bindings, `manager_of_new` and `hand_over` all start from
  it (15 callers).
- `PlaneGrab` and its `Pin` map every motion of a press through a plane with `anchor` and
  `local_in`, without the inside test (`input.rs:882-1002`, `render.rs:1388`).
- D70 `forward` and the capability `node.portal` are decided and not built.

## 1. What a virtual output is

**To a window manager:** a normal output. It is in `output.list` with `virtual: true` and
`owner`, has a `wl_output` global, a mode, a scale, a camera, a work area, an overlay root
`screen:NAME` and layer surfaces. `chora-xp --output VIRTUAL-1` needs no change.

**To the core:** an output that no backend shows and no pointer enters. It lives in a new list
`State::virtuals`, not in `State::outputs`. Its frame goes into a texture. A shader node of
its owner reads that texture with the uniform `{"output": NAME}`, and with `forward: true`
the node passes pointer input into the output.

**The smallest core change**, in one sentence: move the "render an output into a texture" of
the headless backend into a backend-neutral `virtual.rs` with its own list, let a shader node
of the owner sample it with `{"output": NAME}`, and let `State::spot` continue through a node
with `forward` into that output.

Details:

- **A list of its own, fail closed.** `State::outputs` has about 70 uses; most mean "a screen":
  pointer layout, lock, `wlr-output-management`, capture lists, inject defaults, "first
  output". A flag on one list leaks at every forgotten filter. With a second list, a forgotten
  site only lacks a feature on a virtual output, which the xp test shows. One helper
  `State::all_outputs()` serves the sites that must include them: `output_info`,
  `output_changed` (cameras, overlay roots), layer shell (`layers.rs:74-208`), enter and leave
  (`protocols.rs:184-197`, `transaction.rs:455`), `scene_area_at` for popups, foreign toplevel.
- **Scene area.** The scene is one; a virtual output needs a region no screen shows by
  default. Slot `i` (0 to 15) sits at layout position `(-(i + 1) * 262144, -262144)`. Real
  outputs fill at most 0 to 262144 in x. The gap between slots is at least 245760 pixels.
  The default camera follows the position, as for every output (`output.rs:636`).
- **API.** `output.add {spec, virtual: true}` on every backend returns the output; the core
  names it `VIRTUAL-n`. Without `virtual`, the headless hotplug of today stays as it is, so no
  test changes. `output.remove` of a virtual output works only for its owner and on the admin
  socket. New in the scene crate: `Uniform::Output {output}`, the node property `forward`.
  No new node kind, no new event.
- **Amendments to D69 and D70**, to write when the step that builds them merges (judge
  finding 18): the flag `virtual`; `forward` into an own virtual output needs no `node.portal`,
  which stays for portals onto real outputs (K6); the pixel limits of section 6.
- **Budget.** About 450 new core lines, about 120 of them moved from `headless.rs`. All of it
  in `virtual.rs` and `input.rs`; `ipc.rs` (4,225 lines) gets one dispatch line.

Out of scope: output textures of real outputs and the desktop pane (K6), nested portals (a
pane inside a virtual output that forwards again), the grab transfer E3, a resize of a
virtual output (remove and add instead).

## 2. Input

One rule: **the pointer is where the pane says it is.** `State::spot(layout)` becomes:

1. the screen spot of today (`screen_spot`, the old code);
2. if the session is unlocked and the node hit at that spot (`render::node_hit`, which
   already goes through planes and 3D contexts) is a shader node with `forward` and an output
   uniform: `uv = local / size of the node`, `local' = uv * logical size of the output`,
   `scene' = to_scene(camera(V), local')`, and the result is
   `Spot {output: V, local', scene', via: Some(Portal {pane, output})}`.

One level only: a hit inside V does not forward again. Everything after the spot runs
unchanged on the virtual output: layer surfaces and the overlay root of V, the scene hit
test, hover, `pointer.pressed` with `local`, `input.button` with `output` and `position`,
shared button bindings through `claimant_at`, and `MotionEvent.location`, which is the scene
point of the spot (`input.rs:600`). Callers that need the screen keep `screen_spot`: the
cursor, the clamp, `inject` targets.

| Topic | Answer |
|---|---|
| Navigate mode | The construct sets `forward: false` (or holds its grabs). Grabs and the safety layer run before the spot is used, as D70 demands. |
| Focus | Unchanged. The window manager on V gets `pointer.pressed` for its window and calls `focus`. There is one keyboard focus in the session, as with several window managers today (D55). |
| Keyboard | Unchanged. Keys go to the focused surface on whatever output; key bindings are global. |
| Press and drag | `State::press_via` holds the portal of the first button press (or "screen") until the last release. While it is set, `spot` maps through that pane with `local_in`, without the inside test, so a drag that leaves the pane keeps sending points. `uv` clamps to -1 to 2, so an extrapolated point stays inside the gap of the slot. This one rule serves the three grabs: `PlaneGrab`, `NodeGrab` (a title bar of xp) and `ButtonGrab` (`Super+BTN_LEFT`). |
| Tilted panes | `Pin::under` pins when the surface is under a plane **or** the spot came through a portal. `anchor` (`input.rs:882`) gets one branch: for a pinned press with `via`, the start is the root and the point is the portal point. The `local_in` of today then walks the planes inside V too (a wobbling xp window). |
| Release | The release is **not** pinned: `press_via` ends before the release event is built. A release over the same pane has the same coordinates. A release elsewhere carries the point under the real pointer: in another pane the point in that output, in open space the screen scene point. So `claimant_at` of the dragging window manager finds the receiver, with the code of today (section 5). Wayland clients are not affected: `wl_pointer.button` has no position. |
| Popups | The popup area is the work area of V, through `scene_area_at` on `all_outputs`. A popup is part of its window, so it draws in the texture of V. `handlers.rs:493` already wraps every popup grab in `PlaneGrab::popup`; a press in it pins as above. |
| Client drag and drop | The drag grab of Smithay takes the focus that `move_pointer` passes with each motion, computed from the spot, so it crosses windows on V. It closes the carried finding of `3d.md` ("a DnD grab never sees the other surface") for portals. |
| Act stamps (D72) | `note_act` runs where it runs today: a grabbed key or button, a press on a node that is no window. A press on the start menu of xp through a pane is such a press, so its token names xp. Without a token, the pointer rule uses the spot: the window manager of the face under the pointer gets the new window (`handlers.rs:196-205` unchanged). |
| Injected input | Takes the same path with `injected: true`; it never stamps an act and never sets `pressed`, as today. |
| Lock | A locked session forwards nothing (`spot` step 2) and renders no virtual output. A virtual output has no lock surface and does not count for "locked on all outputs". |
| Cursor | Drawn on the screen at the pointer, at screen scale. Accepted. |

Why no capability: only the owner can sample its output, and the owner is an unconfined
client of the main socket, which can already put any window under the pointer. `forward`
gives it no new power.

## 3. Frame timing, damage and cost

- **No clock of its own.** Before the core builds the elements of a real output, it calls
  `render_virtuals`: every virtual output that a **visible** shader node samples, and that was
  not rendered in this frame interval, renders with its damage tracker at age 1. No damage
  means no GL work and the texture stays. The pane then samples a texture of the same frame:
  no added frame of latency.
- **Damage.** The texture has a version, as node textures have (`render.rs:727-733`). The pane
  element commits only when the version moves, so the screen redraws only the projected
  rectangle of that pane. A commit of a window on V requests a frame of the outputs that show
  a pane of V, by the path that a node texture source uses today.
- **Frame callbacks.** After a render with a reader, `send_frames(V)` and the presentation
  feedback run with the refresh of the fastest real output, which is also the mode refresh
  of V. Without a visible reader the windows on V are hidden windows: one callback per second
  (`render.rs:1479`). This answers 0.10 gate 9, third check: a shown pane never freezes.
- **The cube.** The geometry sets `visible: false` on faces that look away (D19 leaves back
  faces to the plugin). At most two faces render, one after a fly-in at 1:1. A static cube
  renders nothing. Memory: four faces of 1920x1080 are 33 MiB.
- **1:1.** A pane at z = 0 that faces the eye takes the 2D path (D75) and is pixel exact. It
  costs one copy of the damaged area; there is no direct scanout to lose.
- **Minified faces.** They render at full size and are sampled down; quality depends on the
  plane mipmaps (backlog R2). The creator may pick a smaller mode; nothing adapts by itself.
- **Limits of the tests.** All numbers come from llvmpipe. `tests/3d-bench.sh` gets a cube
  scene for the reference hardware (risk 1 of `3d.md` is still open: function is not 60 fps).

## 4. Lifetime

**Create:** `output.add {spec, virtual: true}`. The creator owns it.

**Destroy:** `output.remove` by the owner or the admin socket; the end of the owner after its
grace time (`ipc.rs:65`; a `resume` within it keeps the outputs); the emergency reset.

**What the removal does**, in order:

1. Layer surfaces of V close and windows leave it (`leave_layout`, `output.rs:123`), and
   `output.removed` goes out. A window manager with another output moves its windows, as xp
   does at an unplug.
2. The core notes the surfaces that V showed and no other output shows (`shown_surfaces`,
   `render.rs:1505`).
3. After `RESERVATION` (5 s, `state.rs:153`), every noted surface that still no output shows
   and that still has the same owner is taken from that owner: the core runs the `remove` of
   the owner on its behalf and then `hand_over(surface, giver, except: owner)`
   (`state.rs:611`). The order of D55 holds: the giver, the window manager under the pointer,
   any live claim holder, and at last `place_free` on the first real output.
4. A pending `press_via` of V ends; the samplers of V draw transparent.

**Other ends:**

- The window manager on V ends, V stays: `execute_gone` of today hands every window over.
  The giver comes first, which is the parent fallback of K7.
- Creator and window manager end together (the construct and its children): both paths run;
  step 3 finds nothing left.
- Emergency reset: virtual outputs go first, without the wait; then `scene.reset` puts every
  surface in the grid of the first real output (`safety.rs:306`), as D69 says.

**Invariant, checked by a test over random sequences:** the number of surface nodes never
drops, and after 5 s every surface is attached by a live client or shown by the core on a
real output.

## 5. Window handover

The mechanism does not change: `remove` with `to` between claim holders (D55). A claim in the
region of a virtual output is a claim like any other, so every path of `window-handover.md`
works by name: `window.give`, `choractl give`, the key `give: next`, the MCP tool, voice.
`give_target` sorts claims by x; virtual claims have negative x and come first.

| Move | How |
|---|---|
| real to virtual, by drag | The giver drags on the screen; the release is over a forwarding pane, so its point is in V; `claimant_at` names the window manager of that face. |
| virtual to real, by drag | The drag is pinned to its pane; the release is unpinned and carries the screen scene point; `claimant_at` names the window manager there. "Pull a window out of the cube" is this, with the construct as the receiver. |
| virtual to virtual | The same; the release is over the other pane. Faces of one xp are outputs of one window manager: xp moves the window itself, no handover. |
| `meta.give.at` (handover step 9) | The release point, already in the coordinates of the receiver. |
| An ended window manager | Section 4. |

The handover during a held button (E3) is not part of this design. Until the user accepts
that core power, a window leaves a face at the release.

The guard `tests/gate-handover.sh` gets two more managers, `xp@V1` and `tiling@V2`, shown by
a pane host of the test. The manual cells "WM panes and drop to give" become table lines.

## 6. Security

- **Who.** A named client on the main socket, not `confined`. A client socket and a confined
  connection get code 9, as for `grants.set`. No new capability and no new grant.
- **Count and size.** At most 16 virtual outputs in the session (D69). A side has 1 to 4096
  pixels (`mesh.rs:43`). All virtual outputs together hold at most 2^26 pixels, which is
  256 MiB of RGBA and about eight 4K outputs. The core checks all three before it allocates.
- **Name.** The core chooses `VIRTUAL-n`; a client cannot take the name of a screen.
- **Reader.** Only the owner sets `{"output": NAME}` and `forward` on a node; another client
  gets an error that names the owner. The scene keeps `name -> owner`, set by the core.
- **Not listed:** pointer layout, `wlr-output-management`, kanshi, the lock, screencopy and
  image capture lists, the portal. `screen.capture` by name works with its grant.
- **Lock.** No render and no forwarding while locked.
- **Review.** Steps 3 and 4 touch `input.rs`: a second review with fresh context on the
  input and act part (judge findings 22 and 23).

## 7. Steps

Each step is one branch with its tests written first and seen red, then the 2D guard and the
merge gate. Steps 1 to 5 are core (strongest model); 6 to 9 are plugins and scripts.

| # | Content | Tests that fail first | Exit criterion | Risk | KISS alternative |
|---|---|---|---|---|---|
| 1 | `virtual.rs`: the render into a texture moves out of `headless.rs`; the headless backend calls it. No behavior change | `render_into_draws_an_output_outside_the_backend_list` | headless tests and `2d-guard.sh` unchanged | the EGL flake of backlog F2 hides a regression | leave `headless.rs` alone and copy the 45 lines |
| 2 | `output.add {virtual}` and `output.remove`: list, owner, slot, name, limits, `wl_output`, camera, overlay root, `output.list` | `a_virtual_output_is_in_no_list_of_screens` (pointer, management heads, lock, capture); `the_pointer_never_enters_one`; `a_client_socket_cannot_add_one`; `the_17th_and_the_pixel_budget_fail`; `only_the_owner_removes_it` | `chora-xp --output VIRTUAL-1` starts on headless and on winit and attaches a probe window | a forgotten `all_outputs` site (layer shell, enter) | xp without bars first: skip layer shell on virtual outputs |
| 3 | The reader: `{"output": NAME}`, `render_virtuals` on demand, version, frame callbacks | capture: a flat pane shows the color of a probe window on V; `a_static_virtual_output_renders_once` in 60 frames (`RenderStats`); `an_unread_one_renders_nothing_and_gets_1_hz`; `a_commit_damages_only_its_pane`; `a_foreign_client_cannot_sample_it` | xp on V shows in a flat pane; 0 passes when static | tty: the texture of the primary GPU on an output of a second GPU | render on every frame of the first output (the headless clock) and keep only the damage check |
| 4 | Flat input: `forward`, the spot through one portal, lock | a click on a scaled flat pane reaches the probe at the mapped point; hover; `input.button` carries V; `forward: false` gives the press to the owner; locked forwards nothing; a new toplevel under a pane goes to the manager on V; a node press through a pane stamps an act, an injected one does not; `hits_match_the_flat_walk` stays green | a foot window on V takes typed text after one click, 20 of 20 | a caller of `spot` that needed the screen | forward only for a pane with a plain transform (fly-in only) and ship K7 with that |
| 5 | Tilted input, drags, popups: `press_via`, `Pin` with `via`, the `anchor` branch, the uv clamp, the unpinned release | in a context turned 40 degrees: a click types; a drag that leaves the pane keeps sending points; a grabbing popup opens, takes a click, and ends on a click outside; a node drag stays pinned; the release off the pane reports the screen point; client drag and drop between two windows on V | the tests of 3D step 4 pass through a portal | the unpinned release surprises a plugin that expects the last move point | pin the release too and add `claimant_at` without a point ("under the pointer") to the API |
| 6 | Lifetime: end of the owner after the grace time, steps 1 to 4 of section 4, the reset | `the_outputs_go_after_the_grace_time` and `resume_keeps_them`; `a_stranded_window_lands_on_a_screen_after_5_s`; `a_killed_manager_returns_its_windows_to_the_giver`; `the_reset_removes_virtual_outputs_first`; the invariant over 200 random sequences | 0.10 gate 9, checks 1 and 3 | the wait of 5 s races a slow window manager that was about to move the window | no wait: refuse `output.remove` while a foreign claim lies on V, and drop such claims only at the end of the owner |
| 7 | Guard: `gate-handover.sh` with a pane host, `xp@V1` and `tiling@V2`; cells for give, `choractl give`, key and drag in both directions | the new table lines, first as `missing`, then flipped | all new cells `pass`, the gate under 4 minutes | drag cells flake on injected timing | drags as a manual cell; the other three paths gate |
| 8 | xp on four virtual outputs: bars, desktops, a window moved between its outputs, one face removed | `gate-xp-faces.sh`: a window opens on the face under the pointer; xp moves it to the next face; removing a face moves its windows to another | xp runs on four virtual outputs, shown by the pane host as four flat panes | xp assumes its outputs are adjacent in the layout | one xp process per face; windows move by handover |
| 9 | Construct: the pane source "window manager" (K7): create the outputs, start the child with `--output`, show forwarding panes, end the child and remove the outputs | `gate-construct-wm-pane.sh`: an xp pane is live within 10 s; a click types into its window; a killed child returns its windows | K7 check of the roadmap | `chora-construct/src/main.rs` has 3,505 lines; split it first (judge finding 15) | load panes only from the construct file, no live add and remove |
| 10 | The cube geometry (D76): four slots at 90 degrees in a 3D context, turn navigation, hidden back faces | unit test of the four slot transforms; capture: at a turn of 30 degrees two faces show their colors; a static cube renders 0 passes; a window moved from face 1 to face 2 shows after the turn; frame time printed | **xp runs on four virtual outputs and the construct shows them as a cube** | crossing or edge-on faces show seams from the split and the feather | a prism that only shows one face at a time: turn by swapping the visible pane |

Order: 1 to 6 in sequence; 7 and 8 can run beside each other after 6; 9 needs 8; 10 needs 9.
After step 6 the docs get their text: `docs/api.md` (`output.add`, the uniform, `forward`),
`docs/architecture.md` (outputs), D69 and D70 as amended.

## 8. Risks and open questions

1. **Input complexity in `input.rs`** (1,946 lines). The spot through a portal, `press_via`
   and the unpinned release are small, but they sit in the path of every pointer event.
   Default: the 2D guard and `hits_match_the_flat_walk` after steps 4 and 5; a session without
   a virtual output takes no new branch but one `Option` check.
2. **Unmeasured cost.** Four faces and a screen on real hardware, and the tty path with two
   GPUs, are not tested in the container. Default: the bench script and a hardware cell in
   the hand-off.
3. **The removal wait.** 5 s decides between "the window manager still moves it" and
   "stranded". Default: 5 s, as for reservations; the KISS alternative of step 6 if it races.
4. **A real camera can show a virtual region.** A window manager that pans a screen camera
   to x below -262144 sees and hits the windows of a face directly. Harmless: it is the same
   scene and the same owners. Default: leave it; a camera limit can follow.
5. **The drag preview of K8** needs the unpinned point during the drag, not only at the
   release. Default: the construct shows the preview, since the panes are its own; decide in
   K8.
6. **Pulling a window out while the button is down** needs E3. Default: not built here.
7. **One level of portals.** A construct inside a face cannot forward again. Default: refuse
   and log once; lift it when a user needs it.
