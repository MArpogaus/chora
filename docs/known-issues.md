# Known issues

This is a quick release candidate. The faults below are known. Most of them
are in the look layers (xp, tiling, mc, construct) and the doors they share,
and a cleanup release may address them. Nothing here is a promise of a
date. Severity comes first in every section: BLOCK,
then MAJOR, then MINOR, then COSMETIC.

"likely fixed" means a later commit names the fault or a gate cell covers it,
but nobody has checked it by eye on a fresh capture. "verify" means the same,
for older entries. Larger faults are also tracked on
[GitHub](https://github.com/MArpogaus/chora/issues).

## xp

MAJOR
- xp-23: the paperclip helper was missing from the desktop. Likely fixed (commit 5d801b2f, gate cell 65f4510b); check on the next capture.
- xp-24: text fields (Run, search, dialogs) are not aligned to the XP grid. Likely fixed (5d801b2f "fields on grid"); no pixel assert yet.
- xp-25: Notepad opened nano in a dark terminal. Likely fixed (21799660, eb970cfd: only a GUI editor opens, otherwise a notice says no editor is installed). Install mousepad.
- xp-26: after Sound is set to 75% in Control Panel the tray shows no level and the menu has no tick. Open.
- xp-27: the question card uses another font and a white box, covers the transcript, and names keyboard keys (Alt+Y/N) to a mouse user. Open.

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
- xp-5, xp-6, xp-7, xp-8, xp-12, xp-14, xp-15, xp-17, xp-18 (verify): Start lists LibreOffice though not installed; no sign that a program is starting; notify balloon size and place; caption button size; chat field after the no-helper balloon; question card text; Start menu label truncation; menu leftovers; Alt+Tab not seen in capture.
- Notepad needs an editor with a window; without one a click says so and no window opens. mousepad is recommended (xp-25).

COSMETIC
- xp-20: "Chat with a helper" icon is a green dot; Control Panel rows mix icon and none; two windows are titled "foot".
- xp-21: Turn Off and submenu rows are flat, pale fallback icons, Send and input differ in height.
- xp-22: Turn Off uses own power art instead of the XP flag logo (not importable from the sparse set).

## tiling

BLOCK
- ti-1: with fuzzel older than 1.11 a mouse click in the Apps menu keeps the selection on the first row and starts no program. Install fuzzel 1.11 or later (the Arch package requires it); with 1.11 a gate cell passes. Open for other packages until they pin it.

MAJOR
- ti-21: the bar does not match the cyberpunk style of the rest of the look (borders, glow, chat, launcher). Open.
- ti-22: Super+Print gave no notice. Likely fixed (29ee9ebc, ebf2bad1: notice shown with no bus, Screenshot in the power menu).
- ti-23: a stale "Run a program" tooltip stayed about 14 s over the master title. Likely fixed (c1adc900, e1dda18f: no tooltip on the bar doors).
- ti-24: no power or lock icon in the bar though the key list names it; a "Chat ..." pill floats over client content while the bar says "No helper". Open (the Screenshot entry in the power menu was added, 29ee9ebc).

VERIFY (severity not set)
- ti-20: a click on Apps showed no launcher within 1.5 s in one headless run. MAJOR if real; a CI cell covers the click.

MINOR
- ti-5: a toast that starts while the chat is open is hidden under the chat panel.
- ti-6, ti-19: tooltips "Run a program" and "Every key" stay after a click or Esc.
- ti-7: in fullscreen the bar floats over the window and covers the first terminal line.
- ti-11: after changing look.theme, running programs and assistant surfaces keep the old palette.
- ti-12: there is no focus-left/right key; Super+Right switches the tag and Super+l widens the master (by design, to be listed).
- ti-14: the key list shows only keyboard chords, no mouse basics.
- ti-25: stale or torn buffers after a re-tile; the launcher shows raw .desktop text; a floated window can land half off-screen.
- ti-26: an empty LC_ALL="" masks LANG=de.
- ti-8, ti-9, ti-10, ti-13 (verify): new windows do not take focus; key list is 692 px high; key list is translucent over seams; bar network shows the address.

COSMETIC
- ti-15: about 17 small pixel items (toast fringes, orb ring, launcher size, tooltip shadow, hairlines, island width).
- ti-16: question card uses one size and no mono for the path; the chat panel is empty while a question waits.
- ti-17: Ink shadows are invisible; unfocused border is lighter and uneven; the left island is fixed at 448 px.
- ti-18 (verify): listening chip on the window border; three font families in one frame; key list says Super+Print.

## mc

BLOCK
- mc-32: in dark mc, remote presses (Right, Home) did not turn the page. Open; first check whether it is the product or the test injection.

MAJOR
- mc-33: the Programs/chat door by mouse opened no chat. Likely fixed (dda31e6a: a press on the chat door in the column opens the chat).
- mc-34: the voice HUD text in the footer is about 11 px grey, hard to read from a distance. Open.
- mc-35: a critical alarm is a truncated footer line, not a card; the column breaks. Open.
- mc-36: a "No helper" pill shows while the helper answers and overlaps the status word. Open.
- mc-37: Super+n hides the column but the page does not widen. Open.

MINOR
- mc-13: scene copy: "collector" on page 2 reads as a failure, mixed fonts, divider off centre.
- mc-14: chat body 18 px instead of 19; stray space in "Super+ <- ->".
- mc-9, mc-10, mc-11, mc-12 (verify): chat field shape, card text wrapping, Stop word, notification count with chat open.

COSMETIC
- mc-15, mc-16, mc-17: Yes/No hover and grid, wrapped notification, duplicate "1 AGENDA" head, baseline and margin offsets of 1 to 3 px.
- mc-18: the light terminal is a black block on cream; the question is stated three times in the column.

## construct

BLOCK
- co-18: a drag on the stepped-back cube did not turn it and the chora window vanished from face 1. Likely fixed (merge dbdfcce4 names it; bb8df12a). The ledger still lists it open and co-21 shows the stepped-back view is still wrong, so check by eye.

MAJOR
- co-21: stepped back, the cube is not closed: the neighbour face juts toward the viewer and is larger than face 1, the faces do not share an edge, the floor reflection does not line up (owner report 11-17, tour v7 stills 04-12, 04-13). Open.
- co-16: after a drag in the stepped-back view the cube can leave the screen to the right and an empty floor is labelled "XP 2 face 2 of 4"; only step-in brings it back. Open.
- co-19: F1 in the stepped-back view showed no key overlay. Likely fixed (merge dbdfcce4 names co-19).
- co-20: with the cube open no key closes it (only cube.close over IPC). Likely fixed (merge dbdfcce4 names co-20); check that Escape or the bindings overlay offers a way out.

MINOR
- co-5: the step-back capsule is missing at 2x after step-in.
- co-6: a failed face stood back at 2x has no scene (flat background).
- co-7: the failed-face card is not fitted to its face.
- co-12: faces B, C, D show wallpaper only until each runs its own manager; the face does not say so.
- co-17: three terminals are all titled "foot" in the window list.
- co-8, co-9, co-10, co-11 (verify): old capture gaps, OPEN CUBE leftovers, D in walk view drops into a fullscreen terminal, closed window leaves a pill.

COSMETIC
- co-13: tray text differs between 1x and 2x; tiny terminal text in side panes.
- co-15: an empty floor gives no hint that walking works.
- co-14 (verify): pull frame shows a blank slab and a yellow test marker.

## All looks and shared doors

MAJOR
- sh-23 / sd-h1: xp and tiling in two panes: the tiling output stays black (no bar, no start door), and after a click into it Super+Return opens the xp Start menu in the other pane. Open.
- sd-h2: the xp Start menu stays open through a drag and key gives and covers the window. Open.
- sh-24: a grab gated by the pointer could fall through to the focused client as a raw key. Likely fixed (merge dbdfcce4 names it; 9c4f1301, 0a3224b8 carry the injected flag).

MINOR
- sh-12: the question card shows no time left; xp and mc add no transcript line after an answer.
- sh-14: the mute state has no word in tiling and xp (a dot or red sign only).
- sh-15: Esc differs between looks; the keys sheet is blocked while the chat is open.
- sh-16: launcher wording differs ("Open:", "Apps", ">"; "Run a program" vs "Start a program"). Reported fixed earlier (review raa-3); the ledger lists it again.
- The spoken prompt of a sub-agent's tool title may still keep bidi or hidden characters (the shown text is cleaned).
- sh-28: documentation is stale on mc telling an injected click, and on the icon fallback to the temp directory.
- sh-13, sh-17 (verify): state word spelled three ways and helper surface counted twice; HUD colours per theme.

COSMETIC
- sh-18: door wording and notification shape differ per look.
- sh-21: the task cut is per character and can leave a space before the ellipsis; a long task hides the step count.
