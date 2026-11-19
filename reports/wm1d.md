# WM1d r1: xp through kit::wm::book (F2, F3), wm1c rw-1 and rw-2

2026-11-19 UTC. Lane: cloud, sonnet, branch `claude/wm1d` from 1d1aa5fd (train 58 had not landed; no merge). Head 18231aa1. Brief: briefs/wm1d-r1.md.

## Verdict
Built; cells red first, then green. Merge gate once on 18231aa1 (`--jobs 3`, run /tmp/merge-gate.3X1omQ): all 41 steps PASS, exit 0, no flake this time (construct-hud, e2e, acts, doors, matrix, cube, tiling-look among them). acts: 47 pass, 5 known red (unchanged), 0 blocking. Independent review still to come (rules 7, 8): MERGE-OK pending it.

## Commits (red first)
| commit | what |
|---|---|
| ceb6a91d | test(kit): `Script::beside(other)` (the script hard-coded xp as the second manager), `FakeCore::selecting()` (scene.query answers kind:/owner: from the model; default stays id only, mc's tests depend on it) |
| 8d194c19 | test(xp): 4 cells in wm/lifecycle.rs, 3 red (script: no `xp.window` phases; state: no `origin`; restart: no phases); also carries the red cell of rw-1 (staged by an earlier failed hook run, not a separate commit) |
| 218c2a6b | fix(construct): adopt only a window that the book claimed (rw-1) |
| 18231aa1 | feat(xp): window lifecycle through the book; kit state Window gains optional title/app_id; tiling compile fix; docs/api.md; copies.toml |

## What changed in xp
- `takes()` and the `Event::SurfaceCreated` arm are gone (dispatch.rs). New wm/lifecycle.rs: `window_created` (book.claim), `window_left` (detach and destroy through `book.apply`), `settle` (Adopt -> `adopt` -> confirm or refused; Forget -> old `forget`; Phase -> `xp.window` with the frame while the window shows).
- `adopt` returns whether the window is xp's (false when the core refused). It no longer filters on `to`.
- `adopt_all` (start, restart) claims through the book with the node's `to`, so a free surface reserved for another manager is left (before it was passed `None`). Kept windows come back with origin `kept`; the taskbar order uses only windows the book took.
- Give out: `book.give`, `give_dragged`, `confirm` (phase `given`) or `refused`; give to xp itself keeps its INVALID_PARAMS error. `give::Windows::has` asks the book.
- state adapter: `state_value` is typed on `kit::wm::state`; windows tell `origin`; `title`, `app_id` and the `for` key of the system menu stay (kit Window got optional title/app_id, skipped when none).
- Restart meta (`xp` key, KeptWindow) is unchanged, so the meta of the 29e2cd77 binary reads as before.
- construct rw-1: `created` adopts by token or launch only when `free` (the book claimed it). rw-2: docs/api.md says `node` is the surface where a look has no groups (no code cell: a doc fix).

## Cells (green on 18231aa1)
| cell | where |
|---|---|
| create, refused attach, reserved for mc, detach, destroy, output removed, give in, refused give out, give out | `xp_runs_the_script_of_the_life_of_a_window` (same `Script::run`, `.beside("mc")`) |
| origin new/given in state, no token | `the_state_tells_the_origin_of_a_window` |
| restart from old meta: stacking order, frames, desktop, minimized, taskbar order | `a_restart_reads_the_meta_of_the_old_binary` (guard, green before) |
| restart: reserved-for-mc window left, each adopted window opens once, origin kept/new | `a_restart_opens_each_adopted_window_once` |
| rw-1 | construct `a_launch_adopts_no_window_reserved_for_another_manager` |
xp 197 tests, construct 199, tiling 152, mc 140, chora-kit (testing) all pass. acts unchanged (the gate step passes).

## Copies
`Event::SurfaceCreated outside kit` 1 to 0 (blocks); new row `xp: Event::SurfaceCreated (WM1d)` at 0; `to != NAME` stays 1 (mc acts.rs:31). cut-check ok, size-ratchet ok.

## Coverage
`cargo llvm-cov -p chora-kit --features chora-kit/testing` (XDG_RUNTIME_DIR unset): kit::wm lines 98.4 % (book 98.13, give 100, kept 97.30, state 100; 5 of 313 lines missed).

## Where I read the brief narrowly (said plainly)
- "restore from meta written by 29e2cd77": xp keeps its own `xp` meta key and format; the cell writes that JSON by hand into a kit FakeCore and starts xp on it. xp is not moved to `kit::wm::kept` (it would change the format).
- "taskbar order": read as `wm.opened` after a restart (no taskbar nodes on the fake core, which has no outputs).
- "output removed": xp keeps its windows (book `filters` false, as mc); xp's own bar logic is untouched and not fed to the book.
- No D58 pass-through through the book: xp keeps its own `requested_look` (it needs the float seed), so `with_groups` only names the node of `xp.window`.
- Red commit of rw-1 shares a commit with the xp cells (history only).

## Refactor proposals, ranked
1. `Change::Adopt` carries app_id, title, pid and token (wm1b rw-5, wm1c 2): xp builds a `Created` by hand in `adopt_all`, construct has the same; then `settle` needs no `facts`. S.
2. Book owns the window list: xp keeps `windows`, `opened`, `kept` beside the book (wm.rs:345,418; windows.rs `index`); key by the book's ids in WM4. M-L.
3. Kit `Settle` helper: mc, construct, tiling and xp each write the same `settle` match (windows.rs:37-60 mc, lifecycle.rs construct/tiling/xp); a trait `Look { adopt, forget, phase, apply }` with one `settle` in kit. M.
4. `kit::wm::state::Open` gets an optional `for` so xp's `name_the_window_of_the_system_menu` patch goes. S.
5. xp wm.rs is 1,400 lines with two fake-core helpers; move `wm_on_fake_core` onto `FakeCore::selecting` and delete the hand-rolled socket (wm.rs ~1020). M.
6. FakeCore gives outputs (`output.list` answers `[]`): an xp output cell (bars, taskbar order buttons) needs them. S.

Cut check: xp uses chora-kit, chora-scene, chora-draw only (dev-dependency chora-kit with `testing`); no breach added.

## Review r1

Head 18231aa1, base 1d1aa5fd. Verdict: **MERGE-OK** (0 BLOCK, 0 MAJOR, 2 MINOR, 0 NIT).

Checked, each green: (1) `cargo test -p chora-xp` 197 pass; lifecycle (create, attach, detach, destroy, give in/out) runs through `kit::wm::book` (`lifecycle.rs`, `dispatch.rs`, `windows.rs`); `takes()` and the `SurfaceCreated` arm are gone; the state adapter is typed on `kit::wm::state`. (2) 8d194c19 precedes 218c2a6b and 18231aa1; run on 8d194c19 only, `a_launch_adopts_no_window_reserved_for_another_manager` fails (rw-1 red confirmed). (3) Restart cells read the 29e2cd77 meta by hand (stacking, frames, desktop, minimized, taskbar order); each adopted window opens once; the window reserved for mc is left. (4) `cargo test -p chora-construct` 199 pass; `created` adopts only when the book claimed (`free`); no window is lost when `free` is false (it is reserved for another manager, or already the construct's); rw-2 in docs/api.md matches the code (`with_groups` only in tiling and xp). (5) `tools/cut-check` ok; `Event::SurfaceCreated` outside kit is 0 by grep; `tools/size-ratchet` ok. (6) Visible behaviour as stated, but see rw-1, rw-2.

Findings:
- rw-1, MINOR, plugins/chora-xp/src/wm/state.rs:28,30: `state.get` of xp changed shape: `rect` ints now print as floats (300 to 300.0) and `output` is "" where it was null (a window under no bar). Fix: accept as the kit shape and say so in docs/api.md, or keep `output` an `Option` in `kit::wm::state::Window`.
- rw-2, MINOR, plugins/chora-construct/src/adopt.rs:223: `self.launched.take()` (and `waiting.remove`) still run before the `free` test, so a window reserved for another manager that arrives within LAUNCHED of a launch eats the launch marker; the real program window is then adopted without `force` and without the fly-in. Fix: take `launched` only when `free`.

## r2

Head `679e7786` on claude/wm1d (merge of net3 dc00d33c, train 59 not landed yet, so no merge of the session branch).
- plugins/chora-xp/src/wm/state.rs: conflict resolved by keeping wm1d's typed `kit::wm::state` port and adding `texts: self.texts_value()` (the runs of `theme.texts()` as json); module doc says texts are in; net3's `texts`-is-an-array assertion in `a_fresh_xp_has_an_empty_state` kept.
- crates/chora-kit/src/wm/state.rs: `State` gets `texts: Vec<Value>`, left out when empty (test `texts_are_left_out_when_empty`); not a json side path.
- docs/api.md: merged without conflict; net3's paragraph (xp and chora-notify add `texts`, `shown`, `busy`), wm1d's "Every look adds the `origin`" and xp in the `<look>.window` list are all there.
- Cell that broke in the merge: plugins/chora-tiling/src/state.rs built `State` without `texts`; fixed with `..State::default()` and a small `open_value` helper (the function hit clippy too_many_lines at 61/60).
- rw-1 not touched (the merge did not touch those lines).
- Green: `cargo test -p chora-xp` (198), `-p chora-kit --features testing` (225+), `-p chora-notify` (18), `-p chora-tiling` (152); clippy workspace `-D warnings`; `tests/merge-gate.sh --only xp,xp-fields,xp-question,xp-notepad,hud,acts` all PASS. No full gate.
- Note for the lane: `--only` skips the `build` step, so run `cargo build --workspace` first (the first `xp` run failed only for missing binaries).

## r3

Head `3c84b2d1` on claude/wm1d (merge of claude/wm2 8db02220 into 679e7786; pushed). Train 60 had not landed (wm2 is not in claude/autonomous-dev-session-3w-i33pg7), so the second merge of the session branch is still to do.
- plugins/chora-tiling/src/state.rs: conflict resolved to one focus path: `let mut focus = self.focus.state(NAME, |s| self.wm.has(s));` with the holder set while `keys_open`; the hand-built `Focus` and its import are gone. The keys sheet stays the `Open` of wm1d's `open_value` helper (wm2's inline copy dropped); `texts` stay through `..State::default()`; `Map` import dropped (wm1d's `more` literal).
- docs/api.md: two conflicts, both texts kept: wm2's holder sentence (own nodes, `focus.surface`) plus wm1d's `title`/`app_id` for xp; wm1d's `<look>.window` paragraph (xp in the list, node is the surface, xp tells no rect yet) with wm2's "Such a look applies a foreign update".
- tools/copies.toml, tests/acts/known-red.toml: auto-merged. No cell broke in the merge.
- Green: `cargo build --workspace`; `cargo test -p chora-tiling` (153), `-p chora-xp` (198), `-p chora-kit --features testing` (225+), `-p chora input::` (47); `tests/merge-gate.sh --only keys,pane-keys,wm-pane,acts,xp,hud,tiling-audit` all PASS (run dir /tmp/merge-gate.uxvYko). No full gate.
- Note for the lane: the local `claude/wm1d` of a fresh clone is stale (18231aa1, before the net3 merge); a first merge onto it built fine but state.rs did not conflict, which showed the wrong base. Reset to origin/claude/wm1d first (I did, and redid the merge; the stale merge was never pushed).
