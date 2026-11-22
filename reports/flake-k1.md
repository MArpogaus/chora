
## r1

Branch claude/flake-k1: red ebb8dcb2 (K1_RACE switch), green 308f0915 (head; fresh origin/claude/autonomous-dev-session-3w-i33pg7 merged: already up to date).

Cause: the gate's 0.6 s sleep after the walk keys (tests/gate-construct-k1.sh:51-52) is not "at rest" on a loaded host. The walk is a stride (plugins/chora-construct/src/stride.rs steps_at/tap, 320 ms ease-out tail) stepped per frame by step_stride (camera.rs:258-293). A late frame leaves `after` mid-walk or in the 1e-5 tail. The fly-in stores `back = self.target()` (camera.rs:390-391); the fly-out glides to flown.back and sets pose = glide.to exactly (step_glide, camera.rs:414-440). So the fly-out is bit-exact to the pose at fly-in time and the gate read `after` too early. No product fault. Rest state: `state.get` field `busy` is empty (state.rs:97-109: no glide, walk, move).
Trace (50 ms steps after the keys, K1_TRACE temporary): 647.9649616725608 (+300 ms) then 647.9648992930119 (+350 ms) in one run: still easing at +300; the fly-out reaches the same final value.

Repro (K1 gate alone under a parallel cargo build --release, 4 cores; the merge-gate cell runs the same script): before, 2 of 10 red ("did not return to the pose", 0.0004 to 4 px). K1_RACE=1 (no sleep) red every time: 576.09 vs 592.48.
Fix: after the walk the gate waits `until_ok quiet 80` (state.get busy empty), then reads the pose. K1_RACE=1 is now green. After: 0 of 10 k1 red under load.
Gate: full tests/merge-gate.sh --jobs 2: all steps passed, EXIT 0 (construct-extra PASS).

Foreign flake, same gate, not fixed: "FAIL: the walk during the drag was lost: width N, then <empty>" (gate-construct-k1.sh ~line 125; width() empty after the drag back + 0.3 s sleep). Seen 2/10 before and 5/10 after, under load: sleeps again. Propose FLK-K1b.

## PM review r1 (diff 6b01c976..308f0915: tests/gate-construct-k1.sh +5 -1; PM not the builder)
Checks: red first (ebb8dcb2 K1_RACE before 308f0915); the wait names a real rest state (state.get busy, state.rs:97-109), not a sleep alone; no product change (camera.rs fly-out lands on flown.back exactly, as the report says); full gate EXIT 0.
- rw-1 MINOR gate-construct-k1.sh:52: the 0.6 s sleep stays in the normal path; with the busy wait it is dead time. Fix: drop it in r2.
- rw-2 MINOR gate-construct-k1.sh:103-123: the mouse part (wheel, drag, walk in a drag) still reads after bare sleeps 0.6/0.3/0.5/0.3; FLK-K1b "the walk during the drag was lost: width N, then <empty>" 2/10 before, 5/10 under heavier load. Fix: lane flake-k1 r2.
Verdict (computed): 0 BLOCK, 0 MAJOR, 2 MINOR: MERGE-OK; joins train t110.

## r2

Branch claude/flake-k1: head 7d8ad513 (fresh origin/claude/autonomous-dev-session-3w-i33pg7 merged: already up to date). Red 27522585, green 7d8ad513.

Repro (k1 gate alone, 10 runs under a parallel `cargo build --release` in another target dir, 4 cores): 7 of 10 red, all "FAIL: the walk during the drag was lost: width N, then <empty>" (widths 372-416). One red run traced every 50 ms for 2 s after the drag back: rects `{}` in all 40 samples, `state.get` busy `[]`, windows shown false. So not easing: the pane is at rest and out of the view.

Cause: not a rest state, and no product fault. A walk tap starts from where the running walk of the key before is (camera.rs:240-247 `from = s.at(now)`; stride.rs:23-37 a tap is 800 px in 320 ms), so the pose after `e e e a a s s` and after `w w` depends on the time between the keys (800 to 1600 px for w w), not only on the keys. Stage reads in red runs: pane x before the w w 840, 886, 921 (green runs 625-720), after the drag back no rect (camera.rs:350-354 in_sight, space.rs:267-290 sight: the pane's spot is "aside", off the output). The pane is where a user who walks past it at an angle would find it out of sight; a user would see that, and nothing is lost. The gate's geometry (drag right, two steps) only held for the short walks.
Fixed pose: with every key waiting for rest, `w w` is always two full steps; the drag right then leaves the view every time (3 of 3 red, width 377, then <empty>).

Red 27522585: each key, the wheel, the drag and the drag back wait for the pane to move from its read and then for state.get busy empty (rest_after_move); the sleeps at :52, 0.6, 0.3, 0.5, 0.3 are gone (rw-1, rw-2); K1_RACE skips these waits. Gate red 2 of 2 (deterministic: width 377, then <empty>).
Green 7d8ad513: the drag during the walk goes left (position 70, not 130), the turn that keeps the pane in the view: 377 to 641 every time (3 of 3 unloaded).

After: 20 runs of the k1 gate under a looping `cargo build --release` (load average 6 on 4 cores): 20 of 20 rc 0, 0 red lines.
Gate: full `tests/merge-gate.sh --jobs 2` on claude/flake-k1 (merge with origin/claude/autonomous-dev-session-3w-i33pg7: already up to date), 51 PASS, 1 FAIL: mc-scale "settings.set of the write time beside 300 panels takes 5658 ms, more than 5000 ms" (/tmp/gate.log; FLK-MC-SCALE, known, not mine; not rerun). construct-extra PASS 174 s (k1 inside). EXIT 1 for that cell alone.
