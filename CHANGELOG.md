## v0.3.0 (2026-10-11)

The showroom (milestone M3): layers with meshes, 4x4 transforms and GLSL
effects, animations in the core, image nodes and pointer events over
nodes. The new XP flavor shows them off: a desktop in the style of
Windows XP with wobbly windows, a magic lamp minimize and a desktop cube.
Log in with the session "chora (Windows XP)". No file of Microsoft ships
with it.
Nested, `tests/nested.sh` and `tests/xp.sh` check the features.

### Known issues

- Nobody has run this release on real hardware yet: the tty backend,
  effects on a GPU and the 60 Hz goal on Intel graphics are untested.
- Layers render again in every frame, even when nothing in them changed.
- Layers have no depth test: they draw in the order of their siblings.
- A rotated or projected layer takes the input over its bounding box.
- Every client of the user can take the keyboard as an input method and
  configure the outputs, as in sway; sandboxed clients lose this with
  `wp-security-context-v1` in M4.
- Taskbars of other programs never see a toplevel as minimized.
- The XP flavor is a demo, not a daily driver:
  - windows have no resize borders, and there is no Alt+Tab;
  - fullscreen requests are ignored;
  - the cube shows only the first output;
  - windows that draw their own frame cannot be moved;
  - All Programs opens the launcher;
  - while the cube turns, lifted windows of the far side can hide behind
    the near side.

### Feat

- **xp**: light up buttons and menu entries under the pointer
- tell plugins when the pointer enters and leaves a node
- **xp**: leave the frame to windows that draw their own
- **xp**: wobble windows, minimize through a lamp, turn a desktop cube
- tell plugins who draws the frame of a window
- shade layers with GLSL effects that reload on change
- report size changes, and read events with a timeout
- **xp**: add the XP flavor, its session and its RPM files
- **xp**: add chora-xp, a window manager in the style of Windows XP
- **xp**: add the theme of the XP flavor
- **safety**: remove effects and reset when frames stay slow
- render layers through meshes with full 4x4 transforms
- tell plugins about presses and drags over nodes
- animate node properties in the core
- add image nodes with nine-slice scaling

### Fix

- harden the watchdog, layers, images and effects
- **winit**: pace frames without damage instead of rendering at once again
- **input**: send pointer.moved only for a new position
- **ipc**: let plugins grab Ctrl+Alt keys outside the safety layer
- **scene**: read nodes without effects back from their JSON
- **xp**: read queries without effects, events through the client

## v0.2.0 (2026-10-11)

The session on real hardware (milestone M2): log in to chora with the
tiling flavor, with a panel, a launcher, a lock screen and X11 programs.
Nested, `tests/nested.sh` checks waybar, fuzzel, swaylock, wlrctl and
wlr-randr.

### Known issues

- Nobody has run this release on real hardware yet: the tty backend, the
  login session and the tiling flavor are untested there.
- On the tty backend, a mode change through wlr-output-management that
  fails at the next frame is not reported to the client.
- Every client of the user can take the keyboard as an input method and
  configure the outputs, as in sway; sandboxed clients lose this with
  `wp-security-context-v1` in M4.
- Taskbars never see a toplevel as minimized.

### Feat

- change modes and turn outputs off on the tty backend
- implement wlr-output-management in the core
- **tiling**: start a launcher with MOD+d
- show the new buffers of a transaction with it
- **tiling**: close a window when a taskbar asks
- add wlr-foreign-toplevel-management for taskbars
- report presentation times
- add a login session with the tiling flavor
- add input, idle, scale and toplevel protocols
- **scene**: add the surface.requested event
- run on real hardware, with panels and a lock screen

### Fix

- **session**: give X11 programs DISPLAY and drop a stale environment
- **tty**: keep removed output globals, unique names, dark off outputs
- **lock**: confirm a lock only with frames rendered under it
- keep the GL flip of the winit window out of the output
- **layers**: announce the work area when a panel goes away
- **input**: switch the VT by the modified keysyms
- harden the lock screen, popup grabs and the tty backend
- refresh the pointer after transactions and keep re-map sizes
- **scene**: validate cameras strictly and limit them to 64
- **render**: scale windows by the scale of their node and camera

## v0.1.0 (2026-10-11)

The nested proof of concept (milestone M1): run `chora -- chora-tiling`
inside another Wayland session.

### Known issues

- A camera or a node with a scale other than 1 moves windows, but does not
  resize them.
- The number of cameras has no limit, and a camera change does not update
  the surface under the pointer until the pointer moves.
- A window that unmaps and maps again loses the size that its plugin set.

### Feat

- show the scene on each output through a camera
- grab input for popups, so that menus close on a click outside
- start the emergency reset on SIGUSR1
- add the chora-tiling reference plugin
- add choractl with query, apply, events and outputs
- allow quoted selector values with spaces
- add the nested compositor core with ipc and transactions
- add the scene model with ownership rules

### Fix

- **transaction**: let late surfaces catch up with their newest configure
- **ipc**: send the events of a close in the same loop iteration
- **scene**: leave surfaces out of the node limit
- keep the surface node when a client maps it again
- **safety**: start the startup command before SIGUSR1 is blocked
- **ipc**: close connections that send no hello within 5 s
- **scene**: limit the scene to 16,384 nodes
- **transaction**: stop waiting for surfaces that missed a deadline
- **ipc**: answer waiting transactions after the end of the stream
- **safety**: block SIGUSR1 before any thread starts
- send surface.destroyed only for surfaces with a node
- **scene**: reserve ids of the form s1 for surfaces
- give choractl clear errors and a clean end of events
- keep the tiling plugin consistent with the scene
- let the user type after an emergency reset
- keep host session variables from the startup command
- harden the ipc socket and the surface lifecycle
- limit waiting transactions and copy the scene once per check
- block pointer input only where the blocking node is
- skip nodes with a scale that is not positive
- reject negative sizes and numbers that are not finite
- let the ipc client skip unknown events and null-id errors
- reject size and color on nodes that are not rects
- attach only surfaces without an owner
- clear the properties of detached surfaces
- report every detached surface exactly once
- walk the scene without recursion
- install the license file also without docs
- create the rpm output directory before the build
- catch the deprecated GPL-3.0 identifier in the hook

### Refactor

- **render**: chain the hit test of surface_under
- simplify event filters, IPC reads and hit tests
- drop dead state fields and a hand-made rounding helper
- drop duplicated serde defaults and event names

### Perf

- **scene**: keep the nodes in a persistent map
- draw only damage and throttle frames of hidden windows
