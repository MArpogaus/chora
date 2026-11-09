# UX pass: feature sprint 1 (ACP chat and HUDs)

UX/UI review of `design/hud-sprint.md`, made before programming. Read on 2026-11-09 with the
code at `/home/user/chora` (`chora-assistant`, `chora-mc/src/agent.rs`, `chora-construct/src/hud.rs`),
the style guides, `docs/voice.md` sections 5 and 10, the persona runs r1 and r2 and the
captures in `media/2026-11-09/`. Section 6 lists what must change before programming (marked
**BEFORE**). Values: POLS, KISS.

## 1. Users and goals

| User | Input | Goal in this sprint | What breaks them |
|---|---|---|---|
| Erika, 74, tech 0 | mouse only | see that a helper wants something, allow or refuse it, write one line to a helper | no visible entry (r1 tiling: "no menu"), no feedback after a click (r1 xp), words like "agent", "ACP", `/start` |
| Nurse, hands busy | voice only | hear what the helpers do, mute the mic, know when a hand is needed | a question that needs a key and does not say so in speech; a muted mic she cannot unmute by voice |
| Child, 9 | mouse, some typing | talk to the figure, read what it says | small text, a Yes button that allows things a parent must allow (later sprint, see 6.12) |
| Jonas, 31, tech 3 | keyboard first | open a chat, follow 3 helpers, never leave the keyboard | a feature reachable only by a click on a HUD row |
| Ricer (r/unixporn) | keyboard, editor | ship a HUD folder, see errors at once, iterate fast, post a screenshot | silent failure (error only in the journal), one HUD per context means 4 lines to edit |

Words on screen: the user sees **helper** / **Helfer**, never "agent", "ACP", "sub-agent" or
"HUD". German uses "Sie" (Erika). Ricers read "HUD" only in docs and in `--check-hud`.

## 2. Flows

Each flow lists mouse (M), keyboard (K) and voice (V). "Host" is `chora-assistant`.

**F1 Open a chat with a helper.**
- M: click the row "Chat" (no helper running) or the helper's row in the HUD -> window
  "Chat with Atlas" opens, focused, in the window manager of the context. xp also: Start ->
  All Programs -> Accessories -> "Chat with a helper". mc: the drawer.
- K: launcher (`Super+r` in tiling, Start/`Super` in xp, the app list of mc) -> type "chat" ->
  Enter. Without a name the window shows a numbered list; type `1`, Enter.
- V: "Hey Chora, what is Atlas doing?" answers by speech (no chat). Opening a window by voice:
  not this sprint (6.11).

**F2 Write to a helper and read the answer.**
- M+K: click into the window (already focused), type, Enter. The line shows at once under
  "You", then "Atlas is working ..." with the step, then the answer under "Atlas".
- Cancel: Esc or Ctrl+C; the line "Stopped." shows. Closing the window does not stop the
  helper; the last line before closing says so.

**F3 Answer a question that needs a key (physical).**
- The question card appears next to the figure (all window managers), the chat shows the
  same question with the key line; speech says "Atlas wants to write notes.txt. Please press
  Alt+Y to allow or Alt+N to refuse, or click Yes on the screen."
- M: click Yes or No on the card. K: Alt+Y or Alt+N, works in any window, also in the chat.
- V: a spoken "yes" does not count; speech answers "For this I need a key press or a click."
  A spoken "no" refuses (as today).
- Typed "y" or "yes" in the chat: not sent; hint line (wording in 4.4).

**F4 Answer a normal question (read, search, think).** Same card, plus V: "yes"/"no". Chat:
same rule as F3 (open question 4).

**F5 Mute and unmute.** M: click the figure -> muted badge, line "Microphone off. Click to turn
it on." Click the badge -> on. K: none new (voice `ptt_key` stays). V: "mute" mutes; unmute by
voice is impossible by nature, so the speech before muting says "Click me to turn it on."

**F6 Context switch.** Nobody acts: focus moves to an xp window -> within 0.5 s the xp look
shows on that output. Rules: no focus change, no sound, no move while a question is open
(6.3), the mic state is visible in every look.

**F7 xp draft.** Paperclip bottom right above the taskbar; balloon with the sentence; a tray
balloon "2 helpers working" with one row per helper; click a row = F1.
**F8 mc draft.** Top right of the work area, text only: state word, sentence, one row per
helper; click a row = F1, chat docks in the drawer.
**F9 construct draft.** One glass card top centre: ring, state label, sentence, chips; click a
chip = F1, chat opens in a pane in front of the camera.

**F10 Install a shipped HUD (ricer, K).**
1. `cp -r ~/Downloads/fancy-hud ~/.local/share/chora/huds/fancy`
2. `chora-assistant --check-hud ~/.local/share/chora/huds/fancy` -> "OK" or errors, and on OK
   prints the exact line to turn it on.
3. `chora-assistant --use-hud fancy` (new, 6.6): checks again and runs `settings.set`
   `{hud: fancy}`; the look changes on all outputs.
4. Edit `hud.yaml`, save -> the host reloads a user folder on change; an error keeps the old
   look and shows the error line on screen for 8 s.
5. Undo: `chora-assistant --use-hud default`.
- M (Erika): not a goal. Low-tech users get a choice only through flavor settings later.

## 3. States of each element

| Element | empty | busy | success | error | refused | offline | question waits |
|---|---|---|---|---|---|---|---|
| Figure / mic | voice connected, idle: figure in `idle` pose | `thinking`, `working` pose + label | `speaking` + sentence | "I could not switch the microphone." | injected press: nothing changes, line "Please use your own mouse or keyboard." | voice not running: grey figure, label "Voice off" | `asking` pose, pulse, label "Question" |
| Muted badge | - | - | "Microphone off" badge, fixed, host-drawn | - | - | - | badge stays visible over the card |
| Sentence line | hidden | typewriter of the sentence | stays 2 s after the end, fades | TTS failed: text still shows | - | hidden | hidden; the card shows the question |
| Helper rows | one row "Chat" | "Atlas · working · Write file (2/2)" | "Atlas · done" for 10 s, then "idle" | "Atlas · stopped: <reason>" in red role | project not allowed: "Atlas · not allowed" | voice off: rows hidden | "Atlas · has a question" in amber role |
| Question card | - | - | after answer: "Allowed." / "Refused." 2 s | timeout: "No answer in time. Refused." | injected press: line as above, card stays | voice drops: card closes, line "Question cancelled." | prompt, detail (scrolls), key line, Yes, No, seconds left |
| Chat header | "Chat with a helper" + numbered list; no projects: "No helper is set up." | "Atlas is working ... (2/2)" | "Atlas is ready." | "Atlas stopped: <reason>" | `not_for_confined`: "This chat may not run here." | "Cannot reach chora-voice. Trying again every 5 s." | "Atlas asks you something. See below." |
| Chat input | prompt line "Type a message, then Enter." | typing allowed; Enter queues: "(waiting: 1)" | - | send failed: line kept, "Not sent: <reason>" | - | input kept, not sent | input held, hint line (4.4) |
| HUD folder | - | - | "OK. Turn it on: chora-assistant --use-hud fancy" | one line per error: `hud.yaml:12 panels[2].anchor: unknown value "middle"` | "refused: picture.svg links outside the folder" | - | - |

## 4. Layout, contrast, text, wording

**4.1 Common rules (all looks, enforced by the host, not by the folder).**
- Minimum text: 14 px logical for anything a user must read (sentence, rows, question, key
  line); 12 px for labels of a state that also has a shape. Today the bubble text is about
  11 px (`LINE = 13`, `assistant.rs:39`) and the construct key line is an 11 px muted label:
  both too small.
- Contrast: text at least 4.5:1 on its real background (WCAG AA); the key line and Yes/No at
  least 7:1. `--check-hud` computes it for `bubble` fills and refuses below 4.5. mc light
  `muted` [0.50, 0.48, 0.45] on the page is 3.9:1 (fails); use [0.43, 0.41, 0.38] (5.0:1)
  for HUD text or set HUD text in `text`.
- Targets: Yes, No, rows and the figure at least 32 x 24 px; Yes left, No right, 8 px apart;
  a button counts a click only 400 ms after it appeared or moved.
- The question card: width min(560, output - 64) px, padding 16, under or beside the figure,
  never off the work area, on top of all panels.
- The state is never colour alone: always a word next to the figure (style rule 14 of the
  construct, applied everywhere).

**4.2 Sketches (1280 x 720 logical, work area without bars).**

```
neutral (tiling)                                       xp
+------------------------------------------------+    +------------------------------------------+
| bar 24                                         |    |                                          |
|                         +--------------------+ |    |                          +-------------+ |
|                         | Atlas · working    | |    |      +-----------------+ | 2 helpers   | |
|                         | Write file (2/2)   | |    |      | I am writing    | | working     | |
|                         | Chat ...           | |    |      | the notes now.  | | Atlas  ...  | |
|                         +--------------------+ |    |      +--------\\-------+ +------\\-----+ |
|              +--------------------------+     |    |                (paperclip 64x96)  [tray] |
|              | I am writing the notes.  |  o  |    | [start] [foot]                 [▲] 6:51 |
|              +--------------------------+ 24px|    +------------------------------------------+
+------------------------------------------------+     balloon max 300 wide, Tahoma 14 px,
 rows top right, 320 wide, 14 px; dot 24 px +           margin [96, 32] above the taskbar
 word "Listening" 14 px left of it, bottom right

mc (dark page, no boxes)                               construct (glass card top centre, 480)
+------------------------------------------------+    +------------------------------------------+
| 3 | AGENDA                  LISTENING          |    |        +--------------------------+      |
|                             I am writing the   |    |        | (ring 40) LISTENING      |      |
|                             notes now.         |    |        | I am writing the notes.  |      |
|                             ─ Atlas · working  |    |        | [Atlas ●] [Chat]         |      |
|                               Write file (2/2) |    |        +--------------------------+      |
|                             ─ Chat ...         |    |         panes ...                        |
+------------------------------------------------+    +------------------------------------------+
 Inter 14 (step -1 rounded), accent on the state         Inter 16 text, 11 label, padding 16,
 word only, hairline rows, 13 em column                  32 from the top edge

question card (same content everywhere; look per folder)
+------------------------------------------------------+
| Question from Atlas                          0:42    |   title 14, timer tabular
| Atlas wants to write the file notes.txt.             |   prompt 16
| ~/projects/notes/notes.txt (412 bytes)        ▲      |   detail mono 14, scrolls, max 8 lines
| ...                                           ▼      |
| Press Alt+Y to allow or Alt+N to refuse.             |   key line 14, text colour, 7:1
| A spoken "yes" does not count.                       |
|                               [  Yes  ]  [  No  ]    |   buttons 80 x 32
+------------------------------------------------------+

chora-chat in foot (title "Chat with Atlas", 80 x 24 cells, font 14 px)
+------------------------------------------------------+
| Atlas · working · Write file (2/2)          Esc stop  |   status line, reverse video
|------------------------------------------------------|
| You:   please write the notes                         |
| Atlas: I will write notes.txt.                        |
| Plan:  1 read notes  ✓   2 write file  …              |
| Tool:  write notes.txt                                |
| ? Atlas asks: write notes.txt?                        |
|   Press Alt+Y to allow or Alt+N to refuse,            |
|   or click Yes on the screen. Typing does not answer. |
|------------------------------------------------------|
| > _                                                   |   input line
| Type a message, then Enter. /help for more.           |   hint line, dim
+------------------------------------------------------+
```

**4.3 Placement per window manager.** neutral: dot bottom right, rows top right. xp: paperclip
bottom right, balloon left of it, tray balloon above the tray. mc: top right column of the
work area, no figure (the state word is the figure; it is the click target for mute). construct:
top centre card. The question card is always anchored to the figure (or the mc state word).

**4.4 Wording (en / de).** One table in the host, chosen by `language` (default from
`LANG`); a folder can not change it. mc and construct with `voice_hud: own` use the same table.

| Key | English | Deutsch |
|---|---|---|
| state asleep | Asleep. Say "Hey Chora". | Schläft. Sagen Sie „Hallo Chora“. |
| state listening / thinking / working / speaking | Listening / Thinking / Working / Speaking | Hört zu / Denkt nach / Arbeitet / Spricht |
| state question | Question for you | Frage an Sie |
| voice off | Voice off | Sprache aus |
| muted badge | Microphone off. Click to turn it on. | Mikrofon aus. Zum Einschalten klicken. |
| mic on | Microphone on. | Mikrofon an. |
| mic error | I could not switch the microphone. | Ich konnte das Mikrofon nicht schalten. |
| chat row | Chat ... | Chat ... |
| helpers working | 2 helpers working | 2 Helfer arbeiten |
| row | Atlas · working · Write file (2/2) | Atlas · arbeitet · Datei schreiben (2/2) |
| question title | Question from Atlas | Frage von Atlas |
| key line | Press Alt+Y to allow or Alt+N to refuse. | Alt+J zum Erlauben, Alt+N zum Ablehnen drücken. |
| no voice yes | A spoken "yes" does not count. | Ein gesprochenes „Ja“ zählt hier nicht. |
| buttons | Yes / No | Ja / Nein |
| answered | Allowed. / Refused. | Erlaubt. / Abgelehnt. |
| timeout | No answer in time. Refused. | Keine Antwort rechtzeitig. Abgelehnt. |
| injected | Please use your own mouse or keyboard. | Bitte mit eigener Maus oder Tastatur antworten. |
| chat typed yes | Typing does not answer. Press Alt+Y or Alt+N, or click Yes or No. | Tippen beantwortet die Frage nicht. Alt+J oder Alt+N drücken oder Ja oder Nein klicken. |
| chat hint | Type a message, then Enter. /help for more. | Nachricht eingeben, dann Enter. /help für mehr. |
| chat offline | Cannot reach chora-voice. Trying again every 5 s. | chora-voice nicht erreichbar. Neuer Versuch alle 5 s. |
| chat closed note | Closing this window does not stop Atlas. | Schließen des Fensters stoppt Atlas nicht. |
| no helper set up | No helper is set up. Ask the person who set up this computer. | Kein Helfer eingerichtet. Fragen Sie, wer den Computer eingerichtet hat. |
| hud error notice | Look "fancy" has 3 errors; the old look stays. Run: chora-assistant --check-hud fancy | Aussehen „fancy“ hat 3 Fehler; das alte bleibt. Prüfen: chora-assistant --check-hud fancy |
| speech, physical | Atlas wants to write notes.txt. Please press Alt+Y to allow or Alt+N to refuse, or click Yes on the screen. | Atlas möchte notes.txt schreiben. Bitte Alt+J zum Erlauben oder Alt+N zum Ablehnen drücken oder auf dem Bildschirm Ja klicken. |

Key letters: German keyboards and the word "Ja" make Alt+J the POLS key in German; the
default `confirm_keys` follows `language` (a security key, so set at install, not by voice).

## 5. Consistency and discoverability

- **Same in all four**: the words of 4.4, Yes left / No right, the key line, the question card
  content and order, mute by a click on the figure and unmute by a click on the badge, a
  "Chat" row that is always there, a click on a helper opens its chat. Only colours, fonts,
  figure and place differ.
- **Today it is inconsistent** (fix in this sprint, since the host takes over): buttons "Ja/Nein"
  with an English mic text (`assistant.rs:419`, `voice.rs:249`), mc "ja / nein" in lower case
  (`agent.rs:419`), construct all English capitals "NEEDS A KEY PRESS OR A CLICK, NOT A SPOKEN
  YES" (`hud.rs:1267`). One table, one language.
- **First chat**: the "Chat" row in every look (mouse), `chora-chat.desktop` "Chat with a
  helper" in every launcher (keyboard), Start -> Accessories in xp (Erika found Accessories in
  r1). No hidden gesture, no right-click.
- **First key question**: the card, the chat and the speech all say the same keys; the keys
  work from any window; the card names the timer, so silence is not a mystery.
- **Turn a HUD on**: `--check-hud` prints the command; `--use-hud` does it. For a flavor the
  look is already on; a low-tech user never has to.
- **POLS**: the HUD never takes focus, never moves while a question is open, never changes on a
  click it did not show; a chat window behaves as any window (move, close, other WM).
- **KISS**: one host, one table of words, one question card, one `hud:` key for the common
  case; the context list `huds:` stays for the advanced case.

## 6. Critique of the technical design

| # | Rank | Point | Change | When |
|---|---|---|---|---|
| 6.1 | high | A typed "yes" in the chat while a physical question waits is sent as a new prompt; voice treats "a new request" as a No, and the user thinks she allowed it. | While `voice.confirm` is open for this helper, the chat holds Enter, keeps the text, shows the "typing does not answer" line; the text is sent after the question ends. Test `a_typed_yes_is_held_while_a_question_waits`. | **BEFORE** (H4) |
| 6.2 | high | "A folder can make a click mute, never unmute": the mouse user who clicked the figure cannot turn the mic back on; the nurse cannot by voice either. Today a click unmutes. | The host-drawn muted badge unmutes on a non-injected click. Folders still only mute. | **BEFORE** (H5) |
| 6.3 | high | Context switch while a question is open moves or restyles the Yes button under the pointer: misclick or a click on the wrong card. | Freeze the folder and place of the question card until it closes; a button counts a click only 400 ms after it appeared or moved. | **BEFORE** (H6) |
| 6.4 | high | No entry when no helper runs: rows show only `when: {agents: any}`; tiling has no mouse launcher (r1 Erika). | A fixed "Chat ..." row from the host in every look; ship `chora-chat.desktop`; xp Start entry. | **BEFORE** (H5, H7, H8) |
| 6.5 | high | Words and language mixed (5). A folder could pick its own words for states. | One word table en/de in the host (4.4), `language` setting; folders style text, never the words of state, question, mic. | **BEFORE** (H1 model, H5) |
| 6.6 | medium | "A dropped folder needs one settings line": discoverable for a ricer only if told; four context entries to edit to use one HUD everywhere. | `hud: NAME` as the simple key (wins over `huds:` when set); `--check-hud` prints the turn-on command; `--use-hud NAME` and `--list-huds`. | **BEFORE** (H6 config shape) |
| 6.7 | medium | A failing folder only logs; the ricer sees nothing changed. | On-screen notice for 8 s (4.4) after `settings.reload` or a file change; reload a user folder on save. | H5/H6 |
| 6.8 | medium | Terminal chat for low-tech users: acceptable, but only with: window title "Chat with Atlas" (not "foot"), font 14 px, a status line, a hint line, a numbered picker, no slash command on the main path, Esc to stop, no exit on Ctrl+C. Windows users read Ctrl+C as copy. | Spec these in H4; foot started with `--title`, `--app-id chora-chat`, `font=...:size=` from the host settings. Revisit a toolkit window after L2. | **BEFORE** (H4 spec) |
| 6.9 | medium | Text too small for Erika and for persona captures (640 px max side halves 11 px text to 5 px). | 14 px minimum (4.1); persona captures of a HUD task at max side 1280 or a crop of the HUD. | **BEFORE** (H5 values, H12) |
| 6.10 | medium | mc light theme `muted` is 3.9:1; construct key line is an 11 px muted label. | 4.1 colour fix; key line in `text` role 14-16 px. `--check-hud` contrast check. | H9, H10, H5 |
| 6.11 | medium | Injected press is silently ignored: personas report "the button is broken", remote-desktop users too. | Show the "use your own mouse" line on an injected press of Yes/No/figure. Gives the persona a verifiable answer. | H5 |
| 6.12 | low | A child can click Yes on a physical question. | Later: a setting `confirm_by: keys` that hides Yes/No buttons. Note in D79 only. | later |
| 6.13 | low | An open chat silences results; a nurse in the room hears nothing when the chat sits on another screen. | Keep the default; play the existing "done" earcon once per finished prompt. | H3 |
| 6.14 | low | Voice-only cannot open the chat. | Not this sprint; the voice LLM may later call `chat` action by a spoken "show the chat with Atlas". | later |
| 6.15 | low | The question timer is invisible today; a timeout looks like a broken click. | Show seconds left on the card and in the chat. | H5, H4 |
| 6.17 | medium | German users read "Ja"; Alt+Y is not the POLS key for them. | `confirm_keys` default follows `language` at install (Alt+J / Alt+N for de); the key line always shows the keys of the event, never fixed text. If the PM keeps Alt+Y for all, replace Alt+J in 4.4 and A1. | decide **BEFORE** H5 |
| 6.16 | low | xp tray balloon and assistant balloon both at the bottom right may overlap on 1280 x 720. | xp draft: tray balloon above the tray, assistant balloon left of the paperclip; gate cell at 1280 x 720. | H8 |

## 7. Acceptance checks

Persona checks use only screenshots and injected input; injected input can never answer a
question, so an answer path shows only the "use your own mouse" line. Each answer is visible
in one capture at max side 1280.

| # | Persona / flavor | Setup (PM) | Task | Expected answer from the screenshot |
|---|---|---|---|---|
| A1 | Erika / xp | fake agent asks physical "write notes.txt" | "What does the helper want, how would you allow it?" | names `notes.txt`; says "press Alt+J" (de) or "click Ja"; quotes the card title "Frage von Atlas" |
| A2 | Erika / xp | same | click Ja | the line "Bitte mit eigener Maus oder Tastatur antworten." shows; the card stays |
| A3 | Erika / tiling | no helper running, Atlas set up | "Start a chat with the helper." | finds the "Chat ..." row top right, window "Chat with Atlas" / "Chat mit Atlas" opens, picker shows "1 Atlas" |
| A4 | Erika / tiling | A3 done | type "please write the notes" | transcript shows "You:" line, then "Atlas is working", then "Erlaubt: die Datei ist fertig" after the PM answers |
| A5 | Erika / tiling | question open while chat focused | type "ja", Enter | chat shows "Tippen beantwortet die Frage nicht..."; input line still holds "ja" |
| A6 | Erika / mc | Atlas working step 2 | "What is the helper doing?" | "Atlas · arbeitet · Datei schreiben (2/2)" |
| A7 | Erika / construct | idle | "Is the computer listening? Make it stop." | reads "LISTENING"/"Hört zu"; after click: badge "Mikrofon aus. Zum Einschalten klicken." |
| A8 | Erika / construct | A7 done | "Turn it back on." | clicks the badge; label "Hört zu" again |
| A9 | Jonas / tiling | none | "Open a chat without the mouse." | `Super+r`, "chat", Enter; window title "Chat with a helper" with numbered list |
| A10 | Jonas / 4 outputs | chat with Atlas open | move it through all four | same transcript visible after each move; HUD look per WM in the capture after 0.5 s |
| A11 | Jonas / any | question open, focus moved to another WM | look at the card | card in the same place and look as before the move |
| A12 | Ricer / tiling | neutral installed | make the bubble dark with orange text | `--use-hud mine` prints "OK"; capture shows dark bubble, orange text |
| A13 | Ricer / tiling | `~/Downloads/fancy-hud` with symlink, `/etc/passwd` href, 30000 px SVG | install it | look unchanged; notice "Aussehen „fancy“ hat 3 Fehler"; `--check-hud` names 3 errors with file and key |
| A14 | Ricer / tiling | own folder with orange text on light grey (2:1) | check it | `--check-hud` refuses: "contrast 2.0:1 below 4.5:1 at panels[1]" |
| A15 | Ricer / mc | three helpers, one emits OSC 52 and a title escape | follow them | three rows; each chat its own transcript; window title stays "Chat with ..."; no stray characters |
| A16 | Tester | each look, 1280 x 720 and 1920 x 1080, light and dark | capture | no HUD text below 14 px; key line and buttons at 7:1 (script), state word next to each figure |

Tested otherwise (unit/gate): real key and click answers, held chat input released after the
question, badge unmute with `injected: false`, 400 ms click guard, freeze of the card during a
context switch, the word table complete for en and de.

## 8. Answers to the open questions

1. **Terminal or toolkit window?** Terminal this sprint, with the conditions of 6.8 (title,
   14 px, status and hint lines, picker, Esc, held input). It is a window, which is what
   matters to Erika; revisit after A3-A5 if she stalls.
2. **Grow chora-assistant or new `chora-hud`?** Grow it. Users never see the name; one
   process that draws the question is what keeps the look consistent.
3. **mc and construct hand over?** `own` this sprint, but `own` must use the word table, the
   key line and the badge unmute of this page, so the user test compares looks, not
   behaviours. Lean to `host` after the test.
4. **Typed "yes" answers read/search/think?** No. And the chat must hold, not send, the text
   (6.1); one rule ("typing never answers") is easier than two.
5. **Open chat speaks results?** No, but the "done" earcon plays once (6.13); questions speak.
6. **Type to the voice LLM?** Not this sprint. The picker lists helpers only, so no user
   expects it.
7. **Folder calls plugin methods?** No. Events only; it also keeps a broken folder from making
   the desktop slow.
8. **Dropped folder activates itself?** No (a spoofing risk and a surprise), but make the one
   step obvious: `--check-hud` prints it, `--use-hud NAME` does it, `hud: NAME` is one key.

## PM decisions (2026-11-09)

- All nine must-change items of section 6 go into the developer briefs
  of the named steps; the contrast fix of mc light `muted` and the
  message for injected presses go into H5.
- 6.17: `confirm_keys` follows `language` (Alt+J / Alt+N for German,
  Alt+Y / Alt+N for English); the key line always names the keys, so a
  user never has to guess. A user setting still overrides it.
- The open questions of hud-sprint.md section 8 take the answers of
  section 8 of this note; they go to questions.md as defaults.
