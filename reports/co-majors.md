# co-majors

## r1

Lane: cloud, branch claude/co-majors from 30727c82, head db4d703b (pushed). Setup:
`git fetch --unshallow`, `cargo check --workspace` and `cargo build --workspace` passed.

| id | on the head 30727c82 | red | fix | cell |
|----|----------------------|-----|-----|------|
| co-19 | gone: stood back, F1 shows the bindings overlay | f59df8fb (guard of construct r1) | 929b64e7 (gone before) | `out` (tests/gate-construct-out.sh) PASS |
| co-20 | gone: stood back, Escape closes the cube and no window is lost; the overlay lists "Close the cube, stood back Esc" and "Show these keys, stood back F1"; stepped in, Alt+Escape steps back (listed) | f59df8fb | 929b64e7 (gone before) | `out` PASS |
| co-21 | reproduced in the tour v7 scene (demo/scene-cube.sh, 1280x720 + 640x720 edge) and in the pull gate | 85691f86 | db4d703b | `pull` (2a, closed-cube check) PASS |

### co-21: what was broken

Not the cube. On the head, stood back after the pull-out of the Sunset window, scene.boxes
gives faces 0 and 1 a shared edge (792,229)-(792,491) on both faces, and the mirrors line up
with them (x 416-792 and 792-864). The "neighbour face" of the owner's still is the pane of the
pulled Sunset window, `construct-pane-s1`. It was dropped in front of the face, 0.9 of the
depth of the face. The step back moves the cube 2 focal lengths away from the eye. The pane is
placed by the person, so it does not give way to the cube (d33bbc8e), and it stays near the eye.
It covered x 599-1257 at 1.8 times the height of the face, over face 0, the seam and face 1,
and was drawn above them, so it read as a face that juts out. The reflection still lay under
the real faces, so it looked as though it did not line up.

Fix (construct plugin only, new plugins/chora-construct/src/cube/aside.rs): when the cube
starts to step back, a placed pane that is nearer than the cube and would overlap its
stood-back screen box gets a place beside the cube. The place is at the depth of the cube, on
the side the pane was on, half a face diagonal plus 0.15 face widths plus half the pane width
from the centre. The pane glides there with the stand. It goes home on step in or close (the
rect is the same as before the step back), and a pane that the person moves meanwhile stays
where it was put. Drag back onto a face still works (pull 2a, super and plain). docs/cube-modes.md
says so.

The red check (tests/gate-construct-pull.sh 2a, `closed_cube_breaches`, by scene.boxes on
HEADLESS-1): the faces that show share an edge within 1 px, at a length equal within 2 %, each
mirror lies under its face within 1 px, and no pane is drawn over a face. On the head:
`construct-pane-s1 [-43, 18, 715, 764] lies over face 0 [416, 255, 375, 290]`.

Captures (reports/co-majors/):
- co-21-tour-30727c82-before.png: the tour v7 still reproduced, Sunset over the cube
- co-21-tour-db4d703b-after.png: same steps, cube closed, Sunset beside it with its mirror
- co-21-gate-pull-30727c82-before.png / co-21-gate-pull-db4d703b-after.png: the gate case
- co-19-20-out-30727c82-keys.png: F1 stood back, the overlay with the Esc and F1 rows

### Gate

Merged origin/claude/autonomous-dev-session-3w-i33pg7 (30727c82; already in, nothing new), then
`tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,cube,construct-hud,pull,placed,keys,doors,out`
on db4d703b: all 12 PASS (pre-commit 3 s, build 25 s, test 182 s, clippy 18 s, ci-check 89 s,
cube 114 s, placed 14 s, pull 63 s, out 14 s, keys 18 s, construct-hud 115 s, doors 333 s).
`out` is the extra cell for co-19 and co-20.
