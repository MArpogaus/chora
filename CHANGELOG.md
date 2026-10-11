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
