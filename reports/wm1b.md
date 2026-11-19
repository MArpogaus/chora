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

## Review r1

2026-11-19 UTC. Independent review (sonnet, high) of claude/wm1b at 0ffc53a8 against 27bd60c7. Brief: briefs/review-wm1b-r1.md.

### Verdict
MERGE-OK (computed: 0 BLOCK, 0 MAJOR open; 3 MINOR, 4 NIT). The merge gate of the builder stands (not re-run here).

### Checks
1. Lifecycle only through `kit::wm::book`: yes. grep in plugins/chora-mc/src: no `SurfaceCreated`, no `to != NAME` in a claim filter (only acts.rs:31, the give-to-self guard), no known-surface check, no `reattach`, no state adapter of its own. `adopt` (windows.rs:112) and `forget` (windows.rs:249) are reached only from `settle`/`take` (windows.rs:56, 103); the other callers are tests (manage.rs:542-584). `Event::SurfaceDestroyed` appears only in a test (windows.rs:421).
2. Red first: yes. Log order 47d9dbb (kit Script), 8d492e8 (mc cells), 0ffc53a (port). Ran `cargo test -p chora-mc` at 8d492e8: 2 FAILED (`mc_runs_the_script_of_the_life_of_a_window`, panics at script.rs:81, no `mc.window` phase; `the_state_tells_the_origin_of_a_window`), 137 pass. The restart and scene-order cells were green as guards, as the report says.
3. Behaviour unchanged: detach goes through `Book::apply` (leave + claim), then `take` sees `holds` and calls `put_back` (windows.rs:84-103, 228); the script cell asserts the window is mc's and owned again. Give out is a proposal (manage.rs:160-175: `book.give`, `confirm` or `refused`, then `gave` with the old cleanup). Restore: `adopt_free` takes free surfaces and those reserved for mc, leaves xp's (cell `a_restart_adopts_the_free_windows_of_mc`); mc wrote no window meta in 29e2cd77, so there is none to read.
4. copies: true. `tools/cut-check` ok, no FAIL. `Event::SurfaceCreated` outside kit: construct plugin.rs:27, xp dispatch.rs:20 = 2 (max 2); mc row max 0, count 0; `to != NAME`: acts.rs:31 = 1 (max 1).
5. `kit::testing::Script` is generic (trait `Subject`: hear, has, give; `releasing()` for `--output` looks) and is run by `wm_book.rs::the_script_runs_for_a_book` and by mc (`windows.rs::mc_runs_the_script_of_the_life_of_a_window`). The old mini-look helpers moved out of wm_book.rs.
6. Tests run: `cargo test -p chora-kit --features testing` all pass (223 lib + wm_book and the rest, exit 0); `cargo test -p chora-mc` 139 pass. Commits: owner identity, no attribution lines.

### Findings
- rw-1 (MINOR) plugins/chora-mc/src/windows.rs:228 `put_back`: the dock and fitting branch (a detached docked window goes back to PARKED, `rendered` reset) has no cell; the script drives a scene window only. Fix: add a cell that detaches a docked window and asserts it is parked and `rendered` is reset.
- rw-2 (MINOR) plugins/chora-mc/src/windows.rs:253 and lifecycle.rs:160: the restart cell calls `adopt_free` with hand-built nodes, so `build` -> `scene.query kind:surface` -> `adopt_free` is not run, and it reads no meta. Fix: let the cell run `build` against the FakeCore with free surfaces in the model (the brief's "restore" cell).
- rw-3 (MINOR) plugins/chora-mc/src/manage.rs:160: the result of `self.book.give` is dropped; if the book does not know the surface, `confirm` returns nothing and no `given` phase is told while the window is given. Fix: when it returns false, return the reject error before `give_dragged`, or tell the phase anyway.
- rw-4 (NIT) plugins/chora-mc/src/windows.rs:37-41: `Origin::Given | Kept` drops the event token, so a window reserved for mc that also carries a token mc issued would no longer find its `starting` target (the old code passed the token). No such flow exists today. Fix: pass the token in `Origin::Given` too, or note the rule in the doc comment.
- rw-5 (NIT) plugins/chora-mc/src/windows.rs:20 and crates/chora-kit/src/wm/book.rs:275: `Book::announced_app` returning `Option<Option<&str>>` plus the `Named` enum is two ways to say "the app id". Fix: let `Change::Adopt` carry the `app_id` the book heard (and `claim` take it), then drop `Named` and `announced_app`.
- rw-6 (NIT) plugins/chora-mc/src/main.rs:624: `lifecycle` runs for every `output.changed` and `output.removed` although the book has `filters` false (its outputs are never read); only surface events and the `node.requested` update (not subscribed) matter. Fix: test `matches!(event, SurfaceCreated | SurfaceDetached | SurfaceDestroyed)` or give the book an output-less constructor.
- rw-7 (NIT) plugins/chora-mc/src/acts.rs:31: `to != NAME` could be `Book::give`'s own-name check (the builder's proposal 4). Fix: call `self.give` and let the book refuse; then the `to != NAME` row is 0.

### Refactor ideas (rank)
1. rw-5 (carry app_id on the proposal) in WM1c/d: xp and construct need the same `Named` otherwise.
2. Book owns the window list (builder proposals 1 and 3), keyed by the book's ids in WM4.
3. `Script` gets a dock cell (`Subject::dock`) once xp's regions run it.
