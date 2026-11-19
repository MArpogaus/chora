# WM3 r1: kit::wm::open, the open shell surfaces

2026-11-19. Lane: cloud, sonnet, branch `claude/wm3` from 1d1aa5fd, merged with the session branch at 4548feb3. Head 0e0bd700. Brief: briefs/wm3-r1.md.

## Verdict
Built and ported; the cells are red first, then green. Merge gate once on 0e0bd700 (`--jobs 3`, run /tmp/merge-gate.NGi7SB): "all steps passed" (acts 84 pass, 4 known red, 0 blocking). The first gate run (/tmp/merge-gate.WSEoa8) failed in step `pull` ("pane of the pulled window ... 194 px wide for 200", a construct wobble cell, not code I touched); the step passed 3 of 3 alone and the second full gate passed: a load flake of `pull` under `--jobs 3`, not fixed here. Independent review still to come.

## Commits
| commit | what |
|---|---|
| ee61246e | test: `kit::wm::open` skeleton, 5 red cells in chora-kit; 2 red cells in chora-xp (a give, a give with the drag leave the Start menu open); guards: xp cell "a press on a title bar shuts the menu" and the act "a drag of a window shuts the Start menu" (green, see below) |
| 1c3c4d99 | feat: `chora_kit::wm::open`; xp, tiling, mc, construct list through it; known-red WM3 rows removed; copies rows |
| 0e0bd700 | merge of the session branch (conflicts: xp wm/state.rs cells, known-red.toml) |

## What is built
`chora_kit::wm::open`: `Open` (the list, oldest first), `Surface {id, kind, modal, by, stays}`, `Kind {Menu, Popup, Dialog, Sheet, Launcher, Switcher}`, `Why {Drag, Give}`.
`open`, `shut`, `escape` (newest first, one per press), `modal`, `shut_at(why, keep)` (shuts every surface that is not modal, not declared `staying` and not in `keep`; returns them newest first), `state()` (Escape order, feeds `wm::state::State.open`, which gained `by`). Switcher is the one kind beyond the plan's five (xp has one).
- xp: Start menu, context/system menu (`by` = the window), Turn Off dialog (modal), settings dialogs (staying), switcher (staying) register where they open and shut; `state.get` reads the list; new `wm/surfaces.rs`. A give (`give_dragged`) and the start of a title drag call `shut_surfaces`.
- tiling: `keys_open: bool` gone, the key list is a modal sheet of the list. mc: `open` now in `state.get` (key list when all bindings show). construct: the open list built through `Open` (kinds ask -> dialog, list -> popup).

## Honest deviation from the brief
The brief's act ("xp opens the Start menu, drags a window; `open` must shut") is already green on the base: a press on a title bar closed the menu (`press_node`). I kept it as a guard (`tests/acts/looks/xp.py`, cue `drag_title` in harness.py). The sd-h2 bug that reproduces is the other half: a **give** (key give, `window.give`) and the drag that pulls a window out of a face (`pull_out`) never closed the menu; the red cells are the two chora-xp unit cells in `wm/state.rs`. An act with a give needs a second look in the session (xp and tiling): not built, see proposals.

## Also fixed (cells turn green, known-red rows removed, as the rule says)
- net2-a1: Escape did not cancel Turn Off Computer: the dialog never took the keyboard. It does now (`Dialog::FOCUS`), Escape cancels, the desktop gets the keys back.
- net2-a3 (3 rows): mc reports `open` (its key list).
Known red went 8 to 4 on the merged base (`mc I3`, two `I5`, `I13`), none added.

## Cells
chora-kit `wm/open.rs` 5 unit cells + doctest; chora-xp: give, give with drag, title-bar press, state order; mc: key list open / shut; tiling and construct: the existing open cells run on the typed list.
Tests on 0e0bd700: chora-kit (testing) 273, chora-xp 198, chora-tiling 152, chora-mc 141, chora-construct 198, all green; clippy -D warnings clean.

## Coverage
`env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-kit --features chora-kit/testing`: kit::wm lines: open.rs 100 % (157 of 157), book 98.13, give 100, kept 97.30, state 100.

## Copies (tools/copies.toml)
New: open entries of state.get built by hand outside kit (max 4: the cells that read state.get); tiling `keys_open: bool` max 0. cut-check ok.

## Visible behaviour
Changed only where the bugs were: a give and a drag out of a face shut the Start menu and a context menu; Escape cancels Turn Off. The `open` kind words of construct changed (`ask` -> `dialog`, `list` -> `popup`) and xp's context entry says `by` instead of `for`; nothing reads either.

## Refactor proposals, ranked
1. A two-look act session (xp and tiling): act "a give shuts the Start menu" and "Escape order of a menu above a dialog" at the act level (I4 with a give). S-M.
2. xp: the Start menu and context menu models still keep their own `is_open`/`Option` beside the list (two writers, kept in step at 6 sites). `Menu::open` returning the Surface, or the list as the only owner with the models as drawings. M.
3. `Open::escape` is not wired in xp: Escape still goes to each model (menu, context, Help, now the dialog). One `escape` would give the one Esc order of the plan; needs xp's Help panel and cascades to answer it. M.
4. `Why` is unused by `shut_at` (the rule is the same for both); either drop it or let a look declare "stays through a drag but not a give". S.
5. tiling's launcher (fuzzel) and mc's chat are outside programs/windows and not on the list; the launcher kind is used by construct only. Decide whether an outside launcher registers when the look starts it. S.
