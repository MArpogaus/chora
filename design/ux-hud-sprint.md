> SUPERSEDED (11-11): sections 2-4 are replaced by design/looks/.

# UX pass: feature sprint 1 (ACP chat and HUDs)

UX/UI review of `design/hud-sprint.md`, made before programming; revised for the user's request
"Acp chat shall be themed and integrated in corresponding ui" and checked against
`docs/design-principles.md` (Minimal, flawless, flexible). Read on 2026-11-09 with
`chora-assistant`, `chora-mc/src/agent.rs`, `chora-construct/src/hud.rs`, the style guides,
`docs/voice.md` sections 5 and 10, the persona runs r1 and r2 and `media/2026-11-09/`. Section 6
marks what must change before programming (**BEFORE**). "P3" means principle 3 of that page.

## 1. Users and goals

| User | Input | Goal in this sprint | What breaks them |
|---|---|---|---|
| Erika, 74, tech 0 | mouse, types slowly | see that a helper wants something, allow or refuse it, write one line to a helper | no visible entry (r1 tiling: "no menu"), no feedback after a click (r1 xp), words like "agent", `/start` |
| Nurse, hands busy | voice only | hear what helpers do, open and close the chat, mute, know when a hand is needed | a question that needs a key and does not say so; a muted mic she cannot unmute by voice |
| Child, 9 | mouse, some typing | talk to the figure, read what it says | small text; a Yes that allows things a parent must allow (later, 6.14) |
| Jonas, 31, tech 3 | keyboard first | open a chat, follow 3 helpers, never leave the keyboard | a feature only a click reaches; a chat that keeps the keyboard |
| Ricer (r/unixporn) | keyboard, editor | ship a HUD folder that also themes the chat; see errors at once | silent failure, four context lines to edit, a chat look he cannot change |

Words on screen: **helper** / **Helfer**, never "agent", "ACP" or "HUD"; German uses "Sie"
(P4, P7). Ricers read "HUD" only in docs and in `--check-hud`.

## 2. Flows

M = mouse, K = keyboard, V = voice. "Host" is `chora-assistant`, which draws the chat in every
look (hud-sprint section 1). The chat is a view plus one text field; it is no window.

**F1 Open a chat.**
- M: click "Chat ..." (always there) or a helper's row. xp: also a click on the balloon. mc:
  also a click on a helper's column in the agents view. The chat opens in the look of the
  focused output (2.1) with the field focused.
- K: `Super+a` in tiling; the launcher in every look (`Super+r`, Start, the mc app list) ->
  "Chat with a helper" -> Enter. Without a name: a numbered list; `1`, Enter.
- V: "Hey Chora, show the chat with Atlas" (the LLM calls `assistant chat.show`; not
  privileged). "Close the chat" closes it.

**F2 Write and read.** Type, Enter. The line shows under "You", then "Atlas · working · step",
then the answer under "Atlas". Shift+Enter breaks a line; paste with Ctrl+V and IME work.
"Stop" (button) cancels the running prompt; "Close" or Esc closes the chat and keeps the draft;
the helper goes on (the view says so on close). V: speaking to the helper works as today.

**F3 Answer a question that needs a key.** The card shows inside the chat if the chat was open
when it was asked, else at the figure; it stays there until it ends. Speech: "Atlas wants to
write notes.txt. Please press Alt+Y to allow or Alt+N to refuse, or click Yes on the screen."
- M: Yes or No on the card. K: Alt+Y / Alt+N from any window, also while the field has the keys.
- V: a spoken "yes" does not count ("For this I need a key press or a click."); "no" refuses.
- Typed "yes" in the field: not sent, held, hint line (4.4); sent after the question ends.

**F4 Normal question (read, search, think).** Same card plus V yes/no; typing still never answers.

**F5 Mute and unmute.** M: click the figure -> badge "Microphone off. Click to turn it on.";
click the badge -> on. V: "mute"; before muting, speech says "Click me to turn it on." K: a
binding per flavor to `voice mute` toggle (6.12).

**F6 Context switch.** Focus moves to an xp window -> within 0.5 s the xp look shows on that
output; an open chat is redrawn as the xp balloon with the same transcript and draft. No switch
while the field has the keyboard or a question waits. No focus change, no sound.

**F7 xp.** Paperclip bottom right, balloon with the sentence, tray balloon "2 helpers working";
a click turns the balloon into the chat balloon (340 wide, up to 60 % of the work area).
**F8 mc.** Top right text HUD; the agents view shows one column per helper; a click on a column
or a row opens the chat in the drawer under it, mc-style.
**F9 construct.** Glass card top centre with ring, label, sentence, chips; a chip opens the chat
as a glass panel on the right.
**F10 tiling.** Dot bottom right, rows top right; the chat is a side panel at the right edge.

**F11 Install a shipped HUD (ricer, K).** `cp -r ~/Downloads/fancy-hud
~/.local/share/chora/huds/fancy` -> `chora-assistant --check-hud ...fancy` ("OK" plus the
command to turn it on, or errors with file and key) -> `chora-assistant --use-hud fancy` -> the
look, and the chat look of its `chat:` block, change on all outputs. Saving `hud.yaml` reloads;
an error keeps the old look and shows a notice for 8 s. Undo: `--use-hud default`.

### 2.1 What happens to an open chat on a switch

It follows the user; it is not a window. Reasons: a window would carry one look into another
(the xp balloon inside mc) and break P8; the conversation lives in voice and the view in the
host, so a switch is a restyle with the same lines and the same draft. Windows move between
window managers exactly as without the chat (P1); the field has no claim, so a drop over it lands
in the window manager under it.

## 3. States

| Element | empty | busy | success | error | refused | offline | question waits |
|---|---|---|---|---|---|---|---|
| Figure / mic | idle pose + word | `thinking`/`working` pose + word | `speaking` + sentence | "I could not switch the microphone." | injected press: "Please use your own mouse or keyboard." | grey, "Voice off" | `asking` pose, "Question for you" |
| Muted badge | - | - | fixed host badge | - | - | - | stays above the card |
| Helper rows | only "Chat ..." | "Atlas · working · Write file (2/2)" | "Atlas · done" 10 s | "Atlas · stopped: <reason>" | "Atlas · not allowed" | hidden; "Chat ..." stays | "Atlas · has a question" |
| Question card | - | - | "Allowed." / "Refused." 2 s | "No answer in time. Refused." | injected press: line, card stays | "Question cancelled." | prompt, detail (scrolls), key line, Yes, No, time left |
| Chat view header | "Chat with a helper" + numbered list; none: "No helper is set up. Ask the person who set up this computer." | "Atlas · working · step (2/2)" + Stop | "Atlas is ready." | "Atlas stopped: <reason>" | "This chat may not run here." | "Cannot reach chora-voice. Trying again every 5 s." | "A question is waiting." (points to the card) |
| Chat field | "Type a message, then Enter." (dim) | Enter queues: "(waiting: 1)" | line moves to the transcript | "Not sent: <reason>", text kept | - | text kept, not sent | text held, hint line |
| HUD folder | - | - | "OK. Turn it on: chora-assistant --use-hud fancy" | `hud.yaml:12 chat.size: 11 is below 14` | "refused: picture.svg links outside the folder" | - | - |

## 4. Layout, contrast, text, wording

**4.1 Rules the host enforces in every look (P7, P8).**
- Text a person must read: 14 px or more (sentence, rows, card, chat lines, field). Labels of
  a state that also has a shape or word: 11 px or more. Today the bubble is about 11 px
  (`assistant.rs:39`) and the construct key line is an 11 px muted label: both change.
- Contrast: text 4.5:1 on its real background; key line and buttons 7:1. `--check-hud` computes
  it for every colour pair of a folder, `chat:` included, and refuses below. mc light `muted`
  [0.50, 0.48, 0.45] on the page is 3.9:1: use [0.43, 0.41, 0.38] (5.0:1) for HUD and chat.
- Targets 32 x 24 px or more; Yes left, No right, 8 apart; a click counts 400 ms after a button
  appeared or moved. The state is never colour alone.
- Agent text sits under the helper's name in the message style; only voice's `question` entries
  draw a card (P5).

**4.2 Sketches (1280 x 720 logical).**

```
tiling: side panel 400, full work-area height       xp: chat balloon 340, above the paperclip
+---------------------------------+--------------+  +---------------------------------------+
| bar 24                          | Atlas        |  |            +------------------------+ |
|                                 | working (2/2)|  |            | Atlas        [Stop][x] | |
|                                 | [Stop] [x]   |  |            | You: please write the  | |
|   windows (work area shrinks    |--------------|  |            |   notes                | |
|   by nothing: the panel floats  | You:         |  |            | Atlas: I will write    | |
|   over the right edge)          | please write |  |            |   notes.txt.           | |
|                                 | Atlas:       |  |            | [ Type a message ...  ]| |
|                                 | I will write |  |            +-----------\\-----------+ |
|                                 |[Type a mess.]|  |                    (paperclip)        |
+---------------------------------+--------------+  | [start] [foot]               6:51    |
 sans 14, line 20, bar colours, field 32 high        +---------------------------------------+
                                                       Tahoma 14, XP edit box 1 px #7F9DB9

mc: drawer under the agents view (14 lines)          construct: glass 480, right, 32 margin
+------------------------------------------------+  +---------------------------+---------+
| 4 | AGENTS                                     |  |                           |CHAT·    |
| Atlas ● working     Bert ○ idle                |  |       panes ...           |ATLAS    |
|  Write file (2/2)                              |  |                           |You      |
|------------------------------------------------|  |                           | please..|
| Atlas                              Stop  Close |  |                           |Atlas    |
| You   please write the notes                   |  |                           | I will..|
| Atlas I will write notes.txt.                  |  |                           |[field ] |
| _ Type a message, then Enter.                  |  +---------------------------+---------+
+------------------------------------------------+   Inter 16 text, 11 labels, padding 16,
 Pagella body 18, Inter labels step -1, hairline     glass and edge of construct-style, glow
 above, no box, baseline grid                        as a picture (no shader in a folder)

question card (same order everywhere; look per folder)
| Question from Atlas                     0:42 |  title 14, timer tabular
| Atlas wants to write the file notes.txt.     |  prompt 16
| ~/projects/notes/notes.txt (412 bytes)  ▲▼   |  detail mono 14, scrolls, 8 lines
| Press Alt+Y to allow or Alt+N to refuse.     |  key line 14, text colour, 7:1
| A spoken "yes" does not count.               |
|                         [  Yes  ] [  No  ]   |  buttons 80 x 32
```

**4.3 Placement.** The card sits inside an open chat or next to the figure (mc: under the state
word), never off the work area, width min(560, output - 64). The field is the bottom of the view.

**4.4 Words (en / de)**, one table in `chora-kit::words`, chosen by `language` (default `LANG`).
Folders and looks never change them (P8).

| Key | English | Deutsch |
|---|---|---|
| asleep | Asleep. Say "Hey Chora". | Schläft. Sagen Sie „Hallo Chora“. |
| listening / thinking / working / speaking | Listening / Thinking / Working / Speaking | Hört zu / Denkt nach / Arbeitet / Spricht |
| question | Question for you | Frage an Sie |
| voice off | Voice off | Sprache aus |
| muted | Microphone off. Click to turn it on. | Mikrofon aus. Zum Einschalten klicken. |
| mic on / error | Microphone on. / I could not switch the microphone. | Mikrofon an. / Ich konnte das Mikrofon nicht schalten. |
| chat row / launcher | Chat ... / Chat with a helper | Chat ... / Mit einem Helfer chatten |
| helpers | 2 helpers working | 2 Helfer arbeiten |
| row | Atlas · working · Write file (2/2) | Atlas · arbeitet · Datei schreiben (2/2) |
| chat buttons | Stop / Close | Anhalten / Schließen |
| you | You | Sie |
| card title | Question from Atlas | Frage von Atlas |
| key line | Press Alt+Y to allow or Alt+N to refuse. | Alt+J zum Erlauben, Alt+N zum Ablehnen drücken. |
| no voice yes | A spoken "yes" does not count. | Ein gesprochenes „Ja“ zählt hier nicht. |
| buttons | Yes / No | Ja / Nein |
| answered / timeout | Allowed. / Refused. / No answer in time. Refused. | Erlaubt. / Abgelehnt. / Keine Antwort rechtzeitig. Abgelehnt. |
| injected | Please use your own mouse or keyboard. | Bitte mit eigener Maus oder Tastatur antworten. |
| typed yes | Typing does not answer. Press Alt+Y or Alt+N, or click Yes or No. | Tippen beantwortet die Frage nicht. Alt+J oder Alt+N drücken oder Ja oder Nein klicken. |
| waiting | A question is waiting. | Eine Frage wartet. |
| field hint | Type a message, then Enter. | Nachricht eingeben, dann Enter. |
| offline | Cannot reach chora-voice. Trying again every 5 s. | chora-voice nicht erreichbar. Neuer Versuch alle 5 s. |
| close note | Atlas goes on working. | Atlas arbeitet weiter. |
| no helper | No helper is set up. Ask the person who set up this computer. | Kein Helfer eingerichtet. Fragen Sie, wer den Computer eingerichtet hat. |
| hud notice | Look "fancy" has 3 errors; the old look stays. Run: chora-assistant --check-hud fancy | Aussehen „fancy“ hat 3 Fehler; das alte bleibt. Prüfen: chora-assistant --check-hud fancy |

The key line shows the keys of the event, never fixed text; Alt+J for German is 6.13.

## 5. Consistency, discoverability, and the three words

- **Same in all four looks**: words, Yes left / No right, the key line, the card, mute on the
  figure and unmute on the badge, "Chat ..." always there, Esc closes the chat, Enter sends,
  Stop cancels. Colours, fonts, shapes and place differ (P1, P8).
- **Today inconsistent** (fixed by the host and the word table): "Ja/Nein" buttons with English
  mic text (`assistant.rs:419`, `voice.rs:249`), mc "ja / nein" (`agent.rs:419`), construct
  English capitals (`hud.rs:1267`).
- **Visible way in (P3)**: "Chat ..." in every look; launcher entry; xp Start -> Accessories.
- **Minimal**: one chat core, one host view, one field program, one card model. mc and construct
  write no chat code; mc only reserves its drawer. No terminal, no second window type.
- **Flawless**: one renderer means one behaviour; `gate-chat.sh` plays one event script through
  all four folders and checks the same lines; the card never moves; typing never answers.
- **Flexible**: the `chat:` block of a folder themes the view and field of every look, mc and
  construct included; a ricer ships it with the folder. Values only (P10).

## 6. Critique of the technical design

| # | Rank | Point | Change | When |
|---|---|---|---|---|
| 6.1 | high | A typed "yes" while a question waits would go as a new prompt, which voice reads as a No. | The field holds Enter and the text, shows the hint, sends after the question ends. | **BEFORE** (H4a, H4b) |
| 6.2 | high | "A folder can make a click mute, never unmute" strands the mouse user and the nurse. | The host badge unmutes on a real click; folders only mute. | **BEFORE** (H5) |
| 6.3 | high | A context switch or an opened chat moving the card puts Yes under the pointer (P6). | The card stays where it appeared; no context switch while it waits or while the field has the keys; 400 ms click guard. | **BEFORE** (H1 `ask`, H6, H7) |
| 6.4 | high | No way in without a running helper (rows only `when: agents any`). | Fixed "Chat ..." row; launcher entry; `Super+a` in tiling. | **BEFORE** (H5, H7) |
| 6.5 | high | Words mixed; folders or looks could word states differently. | `chora-kit::words` en/de, `language`; folders style only. | **BEFORE** (H1) |
| 6.6 | high | A field that takes the keyboard on every map could steal focus (P5, POLS). | Keyboard only with an activation token of a person's act (H4c), never on a context switch. | **BEFORE** (H4c) |
| 6.7 | high | An agent message can imitate a question ("? Atlas asks ... press Alt+Y") (P5). | Cards come only from voice `question` entries via `chora-kit::ask`; agent text always under the name in message style. Test `agent_text_never_draws_a_card`. | **BEFORE** (H4a, H7) |
| 6.8 | high | Four views written per look would drift (flawless). | One host view styled by `chat:`; mc reserves a box (`slot: mc`). | **BEFORE** (done in hud-sprint section 1) |
| 6.9 | medium | `hud: NAME` missing; four context lines to use one HUD. | `hud: NAME` key, `--use-hud`, `--list-huds`; `--check-hud` prints the command. | **BEFORE** (H6 config) |
| 6.10 | medium | A failing folder only logs. | 8 s notice; reload a user folder on save. | H5, H6 |
| 6.11 | medium | Text too small for Erika and for persona captures at 640 px. | 14 px minimum; HUD and chat captures at max side 1280. | **BEFORE** (H5, H12) |
| 6.12 | medium | No keyboard mute (P2). | Each flavor binds `voice mute` toggle (tiling `MOD+Shift+a`); not in this sprint's files if the PM prefers. | H12 or next |
| 6.13 | medium | German readers expect Ja; Alt+Y is not the POLS key. | `confirm_keys` default follows `language` at install. PM decides; else replace Alt+J. | decide **BEFORE** H5 |
| 6.14 | low | A child can click Yes. | Later: `confirm_by: keys`. | later |
| 6.15 | low | xp tray balloon and chat balloon could overlap at 1280 x 720. | Chat balloon replaces the tray balloon while open. | H8 |

## 7. Acceptance checks

Personas use screenshots and injected input; injected input never answers a question, so the
answer path shows only the "use your own mouse" line. Captures at max side 1280.

| # | Persona / look | Setup (PM) | Task | Expected answer from the screenshot |
|---|---|---|---|---|
| A1 | Erika / xp | physical question "write notes.txt" | "What does the helper want, how would you allow it?" | `notes.txt`; "Alt+J" or "Ja"; title "Frage von Atlas" |
| A2 | Erika / xp | same | click Ja | "Bitte mit eigener Maus oder Tastatur antworten."; card unchanged |
| A3 | Erika / tiling | no helper running | "Start a chat with the helper." | clicks "Chat ...", list "1 Atlas", side panel "Atlas" opens |
| A4 | Erika / tiling | A3 | type "please write the notes" | "Sie: please write the notes", "Atlas · arbeitet", then "Erlaubt: die Datei ist fertig" |
| A5 | Erika / tiling | question waits, field focused | type "ja", Enter | "Tippen beantwortet die Frage nicht..."; field still holds "ja" |
| A6 | Erika / xp | Atlas idle | "Ask the paperclip in writing what it does." | the balloon is the chat; answer in it |
| A7 | Erika / mc | Atlas working step 2 | "What is the helper doing? Write to it." | "Atlas · arbeitet · Datei schreiben (2/2)"; click on the column opens the drawer chat |
| A8 | Erika / construct | idle | "Is it listening? Make it stop, then on again." | "Hört zu"; badge "Mikrofon aus. Zum Einschalten klicken."; then "Hört zu" |
| A9 | Jonas / tiling | none | "Open a chat without the mouse, then leave it." | `Super+a` opens the panel with the list; Esc closes it; the terminal has the cursor again |
| A10 | Jonas / 4 outputs | chat open, draft "half" typed | go through all four looks, moving a window each time | chat in each look with the same lines and "half"; each window lands where dropped |
| A11 | Jonas / any | question waits, focus moves | look at the card | same place and look as before |
| A12 | Ricer / tiling | neutral copy | dark chat panel, orange names | `--use-hud mine` "OK"; capture shows it; words and buttons unchanged |
| A13 | Ricer / tiling | fixture: symlink, `/etc/passwd` href, 30000 px SVG, `chat.size: 10` | install | look unchanged; notice "... hat 4 Fehler"; `--check-hud` names 4 errors |
| A14 | Ricer / mc | three helpers, one emits OSC 52, title escape, U+202E | follow them | three rows; each chat its own lines; no stray or reversed text |
| A15 | Tester | four folders, 1280 x 720 and 1920 x 1080, light and dark | `gate-chat.sh` | same visible lines in all four; no text below 14 px; 4.5:1 and 7:1 by script |

Tested otherwise: real key and click answers, held text released after a question, badge unmute
with `injected: false`, the token rule of the field, the 400 ms guard, the frozen card, the word
table complete for en and de.

## 8. Answers to the open questions

1. **Terminal or toolkit window?** Neither; decided by the user: a view inside each look,
   drawn by the host from the folder's `chat:` block, with one layer-surface field.
2. **Grow chora-assistant or new `chora-hud`?** Grow it; it is now also the chat host. One
   process for HUD, card and chat keeps every look the same.
3. **mc and construct hand over?** `own` for the voice HUD this sprint, but the chat and the
   card come from the host and `chora-kit` in every setting, so the test compares looks, not
   behaviours.
4. **Typed yes for read/search/think?** No; one rule, "typing never answers" (6.1).
5. **Open chat speaks results?** No; questions still speak. No extra earcon (P6, Calm).
6. **Type to the voice LLM?** Not this sprint; the list shows helpers only.
7. **Folder calls plugin methods?** No; events only (P10).
8. **Dropped folder activates itself?** No; `--check-hud` prints the step, `--use-hud` does it.
