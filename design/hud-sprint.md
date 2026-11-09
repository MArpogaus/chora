# Feature sprint 1: ACP chat and HUDs

Design of the first feature sprint. Read on 2026-11-09 at `92e7539` and on `claude/acp-gate`
(`42826c0`). References are `path:line` or `path` in the chora repository. Values: POLS, KISS,
lean core, reuse. The request: ACP chats; HUD drafts for mc, xp and construct; a voice HUD that
adapts to context; users ship their own HUDs; user tests.

## 0. Today

* Four voice HUDs, each drawn by its own code: the dot of `voice` itself (`NAME-hud`), the
  figure and bubble of `chora-assistant` (asset folder with `values.yaml` and `dot.svg`; XP
  flavor passes `flavors/xp/assets/assistant`), the regions, agents scene and notifications of
  `chora-mc` (`agent.rs`, `region.rs`), the ring, line and question glass of `chora-construct`
  (`hud.rs`, 1,784 lines). Each subscribes to `voice.*`, checks `client == voice`, claims the
  indicator and answers with `confirm.answer`, ignoring injected presses.
* ACP agents live in `services/voice/src/agents.rs`: one sandboxed agent per project, events
  `voice.state {name, state, task, said, plan, question, log}` with the last 20 transcript lines
  of 300 chars, `voice.remove`. The method `voice.agent {tool, ...}` (privileged) runs
  `agent_list|start|say|stop|status` without the LLM (`client.rs:562`, `main.rs:256`).
* On `claude/acp-gate`: a permission request of kind `read`, `search` or `think` is a normal
  question; any other kind is physical (key of `confirm_keys` or a HUD button); an always word
  answers nothing; `allow_always` only when it is the only allowing option and the question
  says "always". This sprint builds on it and changes none of it.
* Constraints that shape the design: a confined connection cannot create nodes under
  `screen:*` and names only its own `assets.put` files (`docs/api.md` Client sockets). Scene
  nodes take keys through `focus_node`, but have no IME, no paste and no text editing (D18,
  D56). Any client may `events.emit` under its own name (D65).

## 1. ACP chat

**What the user does.** Opens a chat with an ACP agent (a "helper" on screen), types prompts,
reads the full transcript (messages, plan, tool calls, questions), stops a prompt, starts and
stops an agent. Voice may run beside it (the same agent answers spoken and typed prompts, one
queue) or be silent (mic and speaker muted, ASR, TTS and LLM servers down: agents still work,
they only need `sandbox_cmd` and `agent_cmd`).

**Where it lives: inside each look, one view drawn by the host.** The user asked for a chat
that is themed and part of each window manager's own UI (mc agents view, the xp assistant).
Minimal, flawless, flexible gives one shape: one chat core, one view drawn by the host
(`chora-assistant`), and the look of the view in the HUD folder of each look.

| Look | The chat looks like | Place |
|---|---|---|
| tiling (`neutral`) | a plain side panel in the bar's colours | right edge of the work area, 400 wide, full height |
| xp (`xp`) | the assistant's balloon opens into a chat balloon (Tahoma, XP edit box) | above the paperclip, 340 wide, up to 60 % of the work area |
| mc (`mc`) | a page in the drawer under the agents view, mc-style: Pagella body, Inter labels, hairlines, no box | the drawer box that mc reserves (`chat.slot`) |
| construct (`construct`) | a glass panel after `construct-style.md` (glass, edge, glow as a picture) | right, 480 wide, 32 from the edges |

* **Minimal.** Voice keeps the conversation (methods below). `chora-kit::chat` is the model:
  entries, pull after `seq`, wrap, the sanitizer, the held-input rule; `chora-kit::words` is
  the en/de word table; `chora-kit::ask` is the question card model (order, key line, the
  injected and 400 ms guards). The host draws the view and the card from these. mc and
  construct write no chat code: mc only reserves its drawer and announces the box.
* **Flawless.** One renderer and one field program, so every look behaves the same by
  construction; a look changes colours, fonts, shapes and place, never words, keys, order
  or what a click does. One gate script drives all four looks with the same events and checks
  the same visible lines (H7).
* **Flexible.** The `chat:` block of `hud.yaml` styles the view and the field with values only
  (style `plain|bubble|glass`, colours, font, size from 14 up, place `anchor`, `width`, or
  `slot: CLIENT` for a box a plugin announces). A ricer ships the look of the chat with the
  folder. No text input, no words, no code in a folder.
* **Session owner: the host.** It holds the one open chat `{name}` and the voice `chat.open`
  for it. Host methods: `chat.show {name?}`, `chat.hide` (not privileged; the voice LLM may
  call them, so "show the chat with Atlas" works by voice). It emits `assistant.chat {open,
  name, output}`; mc opens its drawer on it and emits `chat.slot {output, rect}` (D65).
* **Text input: one field program, a layer surface.** Scene nodes have no IME, no paste and
  no editing (D18, D56), so the field is a small Wayland client: `chora-chat --input`,
  spawned by the host, one at a time. A `wlr-layer-shell` surface on layer `top`, keyboard
  interactivity `on_demand`, at the bottom of the view. It draws itself with `chora-draw` into
  `wl_shm` (`wayland-client` is already a dependency; no toolkit), uses `text-input-v3` for IME
  (the core has it) and the data device for paste. One line, growing to 4; Enter sends
  (`voice.agent {tool: agent_say}`), Shift+Enter breaks a line.
* **Keyboard in and out.** The host opens the chat only on a person's act (a click on a row or
  the "Chat ..." row, `chora-chat` from a launcher or a key, a spoken request) and hands the
  field an xdg-activation token of that act (D72). New core rule (H4c): an `on_demand` layer
  surface that activates with a valid token gets the keyboard, as a window does; without a
  token it gets nothing. A click on the field takes the keyboard (exists, `state.rs:787`); a
  click on a window gives it back (exists). Esc closes the chat (view and field) and keeps the
  draft for the next open: one Esc backs out, as in a launcher. The view has "Stop" (cancels
  the running prompt, `agent.cancel`) and "Close".
* **Opening without a NAME.** `chora-chat` without `--input` is the opener for launchers and
  key bindings: it calls `assistant chat.show {}`. With no agent named, the view shows the
  numbered list of agents and projects (`agent_list`); a click or `1` + Enter picks one.
* New in voice (small, in `agents.rs` and `client.rs`):
  * each agent keeps a ring of 500 transcript entries `{seq, role: user|agent|tool|question|plan,
    text}`; `voice.state` of an agent carries `seq`, so a reader knows when to pull;
  * `agent.read {name, after}` answers the entries after `seq` (at most 64 KiB);
  * `agent.cancel {name}` (today only the spoken cancel word reaches `Agents::cancel`);
  * `chat.open {name}` / `chat.close {name}` mark a chat as open, also dropped on
    `client.disconnected`, as `indicator.claim` is; the agent record gets `chat: true`;
  * an agent with an open chat does not speak its results, only its questions (open question 5).
  All are privileged and refused for a confined caller (`not_for_confined`).

**Look switch and window moves: the chat follows the user, it is not a window.** When the
context changes (focus to a window of another window manager or another output), the host
draws the same chat with the folder of the new look; the field re-anchors and keeps its draft
(same process). Reasons: the user asked for each look's own UI, so a window that carries the
xp balloon into mc would break that look; the conversation lives in voice and the view in one
host, so a switch is a restyle, not a move; and the chat adds no window to the handover
matrix, so window moves stay exactly as they are. The context does not change while the field
has the keyboard or a question waits (H6). The field is a layer surface without a claim, so
`claimant_at` never picks it and a drop over it lands in the window manager under it (H11).

**Questions.** A question shows as the host's card (`chora-kit::ask`): inside the chat view
when the view is open at the moment it is asked, otherwise at the figure; it stays where it
appeared until it ends (a chat opened later shows the line "A question is waiting." pointing to
the card). Never twice. Agent text never draws a card: the view draws `question` entries of
voice by role, and an agent's message is always under the helper's name in the message style.

**Reuse of the gate.** The chat sends text and reads text. Every ACP permission request still
goes the one way of `acp-gate`: the core asks, the key or a button answers. The field answers
no question: it cannot tell typed text from pasted or injected text, so a typed "y" is no
proof (same argument as for the spoken yes). While a question of its agent waits, the field
holds Enter, keeps the text and shows the "typing does not answer" line; the text goes after
the question ends. `agent_start` through the chat runs as `voice.agent` does today.

**What it may not do.**

* The field program: raise or change grants, answer `confirm.answer`, claim the indicator,
  call `talk` or `mute`. The card's buttons stay the host's, which ignore an injected press.
* Pass a chora socket, MCP server or file system to the agent: `session/new` keeps
  `mcpServers: []` and `initialize` keeps no fs and no terminal.
* Show control characters of agent text. ACP text is untrusted: `chora-kit::chat` drops C0
  except `\n` and `\t`, all C1, ESC, DEL and the bidi controls (U+202A to U+202E, U+2066 to
  U+2069) before a text node gets it. No links, no "run this" action, no markup.
* Serve a confined caller: a confined client keeps today's `voice.*` events (20 lines), and no
  full transcript.

## 2. One HUD contract

### Option A: HUD folder drawn by one host plugin (recommended)

A HUD is a folder: `hud.yaml`, `README.md`, SVG pictures, optional `fonts/`. The asset folder of
D73 grows a manifest; no code. One trusted host plugin draws it: `chora-assistant`, which is the
voice HUD block already (D73) and runs on the main socket, so it may draw under `screen:*`.

```yaml
hud: 1                       # contract version; another number is refused
font: sans-serif
panels:
  - id: figure
    source: voice            # voice | say | agents | chat | context | clock | event
    when: {wm: [xp]}         # optional; all keys must match
    anchor: bottom_right     # 9 positions in the work area of the output
    margin: [16, 48]
    picture: dot.svg#{state} # SVG element id from a field; a missing id shows `idle`
    action: mute             # mute | cancel | sleep | wake | chat; nothing else
  - id: line
    source: say
    near: figure             # placed beside another panel, side by its anchor
    style: bubble            # bubble | plain, colors as values.yaml today
    text: "{text}"
    max_chars: 160
  - id: agents
    source: agents
    anchor: top_right        # no `when`: the host adds a fixed "Chat ..." row, also with no agent
    rows: "{name}  {state}  {task}"
    max_rows: 4
    action: chat             # a click on a row calls the host's chat.show for that agent
```

* **Sources** are a closed list: `voice` (state, awake, listen, muted), `say` (the sentence of
  `voice.say`), `agents` (sub-agents of `voice.state` with `name`), `chat` (open chats),
  `context` (below), `clock`, and `event: CLIENT.EVENT` for any D65 event of a named client,
  checked for the exact `client`. A user who needs computed data ships an ordinary plugin that
  emits an event (it may be confined; emitting needs no capability), and the folder shows it.
* **Templates** are `{field}` and `{field.sub}` only; a value is inserted as plain text and is
  never expanded again. `when` compares a field with a value or a list. No expressions, no
  loops beyond `rows`, no method calls, no scripting.
* **Text** goes into core text nodes (D18) when `style: plain`, so a change costs no raster;
  `bubble` uses `chora-draw::bubble` as today. Pictures are rasterized once per state and scale.
* **The chat look** is one optional block `chat:` (section 1): `style: plain|bubble|glass`,
  colours, `font`, `size` (14 or more, else refused), `anchor` and `width`, or `slot: CLIENT`.
  Values only: the host draws the view, the words come from `chora-kit::words`, the field from
  `chora-chat --input`. A folder without the block gets the neutral chat.
* **The question is not part of the folder.** The host draws `voice.confirm` itself: prompt,
  the full detail with scroll, Yes and No, the key line of a physical question, on top of every
  panel. A folder styles it (colors, font, corner) and cannot hide, cut, relabel or move it off
  the output. The same for the mic: in each context a panel with `source: voice` must be visible
  (inside the output, opacity at least 0.5, at least 12 by 12 px); otherwise the host adds the
  neutral dot. A folder can make a click mute, never unmute: `mute` toggles only on a
  `source: voice` panel, where the host draws a fixed muted badge.
* **Where folders live.** Shipped: `/usr/share/chora/huds/NAME` (source tree:
  `plugins/chora-assistant/huds/neutral`, `flavors/xp/assets/assistant`,
  `plugins/chora-mc/hud`, `plugins/chora-construct/hud`). User: `$XDG_DATA_HOME/chora/huds/NAME`,
  which wins over a shipped folder of the same name. Activation is one settings line (D62), so a
  dropped folder changes nothing until the user names it.
* **Checking.** `chora-assistant --check-hud DIR` prints each error with file and key and exits
  1; `settings.reload` keeps the running HUD when a folder fails and logs the error; a test loads
  each shipped folder.
* **Code split.** The pure model (manifest types, loader limits, context, `when`, templates) is
  the module `chora-kit::hud`. Drawing stays in `chora-draw`. The host is glue. A user who needs
  more than the contract writes an ordinary plugin on the same module (path of option B).

### Option B: a HUD is a small plugin on a `chora-kit::hud` module

Each HUD is a Rust plugin: the kit gives the voice subscription, the question widget, the
context and the placement; the author writes the rest. Shipping is a build, a package or a unit,
a manifest and a client name.

* For: no contract to design, any logic, one process per HUD.
* Against: a HUD under `screen:*` needs the main socket, so every shipped HUD is fully trusted
  code; a confined socket cannot draw it. Every HUD carries its own copy of the question and
  mic rules, which a reviewer must check per HUD. Users need a toolchain. Several HUD processes
  claim the indicator and each draws the question.

**Recommendation: A.** It is the smallest thing a user ships by dropping a folder, it keeps the
security rules in one host, and its model module also serves the rare author of option B. No
new engine, no scripting; the only core change is the keyboard rule of the chat field (H4c).

## 3. Context-dependent voice HUD

The host keeps one context record and re-evaluates on each change, debounced 150 ms, with the
cross-fade that the assistant has.

| Field | From | Values |
|---|---|---|
| `wm` | `focus.changed`, then the owner of the focused surface (`scene.query`); no focus: the claimant of the output under the pointer | client name: `tiling`, `xp`, `mc`, `construct`, ... |
| `space` | the claimant of the real output that shows the focused window; differs from `wm` when a window manager runs in a construct pane | client name |
| `app` | `app_id` of the focused surface | `foot`, `firefox`, ... (the chat field is a layer surface, not an app) |
| `state` | `voice.state` of `voice` | `asleep`, `listening`, `thinking`, `working`, `question`, `speaking` |
| `agents` | sub-agents | `none`, `any`, `working`, `question` |
| `chat` | `chat: true` of an agent | `open`, `none` |
| `output` | the output of the focused window, else of the pointer | output name |

Two levels decide what shows:

1. **Which folder**: the settings of the host map context to folders, first match wins.

   ```yaml
   huds:
     - {when: {space: construct}, folder: construct}
     - {when: {wm: mc}, folder: mc}
     - {when: {wm: xp}, folder: xp}
     - {folder: neutral}
   ```

2. **Which panels**: `when` on a panel, for state, agents, chat or app inside one folder; for
   example the agents panel only while an agent works, a compact line while a chat view
   is shown (`chat: open`).

The HUD shows on `output`; it moves when the focus moves to another output. The panel anchors
are inside the work area of `output.changed`, so they never cover a panel or bar of a window
manager. Panels take no input (`input: pass`) except those with an `action`.

## 4. Drafts for mc, xp and construct

Each is one HUD folder, prototyped in a day, beside the plugin's own HUD until the user test
decides (open question 3). mc and construct get a setting `voice_hud: own|host` (default `own`);
`host` stops their voice ring or status and their question widget, and they stop claiming the
indicator.

* **xp** (`flavors/xp/assets/assistant`, becomes a HUD folder): the paperclip at the bottom
  right above the taskbar, an XP balloon with the sentence (as today), and an XP tray balloon
  "2 helpers working" while agents work; a click on a row, or on the paperclip's
  balloon, opens the balloon into the chat (section 1). The paperclip stays
  drawn in code until X7, so the folder names `figure: paperclip`, as `values.yaml` does now.
* **mc** (`plugins/chora-mc/hud`): mc-style, no box, Inter at step -1, `muted` text, the accent
  on the state word only (mc-style rules 6, 7, 11). Top right of the work area: one line
  "LISTENING" or "SPEAKING" plus the sentence, then one hairline row per agent `Atlas · working
  · Datei schreiben (2/2)`; a click, here or on a column of the agents view, opens the chat in the drawer
  that mc reserves, drawn by the host in this folder's `chat:` look (section 1).
* **construct** (`plugins/chora-construct/hud`): one glass card top centre after
  `construct-style.md`: the state ring (an SVG with one element per state), one line of speech,
  chips of the agents with their state colour. In navigate mode the same card; the question
  glass is the host's, in construct colours. A chip opens the chat as a glass panel at the right (section 1).
* **neutral** (`plugins/chora-assistant/huds/neutral`): today's dot, bubble and buttons, pixel
  for pixel, plus the agents rows and the "Chat ..." row; its chat is the right side panel. Used in tiling
  and as fallback.

## 5. Steps

One step is one merge with its tests, at most about one day of an agent. Lanes are worktrees
whose files do not overlap; a lane runs its steps in order.

| Step | Lane | Files | Needs | Fails first |
|---|---|---|---|---|
| H0 gate | T | `tests/gate-hud.sh`, `tests/hud/` fixtures | none | the table of cells: every cell `missing` today; a forced wrong line fails the gate |
| H1 model | K | `crates/chora-kit/src/hud.rs`, `words.rs` (the en/de word table of the UX pass), `ask.rs` (the question card model), `lib.rs` (3 lines), `docs/hud.md` | none | `unknown_key_fails`, `version_2_is_refused`, `a_symlink_out_of_the_folder_is_refused`, `limits_hold` (64 KiB yaml, 32 panels, 1 MiB per file, 8 MiB per folder), `a_value_is_never_expanded_again`, `when_matches_lists_and_all_keys`, `every_word_has_en_and_de`, `the_key_line_names_the_keys_of_the_event`, `a_button_counts_only_400_ms_after_it_moved` |
| H2 pictures | D | `crates/chora-draw/src/theme.rs`, `assets.rs` | none | `an_image_href_outside_the_folder_draws_nothing` (`/etc/passwd`, `file://`, `../`, `http://`), `a_huge_svg_is_refused` (over 2048 px logical), `fonts_load_only_from_the_folder` |
| H3 voice chat API | V | `services/voice/src/agents.rs`, `client.rs`, `core.rs`, `docs/voice.md` | rebase on `acp-gate` | `agent_read_answers_after_seq`, `chat_open_drops_on_disconnect_and_on_chat_close`, `chat_methods_refuse_a_confined_caller`, `an_open_chat_mutes_results_not_questions`, `an_edit_request_stays_physical_with_a_chat_open` |
| H4a chat model | K | `crates/chora-kit/src/chat.rs`, `lib.rs` (1 line) | H1 | `control_characters_and_bidi_overrides_are_dropped` (ESC, OSC 52, BEL, C1 0x9b, U+202E), `pull_after_seq_resumes_after_a_gap`, `a_typed_yes_is_held_while_a_question_waits`, `held_text_is_sent_after_the_question_ends`, `entries_wrap_to_whole_lines` |
| H4b chat field | V | `services/voice/chat/` (new crate, binary `chora-chat`: opener and `--input` layer surface), workspace member line, `tests/chat.sh` | H3, H4a, H4c | `a_typed_yes_answers_nothing` (no `confirm*` and no `agent_say` in the log while a question waits), `tests/chat.sh`: with a token the field takes the keyboard, without one it does not; Enter sends to `fake-acp.sh`; Esc closes the chat and the window gets the keyboard back; a paste and a `text-input-v3` commit arrive; the field re-anchors on a new place and keeps its draft |
| H4c field keyboard | R | `crates/chora/src/layers.rs`, `state.rs` (the activation rule only) | none | `an_on_demand_layer_with_a_valid_token_gets_the_keyboard`, `a_layer_without_a_token_gets_no_keyboard_on_map`, `a_named_client_gets_no_keyboard`, `a_click_on_a_window_takes_the_keyboard_back` |
| H5 host, one folder | A | `plugins/chora-assistant/**` | H1, H2 | `the_neutral_folder_loads`, the gate cell "neutral equals today" (`png-same.py` against a capture of `92e7539`), `a_folder_cannot_hide_the_question`, `a_folder_without_a_voice_panel_gets_the_dot` |
| H6 host, context | A | `plugins/chora-assistant/src/context.rs`, `config.rs`, `assistant.yaml` | H5 | `focus_on_an_xp_window_picks_xp`, `a_wm_in_a_construct_pane_keeps_space_construct`, `a_burst_of_focus_changes_switches_once` (150 ms), gate cells "tiling to xp to mc to construct switches the folder" |
| H7 host, chat session and view | A | `plugins/chora-assistant/src/chat.rs` (new), `main.rs`, the `chat:` block in `chora-kit/src/hud.rs` is H1's; `huds/neutral/hud.yaml` (chat values); `plugins/chora-tiling/tiling.yaml` (1 line `MOD+a: {run: chora-chat}`); `chora-chat.desktop` ("Chat with a helper"); `tests/gate-chat.sh` | H4b, H6 | `a_click_on_the_chat_row_opens_the_chat_with_a_token`, `the_chat_follows_a_switch_of_look_and_keeps_the_draft`, `the_context_stays_while_the_field_has_the_keyboard`, `a_question_stays_where_it_appeared`, `agent_text_never_draws_a_card`; `gate-chat.sh`: one event script, the same visible lines in all four folders at 1280x720 |
| H8 xp draft | X | `flavors/xp/assets/assistant/`, `flavors/xp/*.service` | H6 | gate cell "xp shows tray rows while an agent works"; folder loads; the chat values of the balloon (Tahoma 14, XP edit box border) load |
| H9 mc draft and chat place | M | `plugins/chora-mc/hud/` (with `chat: {slot: mc}`), `plugins/chora-mc/src/config.rs`, `main.rs`, `agent.rs` (a click on a column calls `assistant chat.show`) | H6 | `voice_hud_host_draws_no_voice_status`, `voice_hud_host_claims_no_indicator`, `an_open_chat_opens_the_drawer_and_emits_its_box`, `a_click_on_an_agent_column_shows_its_chat`; gate cells "mc with host HUD", "mc chat in the drawer" |
| H10 construct draft | C | `plugins/chora-construct/hud/` (with the `chat:` glass look), `plugins/chora-construct/src/config.rs`, `main.rs` | H6 | the same two unit tests for construct, the contrast test of rule 5 also for the `chat:` values, gate cells "construct with host HUD", "construct chat panel", `tests/2d-guard.sh` stays green |
| H11 handover with HUD and chat | T | `tests/gate-hud-handover.sh` (new; it sources the handover helpers and does not edit `gate-handover.sh`) | H7, H9, H10 | all 12 pairs move a window by `window.give` and drag while a chat view is open and while the field has the keyboard: windows move as without the chat; the chat view follows into the look of the new focus with the same transcript and draft; a drop over the field lands in the window manager under it; no press lands on a HUD or chat node |
| H12 persona and docs | P | `tests/persona/session.sh` (`--voice`: chora-voice with `w16-fakes.py`, `fake-acp.sh`, the host), `README.md`, `docs/decisions.md` D79, CHANGELOG | H7 to H10 | `session.sh --voice` smoke: `voice.state` reaches the host, a fake agent shows in a row, `chat.show` opens the view of the flavor |

Waves: H0, H1, H2, H3, H4c together (5 worktrees); then H4a and H5; then H4b and H6; then H7,
H8, H9, H10 together; then H11 and H12, then the user tests. `docs/hud.md` belongs to lane K;
others add to it only in H12. Lane R (core) has only H4c. mc and construct write no chat code:
H9 touches `agent.rs` for the click and `main.rs` for the setting and the drawer; H10 only
`main.rs` for the setting. H7 adds one line to `tiling.yaml`. The window-handover steps also
touch tiling, xp, mc and construct: H7, H9 and H10 rebase after each of them and take no part
of their files.

Draft decision D79 (D78 is the grant draft): "A HUD is a folder with `hud.yaml` (contract 1),
pictures and fonts that `chora-assistant` draws: closed sources, plain `{field}` templates,
`when` on context, anchors in the work area, actions from a closed list. The host draws the
question of the voice agent and the mic state itself. A folder runs no code; computed data come
from a plugin by `events.emit`. An ACP chat is one core (voice transcript methods,
`chora-kit::chat`), one view drawn by the host in the `chat:` look of each folder and one
layer-surface text field; it follows the user between looks and answers no question."

## 6. User tests

Personas run through `tests/persona` (capture and inject only) in `session.sh FLAVOR --voice`.
The PM drives the fakes between tasks (`fake-acp.sh` with `FAKE_ACP_DELAY=3`), never the
persona. All input of a persona is injected, so by design it cannot answer a question (the host
and voice ignore injected presses and keys): its tasks check that it finds and reads a question,
and the answer path is tested otherwise.

| # | Persona | Flavor | Goal in the brief | Expected answer |
|---|---|---|---|---|
| L1 | low-tech | xp | "A helper on the screen wants to do something. Find out what, and how you would allow it." | names `notes.txt` and the key of the key line or the Yes button; after an injected click on Yes the card says "Please use your own mouse or keyboard." |
| L2 | low-tech | tiling | "Write 'please write the notes' to the helper Atlas and tell me what it answers." | with no helper running: clicks "Chat ...", picks Atlas, the side panel opens; types, reports "Erlaubt: die Datei ist fertig" or "Abgelehnt ..." |
| L5 | low-tech | xp | "Ask the paperclip helper what it is doing, in writing." | clicks the paperclip's row or balloon, the balloon becomes the chat; reads the answer in the balloon |
| L6 | low-tech | mc | "Write to the helper from the agents page." | clicks Atlas's column, the drawer opens with the chat, types, reads the answer |
| L3 | low-tech | mc | "What is the helper doing right now?" | "Datei schreiben", step 2 of 2 |
| L4 | low-tech | construct | "Is the computer listening to you? Make it stop listening." | reads the state, clicks the figure, the muted badge shows |
| U1 | r/unixporn | tiling | "Give the assistant your own look: start from the folder the system ships and make the bubble dark with orange text." | in `foot`: copies `/usr/share/chora/huds/neutral`, edits `hud.yaml`, `choractl call assistant settings.set`; capture shows the colours |
| U2 | r/unixporn | tiling | "Install the HUD in `~/Downloads/fancy-hud` and tell me if anything looks wrong." (a fixture with a symlink to `~/.ssh/id_ed25519`, an `href` to `/etc/passwd` and a 30000 px SVG) | the HUD does not change; `--check-hud` names the three errors; nothing of the files shows |
| U3 | r/unixporn | 4 outputs | "Keep a chat with Atlas open and go through all four desktops; move a window along each time." | the chat shows in each look with the same transcript and the half-typed draft; windows move as without the chat; the look changes within 0.5 s |
| U4 | r/unixporn | mc | "Run three helpers and keep up with them in the chat and the HUD." | three rows; the chat of each shows its own transcript; no stray characters (fake agent emits OSC 52, a title escape and U+202E) |
| U5 | r/unixporn | tiling | "Make the chat panel match your bar: dark, orange names, your font." | edits the `chat:` block of a copied folder, `--use-hud`; capture shows it; words and buttons unchanged |

Tested otherwise, because a capture cannot show it: a spoken or typed yes to a physical request,
an always word, an injected key or press (voice and host unit tests, gate cells); a button press
that is not injected (`answer(node, injected: false)` unit tests); confined callers (H3 tests);
the clipboard after an OSC 52 (H4a test), the keyboard rule of the field (H4c); file reads outside a folder (H1, H2 tests); timings and
load (gate-hud: 100 `voice.state` in 5 s, `layers.dropped` stays empty, host CPU below 1 % idle,
chat Enter to first agent text below 300 ms with the fake).

Exit criteria of the sprint:

* H0 to H12 merged; `gate-hud`, `gate-hud-handover`, `gate-handover`, `2d-guard`, the voice
  tests and clippy green.
* L1 to L6 reached in two runs each, under 30 actions, no STUCK; U1 to U5 reached in one run
  each, under 60 actions; each SURPRISE is fixed or filed.
* No open security finding on H3, H4a to H4c or H5; a review of the host by the split-review charter.
* The PM decides `voice_hud` defaults for mc and construct from L3, L4 and U3.

## 7. Risks

* **2D.** HUD nodes live under `screen:*` with 2D transforms only; the construct card too. The
  2D guard runs in H10 and in the gate. A plane transform in a folder is not expressible.
* **Security.** A shipped folder is untrusted input:
  * no code: closed sources, closed actions, plain templates; text of events is never markup;
  * files: real path inside the folder on the open handle (as `read` of voice does), no
    symlink out, size limits, SVG `href` and fonts resolved only inside the folder, no network;
  * spoofing: the host owns the question and the mic badge and draws them on top; a folder
    panel cannot cover the question, and panel size is capped at a third of the output, so a
    folder cannot paint a fake lock screen;
  * actions: `mute` only turns the mic off; `chat` only calls the host's `chat.show` with an
    agent name that matches `good_name`; nothing calls `confirm*`, `talk`, `indicator.claim` or settings;
  * the host keeps the trust of `indicator.claim` (keyboard-level) and stays on the main socket;
    `huds` in its settings is a path list and `settings.set` is privileged (D62);
  * ACP: the chat adds no route around `acp-gate`; transcript text is untrusted and never draws
    a card; full transcripts never reach a confined caller;
  * focus: the field takes the keyboard only with an activation token of a person's act (H4c).
* **Performance.** Voice emits at most 32 events/s; the host coalesces per frame, uses text
  nodes for changing text and caches pictures per state and scale. A heavy SVG filter is a CPU
  cost at load only, and capped by the size limit. The watchdog (D57) still guards the host.
* **Windows between window managers.** The chat is no window: it follows the user as a view of
  the host and a layer-surface field without a claim; H11 checks all 12 pairs with it open. HUD nodes take no input except action panels, so no drag or drop lands on them, and
  they have no claim, so `claimant_at` never picks them. A context switch must not steal the
  focus: the host never sends `focus` or `focus_node`. Risk left: a WM inside a construct pane;
  `space` covers it, H6 tests it.
* **Two drawers.** While mc or construct draw their own voice HUD and the host draws too, a
  question shows twice. `voice_hud: own` makes the host skip the question and the voice panels
  in that context; H9 and H10 test both settings.
* **Contract lock-in.** `hud: 1` is checked; a change of meaning is `hud: 2`. Until 1.0 the
  contract may change with a note in CHANGELOG.

## 8. Open questions for the user

1. Chat as a terminal window or inside each look? Decided by the user: inside each look, one
   host-drawn view styled by the folder, one layer-surface field (section 1).
2. Grow `chora-assistant` into the host, or a new block `chora-hud`? Default: grow it; the name
   and the XP unit stay, `--assets` names the fallback folder.
3. Should mc and construct hand their voice HUD to the host? Default: `voice_hud: own` this
   sprint; the PM switches after the user tests.
4. May a typed "yes" in the chat answer a `read`, `search` or `think` question? Default: no, the
   chat answers nothing.
5. Does an agent with an open chat speak its results? Default: no, only its questions.
6. Type to the voice LLM itself in the chat, not only to ACP agents? Default: not this sprint;
   the picker lists helpers only.
7. May a HUD folder call non-privileged plugin methods, such as `tiling.get_tags`? Default: no,
   events only; a plugin emits what a HUD needs.
8. Does a dropped folder activate itself? Default: no, one line in `chora-assistant.yaml`.
