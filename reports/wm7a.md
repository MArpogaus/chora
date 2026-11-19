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
equals the front face (after the 3 s step-back hint; tolerance: a channel may differ by 8, in at most 1/1000 of the pixels), construct killed mid-turn and started
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
| keys, pointer outside the claim | Super+d swallowed stood back, both on the cube and beside it; by input.rs:480-499 also during a turn (code reading only, see r2 for the measured cell) | inject |
| the turn grab | Super+drag on the face at 1:1 goes to construct's own grab (its pane move, camera.rs:473): 0 presses reach the window, the cube does not turn. On a face the claimant under the pointer is xp, so if the face manager also grabs the button (xp wm.rs:219 off virtual outputs today; tiling's move), ipc.rs:432-448 gives it the press, not construct | inject, render of grabs |
| construct killed mid-turn | xp leaves the core at 0.0 s (process gone 1.9 s); a restarted construct (3.0 s) removes the virtual outputs at 3.15 s; the window is back, as a construct pane, at 8.14 s. Without a restart the outputs and the window stay ~10 s, then the window lies ownerless at root (finding wm7a-m2, I1, slice WM7b; r2 does not fix it) | SIGKILL, poll 0.1 s |
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

## Review r1
Lane: opus high, cloud, read-only on claude/wm7a 4069e38f (base a17e7858). Verdict: **FIX** (1 MAJOR, 7 MINOR, 0 BLOCK).
Checks: (1) red first holds by order: b7afaa13 touches only tests and fails with "no client exports cube.list"
(read, not run); but the green commit also rewrote the 1:1 act (node check to pixel check plus a 4 s wait).
(2) API small and typed, one cube code in construct, no core change, events `construct.cube` pass the core's
`<client>.NAME` rule (ipc.rs:1052), `methods.list` shape matches `exporter_in`, xp/cube.rs untouched;
`cargo test -p chora-kit --features testing --lib cube` 5 pass, `-p chora-construct` 191 pass.
(3) I1 mid-turn: the act does not kill mid-turn (rw-1); the report's 8.14 s give-back is measured (measure.json
kill_marks). (4) Docs match names and fields, gaps rw-4. (5) `tools/size-ratchet --base a17e7858` ok,
cube.rs 1641 production lines as listed, role.rs split true; `tools/cut-check` ok (8 crates, 18 reported);
copies.toml has no row for xp's `turn_to` copy (refactor proposal, WM7b). (6) Numbers in the table match
measure.json; gaps rw-6, rw-7.

- rw-1, MAJOR, tests/acts/looks/construct.py:197, the kill act's first cue `cube.turn {to: 0}` starts a 360 ms
  turn, so `kill_construct_mid_turn`'s `cube.turn {by: 1, ms: 3000}` (line 131) is ignored ("ignored while a turn
  runs", answer unchecked) and the SIGKILL lands about 1 s later at rest: I1 mid-turn is not guarded by the act;
  fix: drop the `{to: 0}` cue (or wait for `turning: false`), assert the 3000 ms turn answered true and
  `cube.list` says `turning: true` before the kill.
- rw-2, MINOR, plugins/chora-construct/src/list.rs:102, `pull` spells its own "no cube is open" refusal beside
  `modes::no_cube()` (modes.rs:148), against 90956a2b's one refusal; fix: return `crate::modes::no_cube()`.
- rw-3, MINOR, plugins/chora-construct/src/list.rs:97, `cube.pull` has no unit test and no act (made pane,
  refusal for a window off a face, give-back to the face manager when the pane goes, as configuration.md:1066
  claims); fix: one act that pulls the probe window and removes the pane, expecting owner xp again.
- rw-4, MINOR, docs/api.md:1533 and docs/configuration.md:1064-1066, the answer `{pane}` of `cube.pull` is not
  documented, and the `cube.turn` row lacks the full stop before "Answers whether the cube took the turn"
  (a turn to the front face answers false); fix: name the answer and the stop in both files.
- rw-5, MINOR, tests/acts/looks/construct.py:182-196, the acts after the known-red pager act expect the front
  the broken pager leaves (0, then 1), so the fix of wm7a-a2 in WM7b turns three green acts red; fix: start the
  step-back act with `cube.turn {to: 0}` (or expect relative to the front read before).
- rw-6, MINOR, reports/wm7a.md section 3, measure.json `turn_key_to_window = 2` (two key lines reached the probe
  window on Ctrl+Alt+Right at 1:1) is not reported, and "also during a turn" for keys is code reading
  (input.rs:480-499), not a cell (rules.md 7); fix: name what the two lines are (modifiers, or the Right key, a
  stray key) and measure keys during a turn, or mark that row open.
- rw-7, MINOR, reports/wm7a.md section 3, the kill row's "without a restart the window lies ownerless at root"
  is an I1 case with no finding id or slice, and "pixel for pixel" at 1:1 overstates the act (it passes up to
  1/1000 of the pixels); fix: file it as wm7a-m2 for WM7b and state the act's tolerance.
- rw-8, MINOR, plugins/chora-construct/src/cube/role.rs:86, `construct.cube` is told on open and on each change
  but never when the cube closes or its manager ends (the `told` state goes with the host), so a look that
  follows the event keeps `turning: true` after a close mid-turn; fix: emit a last event on close or document
  that the end of the cube is `cube.list` with no faces.

## r2
Lane: sonnet high, cloud, branch claude/wm7a 066ae80d (base 1d1aa5fd merged as b2a77ff0). Gate: `cargo test -p chora-construct` 200 pass,
`-p chora-kit --features testing` green; `tests/merge-gate.sh --only acts,cube,pull,construct-hud,placed` all PASS (no full gate).
- rw-1 fixed (c3721e75): the kill act drops the `{to: 0}` cue, asserts the 3000 ms `cube.turn` answers true and `cube.list` says
  `turning: true` before the SIGKILL. It was green at once, so construct needs no fix: I1 holds mid-turn (window given back, no loss).
- rw-5 (eba00709): the step-back act starts with `cube.turn {to: 0}` and a 0.8 s wait, so no act depends on the front the known-red pager leaves.
- rw-3, a real defect (eba00709 red, f644d056 fix): the new act pulls the probe window, checks the pane in `construct.list`, a second pull is
  refused, removes the pane and expects owner xp again. Red: `cube.pull` refused every window ("no window s2 on a face of the cube") because
  `Host::manager` and `cube.list` answered the display name `XP` where windows are owned by the client `xp` (the docs say "client name" for `manager`).
  The unit test had hidden it by setting the name to "xp"; it now uses "XP". Fix: both use `cube::MANAGER`.
- rw-2 (f644d056): `pull` returns `crate::modes::no_cube()`.
- rw-4 (066ae80d): api.md and configuration.md name `{pane}` of `cube.pull` and the full stop of `cube.turn` (`false` when ignored: a turn runs, or `to` is the front face).
- rw-6, measured once on this build (headless, probe window on the face): Ctrl+Alt+Right reaches the probe window as `key 29` and `key 56`, the two
  modifier presses (Ctrl_L, Alt_L evdev codes); the Right key is not forwarded. Keys during a turn (0.5 s into a 3000 ms turn, `turning: true`):
  a typed "a" reached the window (`key 30`), Super+d did not. So "faces forward nothing" during a turn is wrong for plain keys; the row now says code
  reading only, and this cell stands as one run, not an act.
- rw-7: the 1:1 act tolerates a channel difference of 8 in at most 1/1000 of the pixels (stated in section 2); the ownerless window is filed as wm7a-m2, rw-8 as wm7a-m3, both WM7b, not fixed here.
