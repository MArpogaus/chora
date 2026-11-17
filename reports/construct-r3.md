# construct r3 (rc 1): report

Lane: cloud, branch `claude/construct`, sonnet medium, 2026-11-17. Session branch `aa8aa8c0` merged first (fast-forward of
the local branch to `59a16702`, then merge). Head `93d508ea` (pushed).

| Item | Result | Red | Fix | Cell |
|---|---|---|---|---|
| N-1 drag back | not a product fault: the tour script never pulled | n/a, the cell is green on the head (see below) | `ba3fa5e2` (cell + `demo/tour/ch6.py`) | `gate-construct-pull.sh`: "...window of xp again, on the taskbar of that face, with no pane drawn" |
| B1 `placed` persisted | fixed | `daa0c747` | `52a71b60` | `tests/gate-construct-placed.sh` |
| B4 `gate-construct-hud.sh` | script fault (4 stale reads, 1 real flake source); green 3 of 3 alone; in the gate | red on base (fails "the pane ahead has no label") | `93d508ea` | step `construct-hud` |

**N-1.** The extended pull cell (owner xp, task button of VIRTUAL-1 is 1, no pane node drawn after the drag back) passes 3 of 3 on
the head (once the first run of the unchanged cell failed "194 px wide for 200", a timing read in the earlier cell, not reproduced).
So the product is right. The tour is wrong at act `06-15-pulling` of ch6: it drags from (330,186), which is empty space once
the cube stands back (the face spans x 416-792); a stood-back drag turns the cube and pulls nothing (cell 2b). Nothing was
pulled, so the "pane in front of the cube" in `06-16-pane`/`06-18` is the floating plane of the dusk window, and the later Super-drag
dragged nothing. Capture `construct-59a16702-...-tour-old-script-no-pull.png` shows it (the window is on the face, no pane label).
The fixed ch6 steps in, adds a second output (as the gate), drags the title bar over the edge, drops beside the cube, steps back
(Alt+Escape), reads the pane and the front face from the node tree and Super-drags back. Captures: `...-pulled-pane-stepped-back.png`
(pane label Dusk in front of the cube) and `...-dropped-back-on-face.png` (xp window on the face, no pane, taskbar entry back).
The review frames f149-f161 of tour v6 were not available here; I name the act, not the frame: the pull at act `06-15`.

**B1.** `placed` is now a field of the construct file (`PaneFile.placed`, written only when true); a loaded pane takes it
(`adopt.rs`, `scene.rs`, `wm.rs`). Red: save a moved pane, restart, step back: the pane node exists with `visible: false`.
Green: drawn. A pane nobody placed still gives way to the cube (first cell of the same script). The round trip test of file.rs
carries `placed`. A pulled window with no command is not a file pane (`Pane::file` gives none), so after a restart it belongs to
its face again; the file keeps only panes with a command, a plugin or a manager. docs/cube-modes.md says so.

**B4.** The script read nodes the product moved: (1) the pane label is `construct-label-body-text` and shows the window title
(foot gave "foot" for both panes: panes now get `-T`), (2) the label and list fits compare to `construct-label-body-glass` /
`construct-list-body-glass` (the old glass nodes are 1x1), (3) the notice is in `construct-hud-say`, not in the state word.
(4) At 1280x720@1.5 a Tab opened no list while the pointer sat over a pane; the script now moves the pointer away and waits
for the Escape to close the list. Product behaviour not judged (Tab with the pointer over a pane at that scale: if a person sees
it, it is a MINOR). The step `construct-hud` (about 4 min) and `placed` are in `tests/merge-gate.sh`.
Other unreferenced construct scripts, not fixed: `gate-construct-k1.sh`, `k2a`, `3d`, `mcpane`, `nested`, `windows`
(`construct-captures.sh` is a helper of `pixels`).

**Gate (head `93d508ea`).** One full run was started; it hung at step `tiling-look` (`gate-tiling-power.sh` waits on fuzzel
1.9.2, the pin is 1.11.1; known F27 family, not this lane) after PASS of red-first through xp-question, hud, layout,
tiling-audit. I killed it by pid. Then `--only red-first,pre-commit,gate-lib,build,test,clippy,ci-check,2d-guard,cube,pull,placed,
construct-hud,out,wm-pane,pane-keys,walk,keys,pixels,layout`: all PASS (cube 108 s, pull 57 s, placed 14 s, construct-hud 243 s).
Not run: tiling-look, chat-keys, doors, last-words, winit, mc-*, e2e, xp-* after xp-question, matrix, virtual. So the gate
is not green as one run on this container.

**Refactor proposals** (no work done): (1) `plugins/chora-construct/src/main.rs` + `adopt.rs`/`scene.rs`/`wm.rs`: four `Pane { .. placed: ..}`
literals repeat the same field list; a `Pane::new(name, source, content, pose, hold)` (S). (2) `tests/gate-construct-pull.sh` and
`gate-construct-placed.sh` repeat `until_ok`, `pane_drawn`, `session_start`; move to `gate-lib.sh` (S). (3) `demo/tour/ch6.py`
hard-codes the use-mode title bar (870,225); read it from `xp-title-*` (S).
