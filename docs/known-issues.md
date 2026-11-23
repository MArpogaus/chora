# Known issues

This is a quick release candidate. The faults below are known. Most of them
are in the look layers (xp, tiling, mc, construct) and the doors they share,
and a cleanup release may address them. Nothing here is a promise of a
date. Severity comes first in every section: BLOCK,
then MAJOR, then MINOR, then COSMETIC.

"likely fixed" means a later commit names the fault or a gate cell covers it,
but nobody has checked it by eye on a fresh capture. "unverified" means the same, for older entries: no one has checked it since rc 3. Larger faults are also tracked on
[GitHub](https://github.com/MArpogaus/chora/issues).

## xp

MINOR
- xp-9: Turn Off dialog: Cancel is about 56x16 (under the 24 px target), icon captions are 13 px, the desktop is not dimmed.
- xp-10: Date and Time: 12 px text, small buttons, a single click on the clock does nothing, no taskbar button.
- xp-11: Help window: Esc does not close it, empty right pane on the first topic, the key table has no scroll to more rows.
- xp-13: Recycle Bin hides under the open chat balloon; desktop labels and titles are about 12 px.
- xp-16: the paperclip helper is small and unlabelled; a tester did not find it.
- xp-19: workspace buttons 1-4 are flat blue squares that read as foreign to the taskbar.
- xp-28: chat header stays "Working, step 1 of 1" with Stop active after the answer; the editor opens over desktop icons.
- xp-29: the helper chat opens from the tray icon; a click on the paperclip itself should open it.
- xp-30: no welcome balloon at the first start; the paperclip should greet once.
- xp-5, xp-6, xp-7, xp-8, xp-12, xp-14, xp-15, xp-17, xp-18 (unverified: no check in the ledger since rc 3): Start lists LibreOffice though not installed; no sign that a program is starting; notify balloon size and place; caption button size; chat field after the no-helper balloon; question card text; Start menu label truncation; menu leftovers; Alt+Tab not seen in capture.
- Notepad needs an editor with a window; without one a click says so and no window opens. mousepad is recommended (xp-25).

COSMETIC
- xp-20: "Chat with a helper" icon is a green dot; Control Panel rows mix icon and none; two windows are titled "foot".
- xp-21: Turn Off and submenu rows are flat, pale fallback icons, Send and input differ in height.
- xp-22: Turn Off uses own power art instead of the XP flag logo (not importable from the sparse set).

## tiling

BLOCK
- ti-1: with fuzzel older than 1.11 a mouse click in the Apps menu keeps the selection on the first row and starts no program. Install fuzzel 1.11 or later (the Arch package requires it); with 1.11 a gate cell passes. Open for other packages until they pin it.

MINOR
- ti-11: after changing look.theme, running foreign programs keep the old palette (the protocol gives chora no way to tell them).
- ti-12: there is no focus-left/right key; Super+Right switches the tag and Super+l widens the master (by design, to be listed).
- ti-14: the key list shows only keyboard chords, no mouse basics.
- ti-25: stale or torn buffers after a re-tile; the launcher shows raw .desktop text; a floated window can land half off-screen.
- ti-8, ti-9, ti-10, ti-13 (unverified: no check in the ledger since rc 3): new windows do not take focus; key list is 692 px high; key list is translucent over seams; bar network shows the address.

COSMETIC
- ti-15: about 17 small pixel items (toast fringes, orb ring, launcher size, tooltip shadow, hairlines, island width).
- ti-16: question card uses one size and no mono for the path; the chat panel is empty while a question waits.
- ti-17: Ink shadows are invisible; unfocused border is lighter and uneven; the left island is fixed at 448 px.
- ti-18 (unverified: no check in the ledger since rc 3): listening chip on the window border; three font families in one frame; key list says Super+Print.

## mc

MAJOR
- mc-47: with fuzzel older than 1.11 a click on a row of the launcher in mc starts no program (keys work). Install fuzzel 1.11 or later; with 1.11.1 the gate cell mc-sofa launcher-click-row passes 5 of 5 (same cause as ti-1).

MINOR
- mc-9, mc-10, mc-11, mc-12 (unverified: no check in the ledger since rc 3): chat field shape, card text wrapping, Stop word, notification count with chat open.

COSMETIC
- mc-15, mc-16, mc-17: Yes/No hover and grid, wrapped notification, duplicate "1 AGENDA" head, baseline and margin offsets of 1 to 3 px.
- mc-18: the light terminal is a black block on cream; the question is stated three times in the column.

## construct

MINOR
- co-5: the step-back capsule is missing at 2x after step-in.
- co-6: a failed face stood back at 2x has no scene (flat background).
- co-7: the failed-face card is not fitted to its face.
- co-12: faces B, C, D show wallpaper only until each runs its own manager; the face does not say so.
- co-17: three terminals are all titled "foot" in the window list.
- co-8, co-9, co-10, co-11 (unverified: no check in the ledger since rc 3): old capture gaps, OPEN CUBE leftovers, D in walk view drops into a fullscreen terminal, closed window leaves a pill.

COSMETIC
- co-13: tray text differs between 1x and 2x; tiny terminal text in side panes.
- co-15: an empty floor gives no hint that walking works.
- co-14 (unverified: no check in the ledger since rc 3): pull frame shows a blank slab and a yellow test marker (the marker is the pointer of the test harness, fb-swarm4-c4).

## All looks and shared doors

MINOR
- sh-12: the question card shows no time left; xp and mc add no transcript line after an answer.
- sh-14: the mute state has no word in tiling and xp (a dot or red sign only).
- sh-15: Esc differs between looks; the keys sheet is blocked while the chat is open.
- sh-16: launcher wording differs ("Open:", "Apps", ">"; "Run a program" vs "Start a program"). Reported fixed earlier (review raa-3); the ledger lists it again.
- The spoken prompt of a sub-agent's tool title may still keep bidi or hidden characters (the shown text is cleaned).
- sh-28: documentation is stale on mc telling an injected click, and on the icon fallback to the temp directory.
- sh-13, sh-17 (unverified: no check in the ledger since rc 3): state word spelled three ways and helper surface counted twice; HUD colours per theme.
- mesh-churn review finding 1 (core, MINOR): two views of one scale (a layer on two outputs, or an output and its capture) keep the old layer sizes for a while after many resizes, so memory use grows during that time. Same as before the mesh-churn fix, which covers one view. Workaround: none needed for one output.

COSMETIC
- sh-18: door wording and notification shape differ per look.
- sh-21: the task cut is per character and can leave a space before the ellipsis; a long task hides the step count.

DEFERRED DEPENDENCIES
- zbus 6 (#96) and serde-saphyr 2 (#87) are major bumps, deferred past rc 7.
