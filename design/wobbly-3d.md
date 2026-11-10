# Effects for every window manager, and wobbly windows in 3D

User request 11-09 (evening, Bangkok 11-10): window effects of the XP
split become general and usable by every window manager; wobbly windows
in 3D with the same code; a showcase: walk through the construct, stand
in front of the XP cube, grab a window and pull it out of the cube into
the 3D space, wobbling. Candidate r/unixporn demo.

## What exists

- The physics is already shared: `chora_kit::wobbly` (408 lines, the
  KWin spring model, 4x4 control points, a 20x20 drawn mesh) and
  `chora_kit::lamp` (the minimize effect).
- Only the driver is XP's own: `chora-xp/src/wm/effects.rs` (307 lines)
  turns grab, motion and release into `mesh` updates of the window group.
- The scene mesh already carries a z per grid point (`Mesh.depth`), and
  the 3D base (D75, merged 11-10) draws meshes in a perspective context as
  one unit and gives input only where the triangles are.
- XP split X6 already plans "a function of chora-kit for update and
  animate requests (D58): a request of an effects plugin reaches a window
  of chora-tiling and of chora-stack".

## Design

E1. One effects plugin, not one copy per window manager. `chora-effects`
    (X6, pulled forward) watches grabs and moves of windows and drives
    the meshes through `node.requested` (D58) on any window manager's
    window group. A window manager opts in with one line
    (`effects: [wobbly, lamp]`) and answers `node.requested` through the
    chora-kit function of X6. xp's `effects.rs` moves there; xp keeps
    only the opt-in. tiling, mc and construct get it the same way.
E2. Wobble in 3D. `Wobble` gains a third axis: the springs act on
    [x, y, z]; z comes from the drag velocity along the view direction
    and from the pane's own motion in the space. In 2D z stays 0, so the
    2D look is unchanged (2D guard). The mesh goes out with `depth`.
E3. Hand over during a drag. Today a give happens at the release. To
    pull a window out of a cube face, the window must leave xp while the
    button is still down, because xp draws only inside its virtual
    output. New: `window.give` with `grab {point, velocity, wobble}`;
    the core moves the pointer grab to the receiver in the same
    transaction; the receiver continues the drag and the spring state,
    so the motion is seamless. Rules: only between window managers
    (claim holders, unconfined), only during a real press (the act rules
    of 11-10), never for injected input; security review.
E4. The stage: construct on 3D (K5), virtual outputs (D69) and the cube
    geometry (D76, K9/K11) with xp on its faces. These are the week-1
    plan anyway.
E5. The demo: a scripted walk (camera path), the grab, the pull out, the
    wobble in 3D, the drop as a construct pane; captured as a GIF and a
    short video for the status page; a push-to-the-limit persona panel.

## Order and size

| Step | Content | Size | Depends on |
|---|---|---|---|
| E1 | chora-effects plugin, X6 kit function, xp opt-in, tiling/mc opt-in | about 1 day | none |
| E2 | 3D springs and depth in `chora_kit::wobbly` | half a day | none |
| E4 | construct on 3D, virtual outputs, cube geometry | 2 to 3 days | 3D base (done) |
| E3 | give with grab | 1 day plus security review | E4 for its test |
| E5 | demo script, capture, persona panel | half a day | all |

## Risks

- Virtual outputs (D69) are the largest unknown; the cube depends on them.
- A mesh in a context sorts as one unit and never splits: a wobbling
  window that crosses another pane may draw wrongly where they cross.
  Acceptable for the demo; a split of meshes is a later step.
- Software GL in the container: the frame rate of the capture is not the
  frame rate on real hardware; the demo states the machine.
- The grab transfer is a new core power: it needs the same review as
  the act-stamped tokens.
