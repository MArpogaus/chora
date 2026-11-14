# Review flakes r3 (claude/flakes 55c527d4..17f98f8a), opus, read-only, 11-14 22:2x UTC

Verdict: MERGE-OK. R1 (pointer_time on every source), R2, R3, F15 (readline Meta prefix, not core), xp cascade turn, kit start_grabs/end_grabs: right causes, red tests real.
| id | sev | file:line | text |
|---|---|---|---|
| M1 | MINOR | plugins/chora-construct/src/cube.rs:255 | a 32-bit wrap during a press gives a false flick (once per 49.7 days). |
| M2 | MINOR | plugins/chora-xp/src/window.rs:~416 | a window that closes before its first size uses a turn (30 px gap); turn after wrap can land on the corner window. |
| M3 | MINOR | crates/chora-kit/src/runtime.rs end_grabs | a failed input.ungrab drops the rest of the old set from tracking (as the old ungrab_all). |
| M4 | MINOR | process | no full gate on the merged head 17f98f8a (the train gate covers it). |
Follow-up (real bug class, low priority, xp first): xp wm.rs:1371, tiling main.rs:586,1610, mc main.rs:960, construct scene.rs:184 still ungrab_all + grab; swap to start_grabs/end_grabs.
