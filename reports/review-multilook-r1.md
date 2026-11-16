# Review multilook r1 (sonnet, read only, 10:47 UTC)

Base caf9c80d, head bb8df12a (5 commits, 7 files, +158/-8). Code reading; unit build not finished, gate results accepted from the builder.

- sh-23 holds: input.rs:466-471 gates a virtual-output manager's grab while the pointer is in another manager's claim; fails open for real-output, mixed or claimless managers. Escape hatch keys decided before grab_action.
- co-16 holds: bb8df12 minimal; modifier press on a pane still moves it.
- 1 MINOR: a starved grab falls through to the focused client (stray key, shared RC bar) -> sh-24.
- 2 MINOR: keyboard-only use follows a stale pointer; global hotkeys of a virtual-only manager starved.
- 3 LOW: injected keys gated by pointer position too (input.rs:433).
- 4 LOW: key_place scans all nodes per key press (input.rs:475-505).
- 5 LOW: no test of the grab_action wiring, pointer leave, removed output.
- 6 LOW: with a cube open there is no obvious key to close the cube (only cube.close over IPC) -> co-20.

Verdict: MERGE-OK. PM: 1 and 6 touch the shared RC bar and POLS, raised to MAJOR (sh-24, co-20), in construct r1.
