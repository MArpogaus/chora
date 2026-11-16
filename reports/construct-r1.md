# construct r1 (rc 1): report

Lane: cloud, branch `claude/construct` from `bb8df12a` (multilook r1), head `81ff7b60` (pushed). Sonnet, high. 11-16.
The session branch `claude/session-log` shares no history with the code branches, so there was nothing to merge first.
Base for the gate: `--base bb8df12` (the default base ref does not exist in this clone).

| Item | Result | Red | Fix | Cell |
|---|---|---|---|---|
| co-18 BLOCK drag on the stepped-back cube, lost window | fixed | `f59df8fb` | `929b64e7` | `tests/gate-construct-out.sh` cell 1 |
| co-19 MAJOR F1 stepped back shows no overlay | fixed | `f59df8fb` | `929b64e7` | out cell 2 |
| co-20 MAJOR no key closes the cube | fixed | `f59df8fb` | `929b64e7` | out cell 3 |
| sh-24 MAJOR gated grab falls through as a raw key | fixed | `573f788c` | `da5a9c00`, `92dcc66f` | unit tests in `input.rs`, `gate-construct-pane-keys.sh` |

**co-18.** The cell failed on the head (not a tour script fault): with one window floating on face 1, a drag that begins on the
window gave `s1 xp face` before and `s1 construct HEADLESS-1` after, and the face label stayed at "face 1 of 4". Cause: a press
on the floating plane of a window carried the window; the first move past a click pulled it out of the cube into a pane
(`Dragged::Pull`, K12), so the cube never turned and the window left its face. Fix: a plain drag turns the cube wherever it
begins; `Dragged::Pull`, `pull_plane` and the window of a press are gone. Pulling out of a face in use mode (title drag past the
edge) and a plain drag of an existing pane over a face stay. Cell: window count, owner and output of each window before and
after the drag are equal, and the cube shows "face 2 of 4".
**co-19.** The key list never bound plain F1 (rule 25: F1 reaches the program of a pane). Stood back no program is there, so F1
shows the list now, only in navigate mode with no modifier; `Super+F1` is unchanged. Escape closes the open list first.
**co-20.** Stood back, plain Escape closes the cube (`close_cube`); Return steps in. Cell: the list takes the first Escape, the
second closes the cube, every window is still there (a pane of the construct on the screen, not owned by the ended xp).
Key list rows "Close the cube, stood back Esc" and "Show these keys, stood back F1" in the group Cube; docs updated
(`cube-modes.md`, `configuration.md`, `construct-style.md`).
**sh-24.** Red: `a_grab_held_back_by_the_pointer_is_swallowed_not_passed_on` and `an_injected_key_skips_the_pointer_gate` failed.
`grab_action` now looks for a grab without the gate, returns `Swallow` when only the gate held it back, takes no gate for an
injected key and looks up the claim under the pointer only after a grab matched. The escape hatch is untouched. The first
version skipped the gate for every injected key; the gate step `pane-keys` caught that the gate cell (injected keys, test mode
`--test-injected-press-is-real`) then lost its gating, so in that mode an injected key counts as the person's
(`92dcc66f`, unit test `an_injected_key_that_counts_as_real_keeps_the_pointer_gate`).

**Deviations, for the PM.**
1. The brief reads "Escape closes the cube". Escape stood back used to step in (docs, one cell); it closes the cube now and
   Return steps in. With `fly_out: Escape` and the grant, Escape steps back and Return steps in (cube gate cell changed).
2. The drag of the stood-back pull-out (K12) is gone, as co-18 asks. Windows are pulled out in use mode only. The cells of
   `gate-construct-pull.sh` for plane pulls became "pulls nothing out"; the cell "a plain drag of a pane over the front
   face gives the window back" lost its stood-back variant: I could not get a pane into view stood back without the pull
   (`pane.move` placed it behind the cube). `press_on_a_pane` has no cell stood back now. Backlog: a gesture to pull a
   window out of the stood-back cube, if wanted.
3. The gate script `out` is a new merge-gate step (`tests/merge-gate.sh`, help text).

**Gate (head `92dcc66f`, `--base bb8df12`).** All steps passed, in two parts: the full run stopped at `doors` (mc column,
screenshot-door and volume-door "missing", the known flake F16/F26; xp, tiling, construct pass; it failed three full runs in a
row after tiling-audit and passes `--only doors`, 330 s). Then `--only last-words,cube,pull,out,winit,wm-pane,pane-keys,walk,
keys,pixels,mc-paths,e2e` PASS (exit 0). Before doors, in the same run: red-first, pre-commit, gate-lib, build, test, clippy,
ci-check, 2d-guard, package-deps, xp, matrix, virtual, xp-faces, hud, layout, tiling-audit, chat-keys PASS. `--only doors` PASS.
Earlier runs: red-first and 2d-guard needed the fetch of `8742dee` (2d-guard base); pane-keys failed once (above, fixed).
Not run: `tests/ci-local.sh` (not touched by the change), a human look at the live demo.
State cleared: the swallowed grab sets no flag; `close_cube` removes outputs, chips, grabs (`close_modes` -> `regrab`).

**Captures** (look construct, build `81ff7b60`, 11-16, headless 1280x800): `reports/construct-r1/construct-81ff7b60-2026-11-16-drag-turn.png`
(after the drag, face 2), `-keys.png` (F1 stood back: the list with the two new rows), `-closed.png` (after Escape).
