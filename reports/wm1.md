# WM1 r1: kit::wm book, give, kept, state; pilot tiling

2026-11-18 07:28 UTC.

Lane: cloud, opus, branch `claude/wm1` from de057ab3. Head 87221071. Brief: briefs/wm1-r1.md.

## Verdict
Built and ported; the cells are red first, then green. Merge gate on 87221071: every step passes except
e2e `gate-tiling-apps-click`. That cell needs fuzzel 1.11.1, the container has apt fuzzel 1.9.2, and
setup.sh cannot fetch 1.11.1 because codeberg.org answers 502 through the proxy (network policy). The base
tiling binary (de057ab3) fails the cell 3 of 3 here too, so the failure is not caused by the port.
BLOCKED on the environment, not MERGE-OK by the gate. Needs owner: allow codeberg.org (or run the gate in a
container with fuzzel 1.11.1). The independent review is still to come (rules 7, 8).

## Commits (red first)
| commit | what |
|---|---|
| d8e15bc4 | test(kit): FakeCore script of `kit::wm::book`, red (11 of 12 cells fail on a typed skeleton) |
| 31fa4616 | feat(kit): `kit::wm::{book, give, kept, state}`; script green (fixtures: detach in the fake core, receiver claim) |
| 0623d5ae | test(tiling): port cells, red (4 red; the 29e2cd77 restore cell green as a guard) |
| 7e30f5e9 | feat(tiling): lifecycle through the book; copies.toml rows; docs/api.md |
| b6172e64 | test(kit): `Book::hears`, forget once, released |
| 87221071 | test(battle): W1/W12 read `kept.1.window`; docs/configuration.md names the keys |

## What is built
- `kit::wm::book`: claim filter (`to` of another manager, own, pending), adopt as a proposal the look
  settles (`confirm` / `refused`), `Forget {why: Destroyed | Detached | Given(to) | Released}`, outputs of
  `--output` with release of every window when the last goes, pass-on of a reserved surface while released,
  `Origin {New{token}, Given, Kept}`, `Phase {Opened, Closed, Given}` as `<look>.window {surface, node,
  phase, rect}` (phase before forget, so the rect is still known), D58 pass-through of foreign opacity,
  effects and meta on the look root, the window groups and the windows (kept and legacy keys stripped).
- `kit::wm::give`: give out as a proposal (`Book::give`, then confirm forgets / refused keeps), `pass_on`.
- `kit::wm::kept`: one versioned key `kept.1.<sub>`; reader takes `kept.1.*` if present, else the look's
  legacy keys (tiling: `window`, `current`, `tags:<output>`); write retires the legacy keys (empty value),
  so a node's meta stays inside 32 keys / 4 KiB.
- `kit::wm::state`: typed `State {windows, desks, focus, open, busy, more}`; window `origin` as a word
  (the activation token stays out).
- Policy: none in WM1. The look settles each proposal itself; a `Policy` trait comes with focus (WM2),
  where the first decision (focus after close) needs it. Simpler than 2.2.
- Out (later slices, as the brief says): focus, open, desks, stack.

## Tiling port
Its claim filter (`to != NAME`, `Event::SurfaceCreated` arms), `give_all_away`, `output_removed`,
`pass_on`, `looked` (D58 for windows only) and the restore meta parsing went; `lifecycle.rs` settles the
book. A detached own window leaves only the model and is adopted anew in the same frame, as before.
`state.get` is built from `kit::wm::state`; new key `origin`; `rect` numbers are floats now (same values).

## Cells (all green on 87221071)
| cell | file |
|---|---|
| create, refused attach, reserved for another, detach, destroy, last output removed (release, pass-on, back), give in, refused give out, D58 root/group/window, phase event, hears, forget once | crates/chora-kit/tests/wm_book.rs (14) |
| kept reader with today's keys, write retires them | crates/chora-kit/tests/wm_book.rs `kept::` (2, counted above) |
| state shape, origin without token | crates/chora-kit/src/wm/state.rs (2) |
| tiling: opens/closes tells phases; D58 on root, group, window; meta under kept.1; restore from 29e2cd77 keys; restore from kept.1 | plugins/chora-tiling/src/lifecycle.rs (5) |
chora-tiling: 151 tests pass; chora-kit with `testing`: all pass.

## Gates
Gate 1 (b6172e64): every step PASS up to e2e; e2e FAIL at gate-0.7-10, because W1 read tiling's old
meta key `window`. Fixed in 87221071 (test(battle) commit: W1 and W12 read `kept.1.window`, docs updated);
gate-0.7-10 then passes alone.
Gate 2 (87221071, `tests/merge-gate.sh --jobs 3`, run /tmp/merge-gate.2bkV9s): red-first, pre-commit,
gate-lib, build, test, clippy, ci-check, 2d-guard, package-deps, package-lists, the xp steps, hud, matrix,
virtual, layout, tiling-audit, tiling-look, chat-keys, doors, acts, last-words, cube, pull, placed,
construct-hud, out, winit, wm-pane, pane-keys, walk, keys, pixels and the mc steps all PASS. e2e FAIL at
script 24 of 29, gate-tiling-apps-click (environment, see Verdict). Scripts 25-29 run by hand on 87221071:
bar-early-signal ok, xp-settings ok, screencast, x11 and xp-token SKIP (all in e2e-skips.txt).
acts: 47 pass, 5 known red (unchanged, none of them tiling), 0 blocking; all 12 tiling cells pass (I1, I3,
I4, I7, I8, I14, window open/close, key list doors). The known-red list did not grow.

## Copies (tools/copies.toml)
`Event::SurfaceCreated outside kit` new, max 4 (mc 2, construct 1, xp 1); `to != NAME` 3 to 2; tiling rows
for both at max 0 (blocking); `UnixListener::bind in plugin tests` 7 to 6 (the count it had).
cut-check: no FAIL, no REPORT.

## Open, said plainly
- Coverage of `kit::wm` (>= 80 % lines): NOT MEASURED, blocked: no cargo-llvm-cov in the container, and
  /usr/bin/llvm-profdata is LLVM 18 while rustc 1.99 writes raw profile version 11 (LLVM 23). Lanes do not
  install tools (conventions 9). Every public function of wm/ has a cell; untested branches I know of:
  `kept::read` skips a node without id, `pass_through` drops args that are no `PropsUpdate`. Needs the CI
  coverage job or `rustup component add llvm-tools` plus cargo-llvm-cov in setup.sh.
- Tour v8 baseline not re-run (no tour tooling on this branch); the gate steps tiling-audit, tiling-look,
  chat-keys, doors, acts, matrix, virtual and wm-pane stand in.
- The pass-through now also strips `current` and `tags:*` from a foreign meta update (before: only
  `window`), on every node of tiling.

## Refactor proposals, ranked
1. Book owns the window list: tiling keeps `wm.windows` beside the book, and tests add windows to `wm`
   only (one test needed `book.keep`). A `Tiling::add` test helper through the book now; in WM4 the look
   keys its data by the book's ids.
2. Move the mini-look of wm_book.rs (settle loop over FakeCore) to `kit::testing::Script`, with
   `testing::output()` and `testing::surface()`; WM1b-d then reuse the same script per look (brief cells).
3. WM1b-d order: mc first (its detach policy puts the window back, the one look-policy hook the book
   leaves open), then xp, construct; each lowers `Event::SurfaceCreated outside kit` by its count.
4. A `Policy` trait only at WM2 (focus_after); keep proposals for lifecycle.
5. kept: retire `tags:<output>` of outputs that are gone (today they stay, as before the port).
