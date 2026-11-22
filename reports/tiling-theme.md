# tiling-theme

## r1

Lane claude/tiling-theme from 6b01c976. Fix 4e46028b (one commit, on top of
6b01c976; the merge with origin/claude/autonomous-dev-session-3w-i33pg7 was
"already up to date").

### ti-39: cause found, fixed

**Look (step 1).** Capture of the gate step with the border at x 12, y 400
(the pixels beside it, 11 and 14, are the wallpaper 0 0 0):

| run | 0.5 s | 5 s | 10 s | opacity of tw-s1 |
|---|---|---|---|---|
| a | 188 126 21 | 188 126 21 | 188 126 21 | 0.7318 |
| b | 202 133 20 | 202 133 20 | 202 133 20 | 0.7940 |
| c | 40 28 11 | 40 28 11 | 40 28 11 | 0.1412 |
| gate, 3 runs of 6b01c976 | | | | 209 138 18; 49 34 12; (findings: 210/202/192) |

It is not a fade, a glow, the inactive colour, a wallpaper blend or an alpha
of the border: the value is constant after the reload (nothing changes from
0.5 s to 10 s) and differs between runs from 14 % to 87 % of the accent.
"Darker the longer the wait" of the finding was the run to run spread.
`tg-s1` (the focus border) holds the accent 245 158 11 with alpha 1.0 in
scene.query; `tf-s1` (the glow) is hidden with shadows off. The node that
draws over it is its parent: the group `tw-s1` of the window has opacity
0.7318 (run a), and the whole window is that transparent too, not only the
border.

**Cause (step 2), with lines.** Trace of the commands to tw-s1 (a temporary
log in call_apply, not committed), run c:

    TRACE ...924.005 update  tw-s1 {transform scale 0.94, opacity 0.0, visible true}
    TRACE ...924.005 animate tw-s1 {transform, opacity 1.0} duration 180 ease_out   (open)
    TRACE ...924.085 update  tw-s1 {transform, visible true}                         (no opacity)

The reload (`reconfigure`, plugins/chora-tiling/src/main.rs:673-690) does
`self.sent.clear()` and `relayout()` with `still = true`, so `animate` is
false (windows.rs:160) and `place()` (dress.rs, the branch
`!dress.animate || ...`) answered `shown(end, None)`: an update without
opacity. An update on a node ends its running animation where it stands
(crates/chora-scene/src/scene.rs:752 `self.animations.remove(id)`, the test
`an_update_or_a_removal_ends_an_animation`), and the node keeps the
interpolated opacity. A window opened less than 180 ms before the reload,
which is what the gate does (`chora-probe window` then `settings.reload` at
once), stays at the opacity of that moment: 18 ms after the open gave 0.14,
80 ms gave 0.79. The comment of `place()` already said "a move carries the
opacity 1 too: it ends an open that still runs"; two branches missed it.

**Fix (step 3).** Both `shown(end, None)` branches of `place()` now carry
`Some(1.0)`. A unit test (`a_jump_ends_an_open_that_still_runs_with_the_opacity_1`)
holds it; `a_requested_look_stays_across_a_relayout` looked at every update
with an opacity and now looks at the window `s1` only (the group is another
node). The gate was right (the design wants a full accent border), no
tolerance raised. The step `known-red-tiling-theme` is now `tiling-theme`
(tests/merge-gate.sh), so a full run includes it.

- Red (before the fix, 6b01c976 + the row): `FAIL: the focus border is not the new accent: 209 138 18 255` and `... 49 34 12 255` (2 of 2).
- Green (after): tests/gate-tiling-theme.sh 3 of 3 green; `tests/merge-gate.sh --only ci-check,tiling-theme --jobs 2` 5 of 5 exit 0, 0 red (tiling-theme PASS 18 s, ci-check PASS 19 s).
- cargo test -p chora-tiling: 182 passed.

### ti-11: already fixed for assistant surfaces; foreign clients: report only

On the head, with the chat panel open (chora-assistant, chat.show) and
`ctl call tiling settings.set '{"look":{"theme":"paper"}}'`, the capture of
the node hud-chat reads 26 29 39 (Ink surface) before and 255 255 255
(Paper surface) 2 s after, at all three sample points; the panel also grew to
its Paper layout (rect y 52 h 656 to y 12 h 696). `apply_theme` calls
`name_theme` (overlay.rs:29, 112-118), which emits `<client>.theme` and
`look.theme` to the HUD host: 6092aee3 (2026-11-12, helper surfaces wear Ink
and Paper) and 773031b4 (2026-11-19, look.theme). The finding came from the
ux-deep-tiling run of 2026-11-12 and was filed as open since. No code
change. There is no gate cell for the chat; the pixels above are the
evidence (a cell would need the assistant in tests/gate-tiling-theme.sh).

Foreign clients (running programs such as foot or a GTK window) keep the
palette they started with: Wayland has no theme message; they read
`org.freedesktop.appearance color-scheme` of the settings portal or
gsettings when they start (or watch the portal themselves). chora only
reads that value (theme.rs, `scheme_of_portal`) and writes no portal.
What the protocol allows chora: nothing. Not changed.

### Gate

Full `tests/merge-gate.sh --jobs 2` (no --only) on claude/tiling-theme at
4e46028b (merged with a fresh origin/claude/autonomous-dev-session-3w-i33pg7,
already up to date at 6b01c976), detached: 58 steps PASS, 0 red, no foreign
red, `all steps passed; no --push, nothing is pushed`, `EXIT 0`, including
tiling-theme (PASS 18 s) and e2e (PASS 256 s).
