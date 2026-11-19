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
