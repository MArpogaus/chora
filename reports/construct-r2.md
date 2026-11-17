# construct r2 (rc 1): report

Lane: cloud, branch `claude/construct`, sonnet high, 2026-11-17. The session branch `4105f8d1` fast-forwarded into
`claude/construct` first (the branch was an ancestor). Head `59a16702` (pushed).

| Item | Result | Red | Fix | Cell |
|---|---|---|---|---|
| N2 MAJOR pane gone after Alt+Escape | fixed (product fault, not the tour script) | `cccb3c98` | `d33bbc8e` | `tests/gate-construct-pull.sh` cells "stepped back after the pull-out the pane ... is drawn" and "stood back, the pane dragged over a face is a window of xp again" |
| rv5-3 MAJOR HUD has no helper button | fixed | `8e97f969` | `59a16702` | `tests/doors/construct.sh` check `helper-button` |

**N2.** The cell failed on the head, 4 of 4 runs: after the pull and `Alt+Escape` the node `construct-pane-s1` exists with
`visible: false` and no rects; the window count is unchanged and the window is owned by the construct, so nothing was lost,
only hidden. Cause: `Construct::covers_cube` hides every pane that is in front of a cube that stands back and overlaps it on the
screen ("the pane gives way to the cube"). A pulled-out pane is put in front of the face, so it always overlaps the cube once the
camera steps back. The comment promised "a pane beside the cube, as one that was pulled out of it, stays", which only held when
the drop point happened to clear the cube (so the cell passed 2 of 6 runs before the cell waited long enough; the poll used to
catch a transient frame, see Deviations). Fix: a pane that was put somewhere (`Pane::put`: a drag, the pull, `pane.move`) is
`placed` and no longer gives way; a pane nobody placed still does (gate-construct-cube.sh cell "the pane gives way to the cube"
passes unchanged). `docs/cube-modes.md` says so. The cell also drags the pane back over a face with Super held: the window is
xp's again on VIRTUAL-1, the pane is gone, window count equal. Unit test `a_move_marks_the_pane_as_placed`.
**rv5-3.** The strip (doors) names `Helper` (`Helfer`) after `Programs`; a click runs `Action::Chat`, as `Super+a`. Before the
fix `construct-door-helper` did not exist (check `helper-button` failed: no node). The island at the top right stays (it shows
the state word; its click still opens the chat, cell `helper-door` passes). Checked at 1280x800 and 1024x768 (layout audit, no
finding). `docs/configuration.md` names the word.

**Gate (head `59a16702`, one run, `tests/merge-gate.sh`, base default).** red-first, pre-commit, gate-lib, build, test, clippy,
ci-check, 2d-guard, package-deps, package-lists, xp, matrix, virtual, xp-faces, xp-paperclip, xp-fields, xp-question, hud, layout,
tiling-audit, tiling-look, chat-keys, doors, last-words, cube, pull, out, winit, wm-pane, pane-keys, walk, keys, pixels,
mc-paths, mc-sofa PASS. **e2e FAIL** at its step 24 `gate-tiling-apps-click`: "a click on the Zebra row did not start zebra".
Not this lane: tiling, known as F27 / ti-1; this container has fuzzel 1.9.2 (setup.sh was not run, the pin is 1.11.1). In
the same e2e run `gate-0.9-session-units` (no desktop-file-validate) and `gate-0.7-6` (no fkeys option) skipped. So the gate is
not green on this container; every step of the lane's area is. Log `/tmp/merge-gate.Ivdmqj/e2e.log` (not kept).
Also run alone: `tests/gate-construct-hud.sh` fails "the pane ahead has no label" on `cccb3c98` too, i.e. before my change
(listed red in doors-r2.md); it is no gate step.

**Deviations.** (1) The first red cell polled `until_ok`, which caught a transient frame during the step-back and passed 2
of 6 runs; the committed cell waits 3 s and reads once (4 of 4 red on the head, 4 of 4 green on the fix). (2) A dragged pane
also counts as placed (`put` is the only mover), so a pane the person moved in front of the cube stays over it; the person put it
there. (3) Gate not green only through the e2e step above.

**Captures** (look construct, build `59a16702`, 2026-11-17, headless 1280x800, 1x):
`reports/construct-r2/construct-59a16702-2026-11-17-pane-stepped-back.png` (cube stood back, the pulled pane beside it),
`construct-59a16702-2026-11-17-walk-3-panes-1x.png` (strip with the word Helper, island No helper).
