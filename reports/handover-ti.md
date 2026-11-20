# handover-ti: tiling -> xp hand-over, one focus holder

## r1

Lane: cloud, branch claude/handover-ti from ce2c436b. Step 0 green
(unshallow, `cargo check --workspace`). Brief: briefs/handover-ti-r1.md;
findings from reports/p2-r2-tiling.md "## r2" and
feedback/2026-11-20-p2r2-ricer.md.

Reproduced first in a persona session tiling+xp 1280x800 (Super+Return,
click, Super+Alt+2, Super+Alt+1): Super+Return opens foot with xp (xp
held the keyboard on its desktop); Super+Alt+1 **gives** the window to
look 1, which is tiling (`kit::give::KEYS`: 1 tiling, 2 xp, 3 mc,
4 construct). Tiling then owns, places (14,54,1252,702) and focuses it,
and the core says `visible`, but the root group of xp, with its opaque
wallpaper, lay above the root of tiling in the same order (D89): the
window was under the xp desktop. I3: a node that takes the keyboard with
`focus_node` (tiling's key list, xp's Start menu, xp's desktop icons
after a menu closes) leaves the focused surface as it is and sends no
`focus.changed` (rw-3), so the look whose window was that surface kept
saying it held the focus.

| Item | Red (test) | Green (fix) | Test | Cell | Capture |
|---|---|---|---|---|---|
| 1 I3, one holder | cb953cfe | f33684bf | `ipc::tests::a_node_that_takes_the_keyboard_is_told_with_its_owner` (chora), `wm_focus::a_node_of_another_look_takes_the_focus_from_the_window` (kit) | tests/gate-handover-shared.sh `keys` (red: "tiling and xp each say they hold the focus") and `node` (Start menu of xp open, then closed, over tiling's window) | - |
| 2 T7, given window visible | cb953cfe | 371f0fa1 | - | tests/gate-handover-shared.sh `give` (red: "s1 is not on the screen: at 640,385 the desktop of xp (139 193 98 255)"; green: 200 30 30, the foot background) | media/2026-11-20/handover-ti-r1/before.png, give.png |
| 3 I5 key list over a notice | not done | - | - | - | - |

Fix 1 (f33684bf): the core sends a new event `focus.node` (`node`,
`owner`, or both null) when the node with the keyboard changes
(docs/api.md events table). `kit::wm::focus::Focus` hears it: a node of
another client makes the look `Foreign` (`taken_by_other`); tiling's key
list claims the focus only while no other client's node took the keys;
xp holds no focus while another client's node has the keys, holds it
while its own node (Start menu) has them, and gives its desktop icons the
keyboard only while no window of another look has the focus.

Fix 2 (371f0fa1): xp's wallpaper and desktop icons sit in their own root
`xp-desktop` in the `background` order (D89), as mc's page does; the
windows, taskbar, dialogs and menus stay in the root `xp` with the claim.
A window that tiling gets on the shared output shows above the xp
desktop, below the xp taskbar band; the Log Off fade greys both roots.

Gate cell: tests/gate-handover-shared.sh (new, in the `matrix` row of
tests/merge-gate.sh after gate-give-holders.sh), tiling first (holds the
give keys), xp joins on the same output, a foot with a red background;
the holder check calls `invariants.i3` of tests/acts (no copy).

Gate (`--jobs 1 --only pre-commit,ci-check,build,test,clippy,gate-lib,matrix,virtual,wm-pane,mc-handover,layout`, tests/persona unchanged, so no persona-selftest):

```
pre-commit PASS     3s /tmp/merge-gate.OJUpal/pre-commit.log
gate-lib   PASS    74s /tmp/merge-gate.OJUpal/gate-lib.log
ci-check   PASS    39s /tmp/merge-gate.OJUpal/ci-check.log
build      PASS    22s /tmp/merge-gate.OJUpal/build.log
test       PASS   191s /tmp/merge-gate.OJUpal/test.log
clippy     PASS    15s /tmp/merge-gate.OJUpal/clippy.log
matrix     PASS   287s /tmp/merge-gate.OJUpal/matrix.log
virtual    PASS    43s /tmp/merge-gate.OJUpal/virtual.log
layout     PASS   180s /tmp/merge-gate.OJUpal/layout.log
wm-pane    PASS    21s /tmp/merge-gate.OJUpal/wm-pane.log
mc-handover PASS    11s /tmp/merge-gate.OJUpal/mc-handover.log
(sha 371f0fa1; partial run, nothing pushed by the gate)
```

Not done: I5 (medium, item 3; items 1-2 ran past 150 k). Left as seen:
with xp's own node (Start menu) holding the keys, the xp menu is drawn in
the root of xp, which lies above tiling's windows (no regression); a
window of tiling and one of xp that overlap stack by root order (xp's
above), not by focus.
