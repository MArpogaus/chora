# Persona round 3 (2026-11-11)

Session branch f6c09e5, debug build. Captures and logs: `media/2026-11-11/persona/`. "P" with a
number is a principle of `docs/design-principles.md`.

## Setup and caveats

- No `session.sh --voice` (H12) on the branch: a scratch harness added chora-assistant, the real
  chora-voice with `w16-fakes.py`, and `fake-acp.sh` as project `demo`; Lena spoke through the fake
  ASR, and a keyword stand-in played the LLM.
- Harness effects, not product: the stand-in said "Erledigt." after every tool call, even a failed
  one; it never started a helper (Lena's goals 2 to 4 were rechecked by a scripted voice run);
  Kemal's run 1 is void (second xp without `CHORA_TEST_ACCEPT_INJECTED_KEYS`); Max's HUD folder
  missed the session's `XDG_DATA_HOME` (only M8 counts).

## Results

| Persona | Task | Reached | Steps (short) |
|---|---|---|---|
| Erika, mouse, xp | start a program | yes | start, Terminal (foot) |
| | find the window again | yes | minimize, taskbar button |
| | open the chat, write | no | Chat ... opens list "1 demo"; typed line is refused as offline (B2) |
| | volume | no | tray, Control Panel, All Programs: nothing (B3) |
| Kemal, keys, tiling + xp | start a program | yes | Super+Return; it opens on the other screen (M4) |
| | find the window again | yes | Alt+Tab on xp |
| | open the chat, write, leave | no | Super+a opens the panel; the keys stay in foot (B4) |
| | volume | no | media keys do nothing, no indicator (B3) |
| | window xp to tiling | no | Super+Shift+arrows, comma, period type text into foot (M5) |
| Lena, voice, de, xp | open the chat | no | 3 tries, each `chat.show` refused (B1) |
| | ask a helper (scripted) | yes | "Atlas, schreib ..." reaches Atlas; card shows |
| | answer the question | no | a spoken "Ja" counts as nothing and gets no reply (M1) |
| | stop a helper (scripted) | yes | "Atlas, stopp" cancels; `agent_stop` stops |
| Max, ricer, construct | construct, panes | yes | t, walk, Return, Alt+Esc, `pane.move` |
| | xp cube | yes | `cube.open {}`, Ctrl+Alt+Right, STEP IN, Alt+Esc, Return |
| | pull a window out | no | the window leaves xp but its pane is invisible (B5) |
| | own HUD folder | no | `--check-hud` OK, then `--use-hud` "no HUD folder" (M8) |
| | break things | partly | bad size and symlink caught; findings below |

## Block

- **B1 Voice cannot open the chat (P2, P3).** `assistant chat.show` refuses the
  voice tool process: "chat.show: not for a confined connection". The client
  `voice-mcp` is a fixed-name socket, so `ipc.rs:485` counts it as confined, and
  `chat/host.rs:83` guards against it. Flow F1-V is a dead end. Repro: voice
  with `allow_methods: [assistant.chat.show]`; the LLM calls `call_method`
  assistant chat.show {name: demo}. See `lena-voice-turns.log` and
  `lena-04-chat-never-opens.png`.
- **B2 Typed chat line answered with a false "offline" (P4, P5).** In the helper
  list, any text that is not a number or a name goes to `say_failed`
  (`chat.rs:521-535`). The field shows "Cannot reach voice control. Trying again
  every 5 s." while voice is up. The list "1 demo" gives no hint to pick first,
  and "demo" is a project name, not a helper. Repro: click Chat ..., type
  "Hallo", Enter. See `erika-13-chat-false-offline.png`.
- **B3 No volume control (P3, P2).** xp has no tray speaker and no Sound entry
  in the Control Panel or All Programs. tiling's waybar `pulseaudio` module is
  an unlabelled yellow block. XF86Audio keys show nothing. See
  `erika-01-*`, `erika-21-*`, `kemal1-12-*`.
- **B4 The chat field never gets the keyboard (P2, P11 inverted).** tiling
  Super+a shows the panel, but typed text goes to the focused foot. It also
  failed with `--test-injected-press-is-real` and `MOD+a: {run: chora-chat}`;
  the root cause is not traced. Kemal found no keyboard way to close the panel,
  and nothing visible closes it in xp either (Erika). See
  `kemal-17-chat-text-goes-to-terminal.png`.
- **B5 A window pulled out of the cube is lost to view (P5).** Steps: open the
  cube, Start, Terminal on the face, drag the title bar to the screen edge,
  Alt+Esc twice. The xp taskbar button goes away. Only a `FOOT` label draws
  through the face, no pane shows, and the Tab window list has no `foot`. See
  `max-40-*`, `max-43-*`.

## Major

- **M1 A voice-only user cannot allow a write, and is not told (P2, P4).** A spoken "Ja" counts as
  nothing (by design, P5), but nothing is said; F3 asks for "For this I need a key press or a click."
- **M2 German voice words (P4, P7).** "Soll ich einen Agenten für demo
  starten?" uses "Agent". "Drücke Alt+y" uses du, not Sie. The card prompt
  repeats the key sentence with other spellings ("Alt+y" / "Alt+Y"). The title
  is "Frage an Sie", not "Frage von Atlas". There is no time left. See
  `lena-s2-*`.
- **M3 The chat is not themed per look (P8, P6).** xp and construct show the
  plain "Chat ..." pill and the 400 px side panel, not the balloon (F7) or the
  glass (F9). The panel covers the paperclip, the clock and the Recycle Bin, and
  has no Stop or Close button (F2).
- **M4 Two outputs: tiling and xp (P8, P3, P4).** waybar draws on the xp output
  over xp, and tiling's tag "2[tile]" shows there. A new window opens on the
  screen that was clicked last, not the one being looked at, and nothing shows
  where. The HUD jumps between screens. See `kemal1-12-*`, `kemal1-05-*`.
- **M5 The give keys are not shown, and unbound Super keys leak (P3, P1).**
  `Super+Alt+1..4` (give) and `Super+a` (chat) appear nowhere on screen.
  Super+Shift+Left types "0D", Super+Shift+comma "<", and Super+c "c" into the
  terminal. See `kemal-24-*`.
- **M6 Only xp starts the chat host (P3).** tiling and construct have no chora-assistant unit, so
  an install has no "Chat ..." there; `session.sh` starts it in no flavor.
- **M7 The cube has no visible way in; `cube.open` without params fails (P3, P9).** Only the
  privileged method opens it; `choractl call construct cube.open` gives `invalid type: null,
  expected struct OpenCube`, though the docs default `faces` to 4. See `max-18-*`.
- **M8 `--check-hud` hint that does not work (P4).** For a folder outside `$XDG_DATA_HOME/chora/huds`
  it says "OK. Turn it on: chora-assistant --use-hud max"; `--use-hud` then says "no HUD folder
  named max". It should name the search path. See `max-45-*`.
- **M9 Walk keys while typing spawn terminals in silence (P4, P3).** Text typed outside a pane ran
  `t` and `space`: 10 foots, 8 "no pane", no "No room in view" notice. See `max-43-*`.

## Minor

- Updates show one capture late after a click or key (Erika, Kemal); a beginner clicks twice (P4).
- "Details auf dem Bildschirm." while the balloon shows one sentence (P5); "Stopp" with nothing
  running gets no reply (P4).
- xp menu words "Foot Client", "Foot Server", "UXTerm", "Vim"; no simple note program (P4, P9);
  foot opens with a yellow "C locale" warning.
- Construct: a label draws over a nearer pane; the centre label hides behind the LISTENING card;
  far panes are grey slabs; closed panes' labels linger; the view drifts (`max-14-*`).
- Construct cube: "STEP BACK" chip without its key after `cube.open`; the chip covers the Recycle
  Bin label; navigate chips stay after a fly into a pane; a 1 s fly-in drops typed keys
  (`max-21-*`, `max-46-*`). Two voice indicators (LISTENING card and green dot, P6); a yellow
  square shows (likely the injected-input mark).

## What held up

`--check-hud` caught `chat.size: 2` and a symlink to `/etc/passwd`; the physical card showed and a
spoken yes did not answer it (P5); "Atlas, ..." and `agent_stop` work; cube turns, step back and
step in by key and chip work; ten terminals and a queued turn crashed nothing.
