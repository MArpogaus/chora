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

## Review r1

Seeded review (briefs/review-handover-ti-r1.md) of claude/handover-ti
371f0fa1 against ce2c436b; read-only on code. Step 0 green (unshallow,
`cargo check --workspace`). `cargo test -p chora` 638 ok;
`cargo test -p chora-kit --features testing --test wm_focus` 6 ok;
`cargo test -p chora-tiling -p chora-xp` 176 + 308 ok.
tests/gate-handover-shared.sh PASS at 371f0fa1; tests/gate-xp-faces.sh
PASS (incl. float-survives-restart, so the second root survives a restart).
Red first checked by building each commit: cb953cfe fails
"keys: tiling and xp each say they hold the focus"; f33684bf passes keys
and fails "give: s1 is not on the screen ... the desktop of xp
(139 193 98 255)"; 371f0fa1 passes. Commit order cb953cfe < f33684bf <
371f0fa1; the matrix row of merge-gate.sh runs the cell.

Probe (scratch copy of the cell, injected clicks accepted by both looks):
give s1 to tiling, then click the xp desktop in the gap at 5,400, then
click s1 at 640,400:

```
given             tiling {holder:tiling, s1}   xp {null}   focused [s1]
click-desktop     tiling {null}                xp {xp}     focused []
click-desktop 2   tiling {null}                xp {xp}     focused [xp-icons]
click s1          tiling {null}                xp {xp}     focused [xp-icons]
```

Checks: 1) focus.node is sent once per change (after_events compares
with told_node), to every subscriber, with only node id and owner; the
escape hatch is untouched. 2) one holder holds in the three cells, but a
click on tiling's window does not take the keys back (rw-1). 3) xp alone
and beside tiling looks the same (capture before.png: wallpaper, icons,
taskbar), icon click, desktop context menu and Escape work; Log Off fade
covers both roots. 4) red first as above, with rw-3. 5) known open: the
overlap of a tiling and an xp window stacks by root order, so a focused
tiling window can sit under an xp window; no window is lost (xp's
taskbar, minimize and the give keys reach every window), so it is MINOR
for the RC shared bar, not MAJOR.

Findings:

- rw-1, MAJOR, plugins/chora-tiling/src/main.rs:1018 (with crates/chora-kit/src/wm/focus.rs:86): after a click on the xp desktop (now visible beside tiling's windows since T7) a click on tiling's focused window does nothing: `pointer_focus` gets `focus.lost`, which neither `focus.node` (`away`) nor a core focus of `None` sets, so tiling thinks s1 still has the keys while xp-icons has them (probe above); fix: count `taken_by_other(NAME)` and a core focus other than the sent surface as lost in `pointer_focus`, and add the click-back step to gate-handover-shared.sh.
- rw-2, MINOR, plugins/chora-xp/src/wm/press.rs:227: the first click on the xp desktop while a tiling window has the focus gives the desktop icons no keyboard (`focus_away` still sees the tiling surface; focus.changed(None) comes after and `focus_moved` does not resend `desktop_keys`), yet xp's state says it holds the focus; fix: resend `desktop_keys` in `focus_moved` when the core focus leaves a foreign window.
- rw-3, NIT, tests/gate-handover-shared.sh:103: the `node` cell was never red for I3: in cb953cfe it called `start_menu` without `open`, and f33684bf fixed the call with the fix (keys failed first anyway); fix: keep test edits in the red commit, or note the cell as green-only in the report.
- rw-4, NIT, crates/chora/src/state.rs:1118: `tell_key_node` reads `scene.focused_node()`, not `key_node()`, so an exclusive layer surface that holds the keys is reported as the node holding them; fix: derive the told node from `key_node()`.
- rw-5, NIT, crates/chora-kit/src/wm/focus.rs:181: `hear` stores the own owner in `away` for an own node that `node` does not track (tiling's key list), so `holder()` answers `Foreign` while `state()` answers the look; fix: drop an `owner` equal to the look's name in `hear` (needs the name) or check it in `holder`.

Counts: BLOCK 0, MAJOR 1, MINOR 1, NIT 3. Verdict: **FIX** (rw-1).
