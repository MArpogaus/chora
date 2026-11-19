# WM1c r1: construct through kit::wm::book

2026-11-19 UTC. Lane: cloud, sonnet, branch `claude/wm1c` from 0ffc53a8, merged with the session branch (mv3 moves). Head 5e72e4f9 (gate ran on 1365525c; the head adds one kit test). Brief: briefs/wm1c-r1.md.

## Verdict
Built; cells red first, then green. Merge gate once on 1365525c (`--jobs 3`, run /tmp/merge-gate.lNgGz1): 35 steps PASS, `construct-hud` FAIL once at "dark-1280x720@1.5: no claim after the restart of voice" (a voice restart and the claim count; no window involved). The step alone on the same head: exit 0 (all ok, incl. "window.show made its pane"). `mc-paths` and `e2e`, which the gate never reached, run with `--only` on the same head (run /tmp/merge-gate.PiT0yT): PASS. So 38 of 38 steps pass, one of them only on rerun: treat `construct-hud` as a flake, 1 of 2 runs. Independent review still to come (rules 7, 8): MERGE-OK pending it.

## Commits (red first)
| commit | what |
|---|---|
| 62a6747d | test(construct): 4 cells in lifecycle.rs; 3 red (script: no `construct.window` phases; state: no `origin`; restart: no phases); `adopt_free` seam out of `start()` |
| 1365525c | feat(construct): lifecycle through the book; kit `wm::Created`; FakeCore answers an id query with the whole node; copies.toml; docs/api.md |
| 5e72e4f9 | test(kit): `Created` cell (coverage) |

## What changed in the construct
- Claim filter (`free = to none or NAME`) and the `Event::SurfaceCreated` arm are gone. `window_created`: the book claims (`claim`), `created` runs as before with `free` = "the book proposed it", and each proposal is answered by `owns` (an attached pane, a hidden window or the launcher window): `confirm` or `refused`.
- Detach and pull-in: `take_free` forgets the window in the book and claims it with the `to` that the core has; `take_detached` attaches it. An own window whose pane is still there but whose node lost its parent is now attached anew (before: ignored, the window stayed free).
- Destroy: the book tells `closed`, then the pane goes; a pane of a window that the construct only shows (overlay, taken) still goes.
- Give out: `Book::give`, then `confirm` (phase `given`) or `refused` (window and pane stay); a window that is no book window is refused.
- `window.show` of a free window and the adoption at the start claim through the book too.
- `construct.window {surface, node, phase, rect: null}` is emitted; `state.get` windows tell `origin` (`new`, `given`).
- Outputs reach the book (`filters` false: the windows stay when an output goes, as before).

## Cells (green on 1365525c)
| cell | where |
|---|---|
| create, refused attach, reserved for xp, detach, destroy, output removed, give in, refused give out, give out | `the_construct_runs_the_script_of_the_life_of_a_window` (same `Script::run` as the book and mc) |
| origin in state, no token | `the_state_tells_the_origin_of_a_window` |
| restart: free and reserved-for-construct windows adopted, xp's left, each opens once | `a_restart_adopts_the_free_windows_of_the_construct` |
| an overlay takes no free window | `an_overlay_takes_no_free_window` (guard) |
| views stay while windows come and go | `the_views_stay_while_windows_come_and_go` (guard) |
| `Created::of`, `label` | wm_book.rs |
chora-construct 194 tests, mc 139, tiling 152, chora-kit (testing) all pass. acts: PASS (known-red list unchanged by the gate step).

## Copies
`Event::SurfaceCreated outside kit` 2 to 1 (xp is the last); new row `construct: Event::SurfaceCreated (WM1c)` at max 0. cut-check ok, no FAIL.

## Coverage
`cargo llvm-cov -p chora-kit --features chora-kit/testing` (XDG_RUNTIME_DIR unset): kit::wm lines 98.4 % (book 98.13, give 100, kept 97.30, state 100; 5 of 313 lines missed). Before the `Created` cell book was 87.85 %.

## Where I read the brief narrowly (said plainly)
- "restore from meta written by 29e2cd77": the construct writes no window meta (as mc), so the restore cell is the start-up adoption of free windows.
- "face order": no cube in the fake core; the cell is a guard that views stay while windows come and go. The cube face order runs in the gate steps `cube`, `pull`, `out` (PASS).
- "its state adapter goes": `state.get` keeps its shape (acts unchanged) and gains `origin`; the owned set is still derived from panes and hidden windows, the origin from the book. A typed `kit::wm::State` for the construct would add keys; left for WM4.
- Test-only switch `Metrics::keep_estimates` (cfg(test)): the fake core lays out no text, so the text measure of a pane ate the refusal that the script plants.

## Refactor proposals, ranked
1. Book owns the pane list: `panes`, `hidden`, `launcher_window` beside the book; `owns` mirrors them. Key the construct's data by the book's ids in WM4.
2. `Change::Adopt` carries `app_id`, pid and token (rw-5 of wm1b): `Created` is a second way to say the same; then xp needs no `SurfaceCreated` match either (last copy row).
3. Make overlay vs flavor a policy of the book (a `Hold` per proposal) instead of "answer refused": `take_free` and `created` then share one settle.
4. FakeCore: answer `scene.query` by selector (kind, owner), so `start()` is a cell and `adopt_free` needs no hand-built nodes; give it text layout so `keep_estimates` goes.
5. `construct-hud` flake: "no claim after the restart of voice" failed once of two (10 s `until_ok`); look at the claim timing after `voice_start`.
