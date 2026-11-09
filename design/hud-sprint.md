# Feature sprint 1: ACP chat and HUDs

Design of the first feature sprint. Read on 2026-11-09 at `7dccda3` and on `claude/acp-gate`
(`72c94d6`). References are `path:line` or `path` in the chora repository. Values: POLS, KISS,
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

**What the user does.** Opens a chat with an ACP agent, types prompts, reads the full
transcript (messages, plan, tool calls, questions), cancels a prompt, starts and stops an agent.
Voice may run beside it (the same agent answers spoken and typed prompts, one queue) or be
silent (mic and speaker muted, ASR, TTS and LLM servers down: agents still work, they only need
`sandbox_cmd` and `agent_cmd`).

**Where it lives: a window.** `chora-chat NAME`, a terminal program in `foot` with app id
`chora-chat`. Reasons: a window moves between all window managers through the existing
handover (hard requirement); `foot` brings IME, paste, scrollback, fonts and accessibility; mc
docks it by app id, the construct shows it in a pane (roadmap K3 prompt panes). A HUD panel
with `focus_node` text input was rejected: no IME, no paste, no move between window managers,
and D56 gives text input to clients.

* Crate `services/voice/chat` (binary `chora-chat`), `crossterm` for raw mode and one input
  line; no TUI framework. It connects to the main socket as `chat-NAME`, unconfined, like
  `choractl`. Without NAME it lists agents and projects (`agent_list`) and asks for one.
* Input: a line is a prompt (`voice.agent {tool: agent_say}`); `/start PROJECT [NAME]`,
  `/stop`, `/cancel`, Ctrl+C cancels the running prompt. Output: the transcript, the plan, and
  an open question with its key line ("Alt+y = yes, Alt+n = no, or the buttons of the HUD").
* New in voice (small, in `agents.rs` and `client.rs`):
  * each agent keeps a ring of 500 transcript entries `{seq, role: user|agent|tool|question|plan,
    text}`; `voice.state` of an agent carries `seq`, so a reader knows when to pull;
  * `agent.read {name, after}` answers the entries after `seq` (at most 64 KiB);
  * `agent.cancel {name}` (today only the spoken cancel word reaches `Agents::cancel`);
  * `chat.open {name}` marks a chat as open for the life of the caller's connection, dropped on
    `client.disconnected`, as `indicator.claim` is; the agent record gets `chat: true`;
  * an agent with an open chat does not speak its results, only its questions (open question 5).
  All four are privileged and refused for a confined caller (`not_for_confined`).

**Reuse of the gate.** The chat sends text and reads text. Every ACP permission request still
goes the one way of `acp-gate`: the core asks, the key or a HUD button answers. The chat answers
no question: a terminal cannot tell typed text from pasted or injected text, so a typed "y" is
no proof (same argument as for the spoken yes). `agent_start` through the chat runs as
`voice.agent` does today; the chat is a trusted program of the user, like `choractl`.

**What it may not do.**

* Raise or change grants, answer `confirm.answer`, claim the indicator, call `talk` or `mute`.
* Pass a chora socket, MCP server or file system to the agent: `session/new` keeps
  `mcpServers: []` and `initialize` keeps no fs and no terminal.
* Print control characters of agent text. ACP text is untrusted: the chat drops C0 except `\n`
  and `\t`, all C1, ESC and DEL before it writes to the terminal (OSC 52 clipboard writes, title
  changes, cursor tricks). No links, no "run this" action.
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
    when: {agents: any}
    anchor: top_right
    rows: "{name}  {state}  {task}"
    max_rows: 4
    action: chat             # a click on a row opens chora-chat for that agent
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
new engine, no scripting, no core change.

## 3. Context-dependent voice HUD

The host keeps one context record and re-evaluates on each change, debounced 150 ms, with the
cross-fade that the assistant has.

| Field | From | Values |
|---|---|---|
| `wm` | `focus.changed`, then the owner of the focused surface (`scene.query`); no focus: the claimant of the output under the pointer | client name: `tiling`, `xp`, `mc`, `construct`, ... |
| `space` | the claimant of the real output that shows the focused window; differs from `wm` when a window manager runs in a construct pane | client name |
| `app` | `app_id` of the focused surface | `foot`, `chora-chat`, ... |
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
   example the agents panel only while an agent works, a compact line while `chora-chat` has
   the focus.

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
  "2 helpers working" while agents work; a click on a row opens the chat. The paperclip stays
  drawn in code until X7, so the folder names `figure: paperclip`, as `values.yaml` does now.
* **mc** (`plugins/chora-mc/hud`): mc-style, no box, Inter at step -1, `muted` text, the accent
  on the state word only (mc-style rules 6, 7, 11). Top right of the work area: one line
  "LISTENING" or "SPEAKING" plus the sentence, then one hairline row per agent `Atlas · working
  · Datei schreiben (2/2)`; a click opens the chat, which mc docks in the drawer by its app id.
* **construct** (`plugins/chora-construct/hud`): one glass card top centre after
  `construct-style.md`: the state ring (an SVG with one element per state), one line of speech,
  chips of the agents with their state colour. In navigate mode the same card; the question
  glass is the host's, in construct colours.
* **neutral** (`plugins/chora-assistant/huds/neutral`): today's dot, bubble and buttons, pixel
  for pixel, plus the agents rows. Used in tiling and as fallback.

## 5. Steps

One step is one merge with its tests, at most about one day of an agent. Lanes are worktrees
whose files do not overlap; a lane runs its steps in order.

| Step | Lane | Files | Needs | Fails first |
|---|---|---|---|---|
| H0 gate | T | `tests/gate-hud.sh`, `tests/hud/` fixtures | none | the table of cells: every cell `missing` today; a forced wrong line fails the gate |
| H1 model | K | `crates/chora-kit/src/hud.rs`, `lib.rs` (1 line), `docs/hud.md` | none | `unknown_key_fails`, `version_2_is_refused`, `a_symlink_out_of_the_folder_is_refused`, `limits_hold` (64 KiB yaml, 32 panels, 1 MiB per file, 8 MiB per folder), `a_value_is_never_expanded_again`, `when_matches_lists_and_all_keys` |
| H2 pictures | D | `crates/chora-draw/src/theme.rs`, `assets.rs` | none | `an_image_href_outside_the_folder_draws_nothing` (`/etc/passwd`, `file://`, `../`, `http://`), `a_huge_svg_is_refused` (over 2048 px logical), `fonts_load_only_from_the_folder` |
| H3 voice chat API | V | `services/voice/src/agents.rs`, `client.rs`, `core.rs`, `docs/voice.md` | rebase on `acp-gate` | `agent_read_answers_after_seq`, `chat_open_drops_on_disconnect`, `chat_methods_refuse_a_confined_caller`, `an_open_chat_mutes_results_not_questions`, `an_edit_request_stays_physical_with_a_chat_open` |
| H4 chora-chat | V | `services/voice/chat/` (new crate), workspace member line, `tests/chat.sh` | H3 | `control_characters_are_dropped` (ESC, OSC 52, BEL, C1 0x9b), `a_typed_yes_answers_nothing` (no `confirm*` call in the log), `tests/chat.sh`: prompt to `fake-acp.sh` shows "Erlaubt" after a `voice.confirm` yes |
| H5 host, one folder | A | `plugins/chora-assistant/**` | H1, H2 | `the_neutral_folder_loads`, the gate cell "neutral equals today" (`png-same.py` against a capture of `7dccda3`), `a_folder_cannot_hide_the_question`, `a_folder_without_a_voice_panel_gets_the_dot` |
| H6 host, context | A | `plugins/chora-assistant/src/context.rs`, `config.rs`, `assistant.yaml` | H5 | `focus_on_an_xp_window_picks_xp`, `a_wm_in_a_construct_pane_keeps_space_construct`, `a_burst_of_focus_changes_switches_once` (150 ms), gate cells "tiling to xp to mc to construct switches the folder" |
| H7 host, chat action | A | `plugins/chora-assistant/src/main.rs` | H4, H6 | `a_click_on_an_agent_row_runs_chora_chat_with_a_token_for_the_context_wm` (D72), gate cell "chat opens in the window manager of the context" |
| H8 xp draft | X | `flavors/xp/assets/assistant/`, `flavors/xp/*.service` | H6 | gate cell "xp shows tray rows while an agent works"; folder loads |
| H9 mc draft | M | `plugins/chora-mc/hud/`, `plugins/chora-mc/src/config.rs`, `main.rs` | H6 | `voice_hud_host_draws_no_voice_status`, `voice_hud_host_claims_no_indicator`, gate cell "mc with host HUD" |
| H10 construct draft | C | `plugins/chora-construct/hud/`, `plugins/chora-construct/src/config.rs`, `main.rs` | H6 | the same two unit tests for construct, gate cell "construct with host HUD", `tests/2d-guard.sh` stays green |
| H11 handover with HUD and chat | T | `tests/gate-hud-handover.sh` (new; it sources the handover helpers and does not edit `gate-handover.sh`) | H4, H6 | all 12 pairs move a `chora-chat` window by `window.give` and drag with the host running; the transcript, the focus and the HUD context follow; no press lands on a HUD node |
| H12 persona and docs | P | `tests/persona/session.sh` (`--voice`: chora-voice with `w16-fakes.py`, `fake-acp.sh`, the host), `README.md`, `docs/decisions.md` D79, CHANGELOG | H7 to H10 | `session.sh --voice` smoke: `voice.state` reaches the host, a fake agent shows in a row |

Waves: H0, H1, H2, H3 together (4 worktrees); then H4 and H5; then H6 and H11; then H7, H8, H9,
H10 together; then H12 and the user tests. `docs/hud.md` belongs to lane K; others add to it
only in H12. H9 and H10 touch their plugin's `main.rs` only for the setting; nothing else of the
sprint touches those files. The window-handover steps also touch tiling, xp, mc and construct:
H9 and H10 rebase after each of them and take no part of their files.

Draft decision D79 (D78 is the grant draft): "A HUD is a folder with `hud.yaml` (contract 1),
pictures and fonts that `chora-assistant` draws: closed sources, plain `{field}` templates,
`when` on context, anchors in the work area, actions from a closed list. The host draws the
question of the voice agent and the mic state itself. A folder runs no code; computed data come
from a plugin by `events.emit`. An ACP chat is the window `chora-chat`, which answers no
question."

## 6. User tests

Personas run through `tests/persona` (capture and inject only) in `session.sh FLAVOR --voice`.
The PM drives the fakes between tasks (`fake-acp.sh` with `FAKE_ACP_DELAY=3`), never the
persona. All input of a persona is injected, so by design it cannot answer a question (the host
and voice ignore injected presses and keys): its tasks check that it finds and reads a question,
and the answer path is tested otherwise.

| # | Persona | Flavor | Goal in the brief | Expected answer |
|---|---|---|---|---|
| L1 | low-tech | xp | "A helper on the screen wants to do something. Find out what, and how you would allow it." | names `notes.txt` and Alt+y or the Yes button |
| L2 | low-tech | tiling | "Write 'please write the notes' to the helper Atlas and tell me what it answers." | finds a row, opens the chat, types, reports "Erlaubt: die Datei ist fertig" or "Abgelehnt ..." |
| L3 | low-tech | mc | "What is the helper doing right now?" | "Datei schreiben", step 2 of 2 |
| L4 | low-tech | construct | "Is the computer listening to you? Make it stop listening." | reads the state, clicks the figure, the muted badge shows |
| U1 | r/unixporn | tiling | "Give the assistant your own look: start from the folder the system ships and make the bubble dark with orange text." | in `foot`: copies `/usr/share/chora/huds/neutral`, edits `hud.yaml`, `choractl call assistant settings.set`; capture shows the colours |
| U2 | r/unixporn | tiling | "Install the HUD in `~/Downloads/fancy-hud` and tell me if anything looks wrong." (a fixture with a symlink to `~/.ssh/id_ed25519`, an `href` to `/etc/passwd` and a 30000 px SVG) | the HUD does not change; `--check-hud` names the three errors; nothing of the files shows |
| U3 | r/unixporn | 4 outputs | "Keep a chat with Atlas open and move it through all four desktops; the overlay must follow." | chat moves 4 times with its transcript; the HUD look changes per window manager within 0.5 s |
| U4 | r/unixporn | mc | "Run three helpers and keep up with them in the chat and the HUD." | three rows; the chat of each shows its own transcript; no garbage in the terminal (fake agent emits OSC 52 and a title escape) |

Tested otherwise, because a capture cannot show it: a spoken or typed yes to a physical request,
an always word, an injected key or press (voice and host unit tests, gate cells); a button press
that is not injected (`answer(node, injected: false)` unit tests); confined callers (H3 tests);
the clipboard after an OSC 52 (H4 test); file reads outside a folder (H1, H2 tests); timings and
load (gate-hud: 100 `voice.state` in 5 s, `layers.dropped` stays empty, host CPU below 1 % idle,
chat Enter to first agent text below 300 ms with the fake).

Exit criteria of the sprint:

* H0 to H12 merged; `gate-hud`, `gate-hud-handover`, `gate-handover`, `2d-guard`, the voice
  tests and clippy green.
* L1 to L4 reached in two runs each, under 30 actions, no STUCK; U1 to U4 reached in one run
  each, under 60 actions; each SURPRISE is fixed or filed.
* No open security finding on H3, H4 or H5; a review of the host by the split-review charter.
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
  * actions: `mute` only turns the mic off; `chat` runs only `chora-chat` with an agent name
    that matches `good_name`; nothing calls `confirm*`, `talk`, `indicator.claim` or settings;
  * the host keeps the trust of `indicator.claim` (keyboard-level) and stays on the main socket;
    `huds` in its settings is a path list and `settings.set` is privileged (D62);
  * ACP: the chat adds no route around `acp-gate`; transcript text is untrusted; full
    transcripts never reach a confined caller.
* **Performance.** Voice emits at most 32 events/s; the host coalesces per frame, uses text
  nodes for changing text and caches pictures per state and scale. A heavy SVG filter is a CPU
  cost at load only, and capped by the size limit. The watchdog (D57) still guards the host.
* **Windows between window managers.** The chat is an ordinary window, covered by H11 for all
  12 pairs. HUD nodes take no input except action panels, so no drag or drop lands on them, and
  they have no claim, so `claimant_at` never picks them. A context switch must not steal the
  focus: the host never sends `focus` or `focus_node`. Risk left: a WM inside a construct pane;
  `space` covers it, H6 tests it.
* **Two drawers.** While mc or construct draw their own voice HUD and the host draws too, a
  question shows twice. `voice_hud: own` makes the host skip the question and the voice panels
  in that context; H9 and H10 test both settings.
* **Contract lock-in.** `hud: 1` is checked; a change of meaning is `hud: 2`. Until 1.0 the
  contract may change with a note in CHANGELOG.

## 8. Open questions for the user

1. Chat in a terminal (`foot` with `chora-chat`) or a toolkit window? Default: terminal.
2. Grow `chora-assistant` into the host, or a new block `chora-hud`? Default: grow it; the name
   and the XP unit stay, `--assets` names the fallback folder.
3. Should mc and construct hand their voice HUD to the host? Default: `voice_hud: own` this
   sprint; the PM switches after the user tests.
4. May a typed "yes" in the chat answer a `read`, `search` or `think` question? Default: no, the
   chat answers nothing.
5. Does an agent with an open chat speak its results? Default: no, only its questions.
6. Type to the voice LLM itself in the chat, not only to ACP agents? Default: not this sprint.
7. May a HUD folder call non-privileged plugin methods, such as `tiling.get_tags`? Default: no,
   events only; a plugin emits what a HUD needs.
8. Does a dropped folder activate itself? Default: no, one line in `chora-assistant.yaml`.
