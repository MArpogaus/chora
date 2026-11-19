# wm7a r1 (F3): cube role API and the hosted xp session measured

Lane: opus high, cloud, branch `claude/wm7a` from a17e7858 (session branch unchanged during the round).
Commits: b7afaa13 `test(acts)` (red), fec3a98a `feat(construct)` (green), 90956a2b and 4069e38f
`refactor(construct)` (one `no_cube`; the role moves to `cube/role.rs`, as cube.rs is on the size ratchet).

## 1. Cube role API
- `chora_kit::cube` (crates/chora-kit/src/cube.rs): `LIST`/`TURN`/`PULL`, `Listing {faces[output, manager,
  health], front, turning, mode}`, `Turn {by | to, ms}` with the short-way `quarters_to` (construct's inline
  copy goes), `Pull {surface}`, event `<client>.cube {front, turning}`, `exporter()` by `methods.list`,
  `list()`, `turn()`. Unit tests: short way, by-or-to, unknown fields, exporter, names.
- construct exports the role (plugins/chora-construct/src/cube/role.rs): `cube.list`; `cube.turn` takes `to` and answers whether the cube took the turn;
  `cube.pull` makes a window of a face a pane without a flight (shares `pane_of` with `window.show`); every
  change of front or turning is emitted once (from `frame`). "Core IPC": the role is a method namespace (D71);
  the core needed no change. Docs: api.md (Plugin methods), configuration.md rows, glossary "Cube role".
- One cube code: construct hosts it; `xp/cube.rs` untouched (WM7b).

## 2. Acts (tests/acts/looks/construct.py, hosted form)
Red on a17e7858 (b7afaa13): every cube act "no client exports cube.list"; `cube.turn` refused `to`.
Green on fec3a98a: `acts: 19 pass, 2 known red, 0 blocking`: cube opens (4 faces, use), a window on the
front face (owner xp), turn key, `cube.turn {to}`, Alt+Escape stands back, drag turns, back at 1:1 the screen
equals the front face pixel for pixel (after the 3 s step-back hint), construct killed mid-turn and started
again loses no window (I1). Known red, slice WM7b: wm7a-a1 I3 (construct and xp both claim the focus when the
cube opens, no window focused); wm7a-a2 the pager (xp shows no pager on a face, so it cannot call `{to}`).
Harness: `Expect.cube`, `Expect.verify`, `Session.call`/`cube`, looks get `target/debug` on PATH.

## 3. Measurements of the hosted session
Build fec3a98a, debug, headless 1280x800, 4 vCPU container. Scripts:
`measure.py`, `measure2.py`, `shots.py` (harness sessions) and the captures are in media/2026-11-19/wm7a/.

| what | number | how |
|---|---|---|
| frame cost at rest, use mode | 0 damaged frames/s, 0 passes on HEADLESS-1 and VIRTUAL-1 (flat construct: 1.2 damaged/s) | `render.stats` over 5 s |
| at rest, stood back | 0 damaged frames/s, 0 passes (3 outputs) | same |
| direct scanout | not measurable headless; structural: the screen always shows the face plane composited by the core | code |
| a turn | 29 frames/s on HEADLESS-1 (VIRTUAL-1 20, VIRTUAL-2 29), debug build | `render.stats` over 1 s of a 3 s turn |
| keys, pointer on the face at 1:1 | xp grab Super+d acts; typed text reaches the window | inject, xp `state.get` |
| keys, pointer outside the claim | Super+d swallowed stood back, both on the cube and beside it; by input.rs:480-499 also during a turn (faces forward nothing) | inject |
| the turn grab | Super+drag on the face at 1:1 goes to construct's own grab (its pane move, camera.rs:473): 0 presses reach the window, the cube does not turn. On a face the claimant under the pointer is xp, so if the face manager also grabs the button (xp wm.rs:219 off virtual outputs today; tiling's move), ipc.rs:432-448 gives it the press, not construct | inject, render of grabs |
| construct killed mid-turn | xp leaves the core at 0.0 s (process gone 1.9 s); a restarted construct (3.0 s) removes the virtual outputs at 3.15 s; the window is back, as a construct pane, at 8.14 s. Without a restart the outputs and the window stay ~10 s, then the window lies ownerless at root | SIGKILL, poll 0.1 s |
| return to 1:1 | the step-back hint chip (184x44 px, 7833 px) shows 3 s (hints.rs:32), then 0 px differ from the face output | capture diff |

Captures: wm7a-fec3a98a-hosted-rest-1to1.png, -hosted-turn-third.png, -hosted-stood-back.png,
-killed-mid-turn-restarted.png. The last shows a finding (wm7a-m1): the restarted construct holds the adopted
pane but its start card still says "Nothing is open yet" at that moment.

Grab answer: the smallest-claim rule does not give construct the press when the face manager grabs the same
button. Hosted mouse turning needs no core slice as long as construct alone holds the turn binding on faces;
if the face manager keeps it, the core slice is "grab hand-over without a window" (not built).

## 4. Recommendation for the owner (1.2)
Hosted, with three conditions for WM7b: (a) construct alone holds the mouse-turn binding on faces (no core
slice); (b) I3 and the pager on faces (wm7a-a1, a2); (c) a construct restart reopens the cube and gives the
windows back to the faces' xp instead of panes, since today a crash costs about 8 s and the desks. At rest the
hosted cube draws nothing, keys work at 1:1, the return to 1:1 is exact. Flat stays the fallback without
construct. Open: direct scanout on hardware (not measurable here).

## Gate
tests/merge-gate.sh on 4069e38f: every step PASS (41 steps, acts 76 s, cube 104 s, e2e 293 s), log /tmp/merge-gate.8F7DaV. The first run on 90956a2b failed ci-check: size-ratchet, construct cube.rs grew 1641 to 1723; fixed by 4069e38f.

## Refactor proposals
- xp wm/desktops.rs:37-44 `turn_to` repeats `chora_kit::cube::quarters_to` (S; WM7b).
- construct cube.rs is 1,641 production lines on the ratchet; the start and health of the manager
  (launch, check_start, restart, Tail) would fit `cube/start.rs` beside `cube/role.rs` (M).
- tests/acts: a look that starts another look (xp by construct) edits `Session.looks` by hand; a
  `Session.adopt(look)` that waits for its `state.get` would serve WM7b and mc panes (S).
- construct `pid` of a client: clients.list has no pid, so acts read /proc like the gates (gate-construct-
  cube.sh:105); a `pid` in clients.list would end both copies (S, core).
Cut: no new breach (construct and kit only).
