
## r1

Branch claude/flake-k1: red ebb8dcb2 (K1_RACE switch), green 308f0915 (head; fresh origin/claude/autonomous-dev-session-3w-i33pg7 merged: already up to date).

Cause: the gate's 0.6 s sleep after the walk keys (tests/gate-construct-k1.sh:51-52) is not "at rest" on a loaded host. The walk is a stride (plugins/chora-construct/src/stride.rs steps_at/tap, 320 ms ease-out tail) stepped per frame by step_stride (camera.rs:258-293). A late frame leaves `after` mid-walk or in the 1e-5 tail. The fly-in stores `back = self.target()` (camera.rs:390-391); the fly-out glides to flown.back and sets pose = glide.to exactly (step_glide, camera.rs:414-440). So the fly-out is bit-exact to the pose at fly-in time and the gate read `after` too early. No product fault. Rest state: `state.get` field `busy` is empty (state.rs:97-109: no glide, walk, move).
Trace (50 ms steps after the keys, K1_TRACE temporary): 647.9649616725608 (+300 ms) then 647.9648992930119 (+350 ms) in one run: still easing at +300; the fly-out reaches the same final value.

Repro (K1 gate alone under a parallel cargo build --release, 4 cores; the merge-gate cell runs the same script): before, 2 of 10 red ("did not return to the pose", 0.0004 to 4 px). K1_RACE=1 (no sleep) red every time: 576.09 vs 592.48.
Fix: after the walk the gate waits `until_ok quiet 80` (state.get busy empty), then reads the pose. K1_RACE=1 is now green. After: 0 of 10 k1 red under load.
Gate: full tests/merge-gate.sh --jobs 2: all steps passed, EXIT 0 (construct-extra PASS).

Foreign flake, same gate, not fixed: "FAIL: the walk during the drag was lost: width N, then <empty>" (gate-construct-k1.sh ~line 125; width() empty after the drag back + 0.3 s sleep). Seen 2/10 before and 5/10 after, under load: sleeps again. Propose FLK-K1b.
