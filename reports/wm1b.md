# WM1b r1: mc through kit::wm::book

2026-11-19 04:29 UTC. Lane: cloud, sonnet, branch `claude/wm1b` from 27bd60c7. Head 0ffc53a8. Brief: briefs/wm1b-r1.md.

## Verdict
Built; cells red first, then green. Merge gate once on 0ffc53a8 (`--jobs 3`, run /tmp/merge-gate.zgRQQO): all 41 steps PASS, exit 0 (e2e, acts, mc-handover, mc-sofa, mc-paths, doors, cube, tiling-look among them). acts: 47 pass, 5 known red (unchanged), 0 blocking. Independent review still to come (rules 7, 8): MERGE-OK pending it.

## Commits (red first)
| commit | what |
|---|---|
| 47d9dbbe | test(kit): `testing::Script` and `Subject` (wm1 proposal 2); wm_book.rs runs it over its mini-look; FakeCore answers `output.list` |
| 8d492e86 | test(mc): port cells; 2 red (script: no `mc.window` phases; state: no `origin`), restart adoption and scene order green as guards; `adopt_free` seam for the restart cell |
| 0ffc53a8 | feat(mc): lifecycle through the book; copies.toml; docs/api.md |

## What changed in mc
- claim filter, known-surface check (`adopt`) and `reattach` went; `windows.rs` has `lifecycle` / `settle`: Adopt (a held surface is put back: the detach policy; else adopted; refused or unbuilt answers `refused`), Forget (destroyed: old `forget`; detached and given: nothing, `give` cleans up as before), Phase (`mc.window`, rect null).
- `Mc::give` is a book proposal (`give`, `confirm`, `refused`); `gave` keeps the old cleanup.
- state adapter: windows and docks tell `origin` from the book (key absent when the book does not know the surface; `open` still absent).
- Book fed with every life event (`Book::hears`), `filters` false: mc still keeps its windows when its output goes (W11), as before.
- New kit fn `Book::announced_app` (the app id of a created surface) so mc names no `SurfaceCreated`.

## Cells (green on 0ffc53a8)
| cell | where |
|---|---|
| create, refused attach, reserved for xp, detach, destroy, output removed, give in, refused give out, give out | `mc_runs_the_script_of_the_life_of_a_window` (windows.rs), same `Script::run` as the book (`the_script_runs_for_a_book`) |
| origin in state, no token, `open` absent | `the_state_tells_the_origin_of_a_window` |
| restart: free and reserved-for-mc windows adopted, xp's left | `a_restart_adopts_the_free_windows_of_mc` (the core keeps no meta for mc windows; mc writes only `pane`, so the 29e2cd77 meta of mc is the free surfaces) |
| order of scenes | `the_order_of_the_scenes_follows_the_windows` ("taskbar order" read as the scene order and numbers of the overview; mc has no taskbar) |
mc: 139 tests, tiling 152, chora-kit (testing) all pass.

## Copies
`Event::SurfaceCreated outside kit` 4 to 2; new row `mc: Event::SurfaceCreated (WM1b)` at max 0; `to != NAME` 2 to 1 (the remaining line is mc's give-to-self guard in acts.rs, not a claim filter). cut-check ok, no FAIL.

## Coverage
`cargo llvm-cov -p chora-kit --features chora-kit/testing`: kit::wm lines 97.9 % (book 97.92, give 100, kept 97.30, state 100; 6 of 291 lines missed). Needs XDG_RUNTIME_DIR-less env only.

## Visible behaviour
Unchanged apart from two additions, both from the design: the event `mc.window` and `origin` in state.get. A detached window becomes origin `new` after it is put back (as in tiling). mc does not subscribe to node.requested, so no D58 pass-through for mc (not in the brief).

## Refactor proposals, ranked
1. Book owns the window list: mc keeps `windows` (scene to surface), `docks`, `fitting` beside the book; tests insert there and some need `book.keep`. Key mc's data by the book's ids in WM4.
2. D58 pass-through for mc (effects on the shells, playbook beat 9): subscribe to `node.requested`, give the book groups; a decision, not a port.
3. Put mc's `Windows::has` and the dock/fitting state on the book (origin for docks), then xp and construct (WM1c, d) reuse `Script` with a `Subject` each.
4. `acts.rs:31` `to != NAME` could use `Book::give`'s own-name check; then the `to != NAME` row is 0.
5. `phase_event` rect for mc windows (`region::place` knows it).
