# UX/UI loop 1 (2026-11-11)

Branch 536aaee, debug build, headless sessions from `tests/persona/session.sh` and the fake voice agent
(`tests/fake-voice.py`, helper "atlas"). Captures: `media/2026-11-11/ux/`. "P" with a number is a
principle in `docs/design-principles.md`. Themes: construct and mc have light and dark; xp and tiling have
one look each. Sizes: 1280x720 throughout; 1920x1080 for construct and for the chat in construct, mc and xp.

Harness notes (not product): the persona session runs the core without `--test-injected-press-is-real`.
Without it, a click on "Chat ..." or `Super+a` opens the chat, but typed keys stay in the window
(`chat-*-light-720-d/e`). With the flag, keys work (`chat-tiling-real-720-a`). `session.sh` should take
`CHORA_TEST_FLAGS`, or persona rounds will report B4 again. The yellow square at the top left is the mark for
injected input.

## Block

1. **Construct notices draw nowhere under the default `voice_hud: host` (P4, P5, decision).** Text typed
   into the space gets no notice (`construct-light-720-14-typed-notice-host`; `construct-hud` opacity 0).
   `voice.rs` `shows_hud()` needs `draws_voice()` (= `own`). With `own`, the notice shows
   (`construct-light-720-own-b-typed-notice`). "No room in view" and give refusals are silenced in the same way.
   *Expect:* one calm line whatever `voice_hud` is. *Change:* gate only the voice parts on `draws_voice()`, and
   let `notice()` draw always. Drop the `NOTICE` label so the notice is one line (see Remove).
2. **The give keys refuse in silence in the construct (P4, P2).** `Super+t`, then `Super+Alt+2`: nothing
   happens on screen. The log says "give to xp: no window has the focus" (`give-720-a/b`). A new pane does
   not get the keyboard, and the walk has no focused window. *Expect:* the pane ahead goes, as `Super+x`
   closes the pane ahead. *Change:* give acts on the pane ahead when no window has focus; a refusal is a
   notice (needs 1).
3. **A drafted chat line blocks the helper pick (P3, P1).** Close a chat with a draft ("hello"), then open
   it again. "1", Enter makes the draft "hello1", and the chat says "Pick a helper first..."
   (`chat-tiling-real-720-c`). *Expect:* the number picks the helper. *Change:* keep the draft only inside a
   chat with a helper; the list starts empty.

## Major

4. **The chat looks the same in every look (P8, P6; M3 still open).** The plain panel shows in xp (cream,
   floating at mid-screen, its field over the taskbar), in tiling, and as a light slab in the dark construct
   (`chat-construct-dark-1080-c`). mc shows a black box on the light page (`chat-mc-light-720-c`) and a white
   card on the dark drawer (`chat-mc-dark-1080-f`). The card title is still "Question for you", not "Question
   from Atlas". *Change:* honour the folder `chat:` block, then the theme; until then, ship one neutral style
   that follows the theme.
5. **"Chat ..." and OPEN CUBE overlap at the bottom left; OPEN CUBE is clipped at x<0 (P6, P7).** Seen at both
   sizes and themes (`construct-light-720-01-start`, `construct-light-1080-d`). The host pill is light in
   the dark construct. *Change:* give the host pill one corner per look; move OPEN CUBE to 32 px from the
   edges.
6. **Chips stay over a full-screen pane (P6).** After a fly-in, "Chat ..." and OPEN CUBE cover the terminal
   text (`construct-light-720-g-flown-in`). The cube chips already hide in a pane. *Change:* hide OPEN CUBE
   in a pane, as the cube chips do.
7. **The host pill covers part of the xp Start menu (P5, P6).** "Chat ..." draws over the Log Off row
   (`construct-light-720-07-start-menu`). *Change:* in xp, the paperclip is the way in, so do not draw the
   pill there (see Remove).
8. **Pulling a window out of the cube and putting it back take different hands (P1).** A plain drag pulls a
   window out of the stood-back cube (`construct-light-720-10`). A plain drag on that pane does nothing
   (`-11`); only Super+drag puts it back (`-12`). *Expect:* the same drag back. *Change:* in navigate mode a
   plain drag on a pane also moves it, and a drop on a face gives it back.
9. **A second window can have no pane, in silence, and the list uses internal words (P4).** At 1080 a second
   `Super+t` gives "foot-2  construct · no pane" in the window list and nothing in view
   (`construct-dark-1080-f-window-list`). At 720 the list says "every window shows" and lists nothing
   (`construct-light-720-f`). *Expect:* Tab lists every window. *Change:* list all windows, with the
   hidden ones marked "not in view". Drop the owner and the words "no pane".
10. **A new pane opens too far away to read (P7).** The text of a new terminal is about 5 px tall
    (`construct-light-720-c-terminal-pane`). *Change:* open at the 1:1 distance (`pane_distance` ≈ 0.65 × width).
11. **The voice state has no word in construct, tiling and mc (P4, style rule 14).** Listening shows only a
    ring (construct), a dot (tiling) or a red dot (mc) (`chat-*-a-listening`). *Change:* the state word next to
    the ring or dot, as the style guide asks.
12. **The status bars show internal text (P4).** xp shows "wpctl not found" in the tray and in the
    volume balloon (`xp-720-c-volume-key`). Tiling's waybar shows "vol: wpctl not found", "192.0.2.2",
    "cpu 2%", "mem 5%" and "1[tile]" (`chat-tiling-light-720-a`). P4 asks for "Connected", not an address.
    *Change:* "No sound device" with the speaker crossed out; remove the address, cpu and mem; show the tag
    as "1".
13. **Chat lines overlap after a reopen (P7).** The list row "1 atlas" draws over the "You" line ("Yoatlas"),
    and a bare "?" line marks the question entry (`chat-tiling-real-720-c`). *Change:* clear the transcript
    while the list shows. Show the question entry as its prompt, under the helper's name.

## Minor

14. Header "atlas ·" with an empty state, and "atlas" in lowercase in the list and transcript; "Atlas" in
    rows (P4). Use the display name, and drop the dot when there is no state.
15. STEP BACK moves: first over the Recycle Bin label, later left of it (`construct-light-720-02` vs `-07`) (P6).
16. Stood back, the cube reads as a flat, smaller screen with no edges or depth. A turn at 0.4 s still looks
    flat (`-04`, `-05`). Nothing names the face (P4).
17. A dropped window lands with its corner at the pointer, not at the point that was grabbed
    (`construct-light-720-12`) (P3).
18. The pane label "FOOT" sits on the xp taskbar of the face (style rule 11), and two terminals both read
    "FOOT" (`-10`, `-d-walked`). Use the window title.
19. The help lists 21 rows. The cube rows show without a cube and are cut ("step back from the…",
    "Ctrl+Alt+Left / Ctrl+Al…"), and Alt+Escape appears twice (`construct-light-720-e-help`) (P6).
20. A Super+drag on empty space turns the view to an empty sky, with no hint of `h`
    (`give-720-e`).
21. After `cube.close`, a face window takes about 3 s to show in the construct (`-14` vs `-15`).
22. xp: Start says "Terminal" and the window says "foot" (`-07`, `-08`). A click on the tray speaker shows
    nothing (`xp-720-a`). F1 has no help; the only list of keys is Control Panel → Keyboard Shortcuts
    (`xp-720-f`). The taskbar shows "1 2 3 4" with no label (`xp-720-a`) (P3, P4).
23. The construct sentence box is a plain box, not glass, at about 11 px (`say-construct-light-720`) (P7).

## Remove outright

- **The "NOTICE" label** on construct notices: a notice is one line of text.
- **The "Chat ..." pill in xp**: the paperclip and its balloon already open the chat (F7). Two ways in at the
  same corner compete, and the pill covers the Start menu.
- **cpu, mem and the IP address in tiling's waybar**: they are not tasks of a person, and they are noise (P2 Calm).
- **The cube rows of the help panel while no cube is open**, and the second Alt+Escape row.
- **"construct · no pane"** and other owner names in the window list.
- **The "Atlas · question for you · Write file" row** while the chat shows the same question as a card. The
  card is the one place for the question (P5).
- **The workspace numbers in the xp taskbar** unless a person can learn what they do.

## The decision on construct notices

Today it does not hold (finding 1): with `voice_hud: host`, the default, the construct draws no notice.
With `own` it draws a two-line panel (label plus sentence) at the bottom centre. The fix is finding 1 plus
dropping the label.

## What held up

- The cube: STEP BACK, the turn keys and chips, STEP IN and Return all work. A pulled-out window stays
  in view (B5 fixed), and a Super-drop puts it back in xp.
- `Super+a`, Enter and Esc work in tiling with real presses.
- The question card shows the key line and "A spoken 'yes' does not count".
- The xp paperclip balloon shows the sentence.
- After `cube.close` the face window is not lost.
- xp has Sound and Keyboard Shortcuts in the Control Panel.
- The construct light and dark themes read well, and the help and the window list are legible at
  both sizes.

Not covered this loop: give and drag between two screens with tiling and xp (`gate-handover-matrix.sh`
covers them), German, and a real voice model.
