# Review flakes r2 (claude/flakes 498d9dea...55c527d4), opus, read-only, 11-14 20:2x UTC

Verdict: FIX (R1). F14, F13, F10/F11 right; red tests real; the time field reaches real, injected and winit input; serde default keeps old clients.
| id | sev | file:line | text |
|---|---|---|---|
| R1 | MAJOR | crates/chora/src/input.rs:552 refresh_pointer, :557 drag_node | each applied transaction refreshes the pointer with InputTime::now(); during a node grab a pointer.moved with the handling time goes out as the cube turns under a still pointer; under load it can land after the release time and lower the flick speed (the fixed bug, back on a real hand). The gate misses it (injected moves come before the frame). Keep the last input time in State for refreshes, or send no move when the position did not change. |
| R2 | MINOR | input.rs:577,1500 | InputTime::now() is u32 ms, wraps after ~49.7 days; docs say monotonic without that. |
| R3 | MINOR | event.rs:239,267 | a missing time becomes 0; then every drag counts as a flick. Treat 0 as unknown. |
PM decision (rules.md 7 exception, logged): flakes merges with train 5 anyway. R1 is no regression against the base (before, every sample had the handling time), and F14/F13 unblock every gate. R1 is the first item of flakes r3.
