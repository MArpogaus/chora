# xp-cube-faces

## r1

Branch claude/xp-cube-faces, from 1ec04694. Area: plugins/chora-xp only (wm/hosted.rs, wm/panels.rs, wm/dispatch.rs, a test accessor in desktop_icons.rs); no core, no construct change.

### rcf-8, rcf-10

- rcf-8: `refresh_icons` laid the icons on the work area of `bars.first()` (face 0), so the keys to the icons of the front face went to icons out of sight on faces 1..3.
- rcf-10: the exported `start_menu` call opened on `bars.first()` on a hosted cube.

Design (PM, POLS): one set of icons that follows the front face, no copies per face. Without a host nothing changes.

### Red first: ba5c0d22

Both tests red on 1ec04694 plus the tests:

    the_icons_follow_the_front_face_of_a_hosted_cube
    face 2 at rest: [[-262136, -262136], [-262136, -262048], [-260952, -261594]]
    the_start_menu_call_opens_on_the_front_face
    assertion `left == right` failed
      left: Some("V-1")

(the front face is V-3; the icons lay at face 0.)

### Fix: d588d573

- `Wm::front_bar()` (wm/hosted.rs): the bar of the front face, else `bars.first()`.
- `refresh_icons` and the `StartMenu` call use it.
- `host_turned` refreshes the icons at rest, in the same event that moves the keys (D116); no timer, no sleep.
- `a_turn_of_the_host_leaves_the_keys_of_another_client` asserted an empty command list; a turn now also moves the icons, so it asserts that no `FocusNode` is sent.

### Green

- `cargo test -p chora-xp`: 347 passed, 0 failed.
- tests/gate-construct-focus.sh:
  `ok: a turn takes the keys from the window of a face that does not show (core: xp-icons  xp: xp - construct: {'holder': None})`
  `ok: the keys stay with the front face after its focused window closed (core: xp-icons  xp: xp - construct: {'holder': None})`
- tests/gate-construct-i3.sh:
  `ok: one focus holder after a click in the xp face with a window in a pane (construct: -, xp: xp)`

### Gate

`tests/merge-gate.sh --jobs 2` (full), detached, on d588d573; the merge with origin/claude/autonomous-dev-session-3w-i33pg7 was a no-op (the branch had not moved). Run dir /tmp/merge-gate.j7j3WB, log /tmp/gate.log.

All cells PASS up to and including `acts`, among them `xp`, `virtual`, `xp-faces`, `xp-fields`, `cube`, `construct-hud`, `construct-extra`, `keys`. One red, not caused by this change:

    mc-scale   FAIL    19s
    ok: scene.save of 300 panels in 2709 ms
    ok: scene.show of 300 panels in 1502 ms
    FAIL: settings.set of a theme beside 300 panels takes 9316 ms, more than 5000 ms

A time budget of mc on 300 panels (run after the 1939 s `acts` cell, two jobs); xp is not involved. Not rerun. The gate stops at the first red, so the steps after it ran with `--only` (run dir /tmp/merge-gate.ZNzXAB, EXIT 0, "partial run: nothing is pushed"):

    watchdog-starved PASS, leak PASS, xp-run PASS, demo PASS,
    tiling-bar PASS, tiling-theme PASS, mc-host PASS, e2e PASS

So: rcf-8 and rcf-10 red (ba5c0d22) then green (d588d573), the construct cells green, the gate green but for the foreign red `mc-scale` (settings.set 9316 ms > 5000 ms).
