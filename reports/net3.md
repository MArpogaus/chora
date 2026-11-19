# net3 r1 (F7, F5 prep): core scene.boxes and text runs

Branch claude/net3 from dc4bc46d, head bc54896a (pushed), 10 commits:
d5d1f8a4 test(core) red, 01b874af feat(core) scene.boxes, 10455167 feat(draw) runs,
ef90a629 test(xp) red, 318a80d8 feat(xp) texts, c5ad240e feat(notify) state.get,
619cd0e0 test(acts) box invariants, 21d718f1 test(acts) cube acts + I7 fix + net3-t1, 30fa671d ci,
bc54896a test(acts) known red net3-m1.
Gate `tests/merge-gate.sh --jobs 3` on bc54896a (06:40-07:2x UTC): every step PASS, GATE_EXIT=0, log
/tmp/claude-0/gate/net3b.log, run dir /tmp/merge-gate.NVKDxi (container-local); acts 75 pass, 7 known red, 0 blocking.
It started 4 s before the commit of bc54896a: same tree, red-first and pre-commit ranged to 30fa671d (the commit's
hooks passed). The first run on 30fa671d failed only at acts: I5 found net3-m1 (below).

## Done
- Core `scene.boxes` (chora-scene boxes.rs: types, pure `opaque` and `corners`; core ipc/boxes.rs): per output,
  bottom first (layer surfaces background/bottom, scene through the camera, overlay, top/shell_overlay), each box
  with node, kind, owner, layer, the rect of scene.query, `opaque`; `corners` for planes and nodes of a 3D context;
  `text`, `font_size` for text nodes. Read-only, no write path; a client socket boxes only its own nodes.
  Red d5d1f8a4: 8 cells "no method scene.boxes"; green 01b874af (8 core + 6 chora-scene cells). docs/api.md "Boxes".
- Text runs (chora-draw): `Fonts::run` is the line as drawn (one shared placement with `draw`: ellipsis, centring),
  its ink box in picture pixels, px, fg; `Theme::texts` by node from `size_images`. Font checks: Tahoma body line
  has px 14, one line of ink inside its rect, width = drawn width; a cut line ends in an ellipsis inside its rect.
  xp (red ef90a629: "texts" missing) and chora-notify (`shown`, `texts`, `busy`; red = compile error) report them.
- tests/acts: Snapshot.boxes (scene.boxes) and blocks (notify state.get); I8 reads boxes instead of scene.query rects.
  New I2, I5, I6, I9, I10, I12, I13. Selftest: 27 new invariant cells (sound + planted breach each) and a missing-key cell for I9; stubbing an invariant to
  pass turns its cells red: I2 2, I5 2, I6 2, I9 4, I10 2, I12 3, I13 2, I8 4. Live selftest in mc: planted overprint
  (I5) and raw entry (I10) fail, sound cells pass. Construct acts: cube.open with xp on four faces, step back (I13).
- Harness fix found on the way: I7 cropped `work_area` (scene coordinates) as output pixels, so every virtual output
  counted one colour; `work_rect` subtracts the output position (cell WorkRect).
- Known red: +2 rows, 7 in all, each a finding seen on the still: net3-t1 tiling I5, the key list runs "Run a
  program or a typed command" into the heading "System" of the next column; net3-m1 mc I5, the wrapped line of
  "Start a program or run a command" overprints the next row "Chat with a helper". Both slice KIT3.

## Deviations, open
- I13 is not red: at rest the stood-back faces 0 and 1 share their edge to 0.1 px ((791.9, 254.9)-(791.9, 545.1)),
  so no co-21 row (a listed pass fails the gate). co-21 may be a drawing fault (depth order, mesh) or a pose of the
  demo (other face sizes); the held 45 degrees is not covered: 2 s into a 4 s `cube.turn by 1` only face 1 shows
  (face 0 hidden mid-turn), worth a look in WM6.
- I9 runs only where chora-notify runs; the act sessions have no session bus, so in the gate I9 is checked by its
  cells only. I12 has no `held` (WM5). I2 skips windows without `output` (construct panes). Images count as not
  opaque (a menu image does not hide a label for I5). mc, tiling and construct draw text nodes (boxes cover them);
  assistant has no state.get, so its runs are not seen.
- Red-first for draw and notify: tests in the feat commits (compile-red recorded), not separate red commits.

## Refactor proposals (ranked)
1. ipc/query.rs:62-96 `screen_rects` and ipc/boxes.rs:25-63 `output_boxes` walk outputs x (scene, overlay) with the
   same camera map; one `placed_on(output)` iterator for both (S).
2. Core image opacity: images.rs could note a fully opaque PNG at decode, so `opaque` covers panels and I5 can see
   a menu hide a label (S-M).
3. Theme.shown is fed only through `size_images`; image nodes applied straight through `rt.conn.apply` bypass it.
   One draw-aware apply in kit for xp, assistant, notify (xp wm/startup.rs:257, 270) (M).
4. Text drawn as `Layer::Svg` bodies is not reported; move those labels to `Layer::Text` (M).
5. output.list could carry the work area in output pixels; scripts crop captures by it (S).
6. `Fonts::width` and `Fonts::run` parse an SVG per line; cache by (line, font) as Theme.widths does (S).
Cut check: no new dependency; xp and notify use only chora-draw, kit and scene.

## Review r1

Seeded review of claude/net3 bc54896a against dc4bc46d (21 files). Verdict: **FIX** (1 MAJOR, 5 MINOR, 1 NIT, 1 pre-existing).
Ran: `cargo test -p chora-scene -p chora-draw -p chora-xp -p chora-notify` pass (34+18+129+195); `cargo test -p chora ipc::`
133/134: only rw-8, which fails the same way on dc4bc46d and passes alone. `tests/test_acts.py` 86 pass. Stubbed I2/I5/I6/I9/I10/I12/I13
one at a time: 2/2/2/4/2/3/2 cells go red, as the report says. `tools/cut-check` ok, no new dependency (serde_json is a
workspace dev-dependency of chora-draw). Check 1 holds: scene.boxes is read-only, a client socket boxes only `own_node` nodes and no
layer surface, the main, plugin and confined sockets follow `ipc.fixed` exactly as scene.query does (the test covers both sockets).
Check 2 holds: `Fonts::run` and `draw` share `placed` (fit, ellipsis, centring), and the font cells pass.
Check 4: net3-t1 and net3-m1 each name a real overprint (see rw-5 for how wide they match). I13 not red: not accepted, see rw-1.
Check 5: no change to looks; xp and notify only add `texts`/`state.get`.

- rw-1 MAJOR tests/acts/invariants.py:493 + looks/construct.py:34: the slice row says I13 holds "at rest and at a held 45 degrees".
  Only the rest pose is acted, and I13 checks only pairs where both faces show with corners. The lane's own probe found face 0 gone
  2 s into a 4 s `cube.turn by 1`, which is the co-21 gap I13 cannot see. So "I13 not red" rests on the one pose that passes.
  Fix: add a held-45° act (pause the turn halfway) and fail when a ring neighbour of a shown face is missing or has no corners, then list co-21 if it goes red.
- rw-2 MINOR tests/acts/harness.py:177: `work_rect` subtracts `position` from `work_area`, but `work_area` is
  `scene_area(camera, zone)` (output.rs:257). That is right for the default camera only, so the I7 crop moves once a look moves the
  camera (construct step back, mc pan). Fix: use `default_work_area` minus `position`, which is the zone and does not depend on the camera.
- rw-3 MINOR tests/acts/invariants.py:427: `bars_over_fullscreen` counts every box above the window that a look owns. Attached
  surfaces are owned by the WM (tree.rs attach), so a modal dialog or popup correctly above its fullscreen parent is flagged as a bar.
  That contradicts `modals_below`. Fix: skip `kind == "surface"` boxes and the look's `open` entries for that window.
- rw-4 MINOR tests/acts/invariants.py:299: `screen_texts` maps runs through `box["rect"]`, which is the query rect already cut by
  ancestor clips. For a partly clipped image node (sliding menu, scrolled list) the runs get squeezed into the visible part, which gives
  false I5 overlaps or misses. Fix: map runs through the unclipped placement (node rect before the clip, or `corners`) and clip afterwards.
- rw-5 MINOR tests/acts/known-red.toml:47,54: `match` is a node id, and harness.py:243 accepts it while every breach contains it.
  So any further overprint inside the tiling key list or the mc key column stays hidden behind net3-t1 and net3-m1. Fix: match on the
  named line ('Run a program or a typed command', 'Start a program or run a command').
- rw-6 MINOR tests/acts/invariants.py:372: I9 checks only the notices already in `shown`. A notice chora-notify accepted and never
  showed (the sh-18 shape, and the "accepted ... is in its shown" half of the row) passes. Fix: have notify report `accepted` ids for
  the act (or have the cue keep the Notify reply id) and require each one in `shown`.
- rw-7 NIT crates/chora-draw/src/theme.rs:559: `render` builds runs (one SVG parse per text layer) for every new picture in
  production, though only `state.get` reads them. Also `runs.clear()` on eviction keeps `shown` paths, so texts vanish silently in a long
  session. Fix: compute runs lazily in `texts()` (memoised per path) and drop `shown` entries whose path was evicted.
- rw-8 pre-existing crates/chora/src/ipc.rs:2156: `a_refused_request_logs_its_client_and_code_without_its_parameters` fails
  under `cargo test -p chora ipc::` on both dc4bc46d and bc54896a and passes alone. The thread-local subscriber loses to callsite
  interest that another test thread cached. Fix: log through a test-wide subscriber, or call `tracing::callsite::rebuild_interest_cache()` inside `logged`.

## r2

Branch claude/net3, bc54896a to dc00d33c (4 commits: red cells c716301d, notify dd827e9e, ipc test 2372edaa, fixes dc00d33c).
Red first: 10 selftest cells failed on bc54896a (one cell each for rw-2, rw-3 x2, rw-4 x2, rw-5 x2, rw-6, rw-1 x2), then green. Live act
"the cube is held halfway through a turn" (`cube.turn by 1, 4000 ms`, read 2 s in with the new `held` of `Session.act`).
- rw-1: I13 now also fails, in a held act, when a ring neighbour of a shown face has no box or no corners. It went red on the head:
  held, faces 1 and 2 show and 0 and 3 have no box. co-21 is listed (construct I13, match "is missing beside", slice WM6); the construct is untouched.
  Caveat for WM6: at rest the same rule is red too (faces 2 and 3 have no box when 0 and 1 show, they look hidden, not broken), so the rule
  runs only in held acts. A cube seen from outside hides its back faces, so "neighbour missing" may be right at 2 s; the pose maths decides.
- rw-2: `work_rect` uses `default_work_area` (no camera). rw-3: surface boxes and the look's `open` entries for the window are no bars.
- rw-4: runs go through the unclipped placement (ancestor matrices and the camera, plain scale and translation), cut to the box after;
  anything else falls back to the old mapping. rw-5: `match` is the named line. rw-6: notify `state.get` has `accepted`, `waiting`,
  `closed` (last 64 ids each); I9 needs a new accepted id in shown, waiting or closed.
- rw-8: a test-wide subscriber that routes each line to the sink of its thread (`rebuild_interest_cache` alone still failed 1 run in 3).
- rw-7 (NIT) not done: lazy `texts()` needs a RefCell in theme.rs, not small.
Done: `cargo test -p chora ipc::` 134/134 six times; scene, draw, xp, notify pass; `tests/test_acts.py` 101 OK;
`tests/merge-gate.sh --only acts,cube` acts PASS 63 s, cube PASS 102 s (first run FAIL only because pre-commit stashed the tree under it).

## PM check r2 (07:33 UTC; PM not the builder)
Diff bc54896a..dc00d33c (8 files, +409 -80); red c716301d before fixes dd827e9e, 2372edaa, dc00d33c.
- rw-1 MAJOR: held I13 measured; red on the head, so co-21 is in known-red (slice WM6, match "is missing beside"); the slice row no longer overclaims. Closed as the brief named.
- rw-2 to rw-6: selftest cells in tests/test_acts.py, red on bc54896a, green on dc00d33c. Closed.
- rw-8: ipc.rs tests use one global subscriber with a thread-local route; hang.rs:301 and log.rs:118 keep with_default (thread default wins). Closed; the full crate tests run in the train.
- rw-7 NIT open -> backlog. Note: the ring-neighbour rule is red at rest too; WM6 decides from the pose maths.
Verdict (computed): no open BLOCK or MAJOR -> MERGE-OK dc00d33c; train 59.
