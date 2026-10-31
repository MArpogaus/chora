# chora-voice: the voice agent in Rust (draft specification)

Status: draft, 2026-10-30. WP0 to WP8 are done. Sources: the Python
agent of before WP6 (`services/voice/agent.py` in git history), D53,
D58, D61, D62, `docs/api.md`, the `chora-kit` runtime and settings,
`choractl mcp`, and live probes of the Gufo servers.

`chora-voice` is the voice agent of chora. It completed full voice turns
against the live servers in WP6; the Python agent and
`voice-agent.service` are gone, and `chora-voice.service` runs it
(`services/voice/README.md`).

## 1. Goals and non-goals

Goals:

- One binary `chora-voice` in the crate `services/voice`: a chora client
  on the `chora-kit` runtime, with D62 settings.
- Silero VAD on `tract` (pure Rust, no C library) as the speech gate.
- Always listening, with the local words "Schlaf" and "Hör zu", plus the
  push-to-talk (PTT) key. The wake word goes through the ASR.
- A cancel word stops the current turn at any time, also while the
  agent speaks or works.
- Spoken yes/no confirmation before the tools of a configurable list.
- Separate mute switches for the mic and the speaker.
- No silent failure; a latency per stage for each turn; a state for the
  HUD.
- Replay tests that measure speech detection and barge-in against
  labelled recordings.

Non-goals: new ASR, TTS or LLM servers; a keyword-spotting model; an own
echo canceller (the PipeWire one stays); a GUI; a package before the
agent completes full turns.

## 2. Architecture

One process. The `core` is one thread that reads one input channel. All
other actors send their inputs into that channel, and the core sends
effects back to them. The core holds the state machine as a pure
function `step(state, input) -> (state, effects)`, so unit tests need no
IO. WP4 adds a tokio runtime for the network actors; they send into the
same channel.

| Actor | Job |
|---|---|
| `audio_in` | A thread: the capture command at 24 kHz mono, 32 ms frames (768 samples), Silero probability and RMS per frame |
| `audio_out` | The playback command at 24 kHz for TTS and earcons; knows when the written sound ends; stops at once |
| `asr` | One websocket per utterance to Qwen3-ASR realtime, PCM in 1 s pieces |
| `tts` | One websocket per sentence to Qwen3-TTS, one sentence ahead |
| `llm` | `/chat/completions` with SSE and tools |
| `tools` | MCP client of `choractl --name voice mcp`, the coder tools, `read`, the confirmation gate |
| `coder` | One background coding task, as in Python |
| `chora` | A thread with the `chora-kit` runtime: methods, settings, PTT grab |
| `hud` | A thread: times the events of the HUD for the `chora` thread; the indicator node on its own connection `NAME-hud` (WP8) |
| `core` | State machine, speech gate, history, word matcher, turn metrics |

Audio: the ASR takes 24 kHz only (WP0), and the TTS sends 24 kHz. So the
mic records at 24 kHz, and the ASR gets that PCM unchanged. Only the VAD
gets a 16 kHz copy: a 33-tap windowed-sinc low-pass at 7 kHz, then two
samples out of each three. Subprocesses replace the `pipewire` crate,
which needs bindgen and C headers. The commands are settings, so
`arecord`/`aplay` are a fallback without code. Tests feed WAV files.

Tool layer: LLM tools go through MCP (`rmcp` client). The tool schemas
and the plugin tools stay in one place, and `chora_mcp` can still run
tools in a container. The agent's own needs (PTT grab, methods,
settings, indicator) go through the runtime connection. This replaces
the PTT polling through `wait_events`.

The `chora` thread runs `runtime::run` and `runtime::serve` of
`chora-kit`; `runtime::dispatch` is public already. Methods send an
input with a reply channel to the core and wait up to 1 s for the new
state. When the runtime gives up after 30 s without chora, the agent
exits with 1.

## 3. State machine

| State | Meaning |
|---|---|
| `sleeping` | Hears only wake words; PTT records a request |
| `listening` | Waits for speech |
| `recording` | Collects an utterance |
| `transcribing` | Waits for the ASR |
| `thinking` | Waits for the LLM, no sound |
| `tool` | A tool call runs |
| `confirming` | Waits for yes or no |
| `speaking` | Plays the reply |
| `working` | Listening while a coding task runs |

The agent starts in `listening` (awake). The core derives the state from
what goes on, in this order: an utterance in progress is `recording`;
an utterance whose text waits is `transcribing` (not while asleep); a
reply is `confirming` while a question waits, `speaking` while its sound
plays or ran dry less than 1.5 s ago (0.25 s after a question that waits
for its answer, so that the speech gate hears a short "Ja"), else
`thinking` or `tool`; else
`listening` or `sleeping`. So speech during a reply shows as
`recording` and `transcribing`, and the state of the reply comes back
after it.

| From | Input | To |
|---|---|---|
| `sleeping` | Short utterance with a wake word | `listening`, earcon |
| `sleeping` | PTT down | `recording`, then back to `sleeping` |
| `listening`, `working` | VAD onset or PTT down | `recording` |
| `recording` | `silence_ms` quiet, PTT up, or `max_turn_ms` | `transcribing` |
| `recording` | Less than `min_speech_ms` speech | previous state |
| `transcribing` | Sleep word | `sleeping`, earcon |
| `transcribing` | Cancel word, empty text or echo | previous state |
| `transcribing` | Code cancel word | previous state; the coding task stops |
| `transcribing` | Other text | `thinking` |
| `thinking` | First sentence | `speaking`; tools run in parallel |
| `thinking` | Tool call on the confirm list | `confirming` |
| `thinking` | Other tool call | `tool` |
| `tool` | Result or timeout | `thinking` |
| `confirming` | Yes | `tool` |
| `confirming` | No, two unclear answers, a new request, or timeout | `thinking`; the result says which |
| `speaking` | Audio and LLM done | `listening` or `working` |
| busy states | Cancel word, `voice.cancel`, PTT down | `listening`, earcon |
| any | Sleep word or `voice.sleep` | `sleeping`, earcon |
| `sleeping` | `voice.wake` | `listening`, earcon |

An ASR error sounds the error earcon. The final text of an older
utterance still counts; a late interim text drops. An utterance while
asleep only looks for a wake word; one by the PTT key is a request, and
the agent sleeps again after the reply.

Barge-in while `speaking`: speech over the barge-in gate (section 4) for
`barge_in_ms` pauses playback at once. The unplayed sentences stay
queued. A cancel word drops them and cancels the turn. Empty text or
echo plays them from the cut sentence; a spoken yes or no to a question
skips the cut question. Other text ends the reply; the history keeps
only the played text, and the new text starts a reply. The utterance of
a barge-in starts with the barge-in speech and `preroll_ms` before it,
and ends as any other after `silence_ms` of quiet, so a short pause
makes no second turn. Text that is not echo ends the reply the same way
while its sound plays, also when it came without a barge-in.

Each heard text that is not a command word shows first in the HUD, as
`phrases.heard` ("Heard: ...") through the `say` event.

Speech while `thinking`, `tool` or `confirming`: each utterance goes to
the ASR at once. A cancel word cancels. In `confirming`, the text is the
answer, but only speech that starts after the question starts to
play. Other text goes into the next LLM round of the running reply,
as a user message (`Inject`; the chat reports it back as `Heard`). This
also holds after the answer of a round: when new words came while the
LLM answered, one more round hears them. After the last round of
`tool_rounds`, new words get `tool_rounds` more rounds in the same turn;
they do not wait until the reply is said. Short text goes in too
("Und größer.", "Nein, rot."). Only a pure yes or no waits in the queue.
It never answers a question. A call that would ask does not ask while new words wait:
it does not run, and its result is "Not confirmed: the user said
something new; nothing happened". Texts that no round heard start the
next reply together, as one message (`queued` in the state record). No
speech is dropped.

A turn whose first round calls tools without words says `phrases.ack`
("One moment.") once. An answer whose last sentence announces an action
in the first person, but calls no tool, gets one more round per turn
with a note to call it or to say that it will not. A turn that already
ran a tool gets no such round. Only explicit forms announce: "let me
...", or "I'll", "I will" or "I'm going to" before a verb of action.
In German, these are "ich werde ...", "lass mich ..." and "ich schaue
nach", "mal" or "kurz". These are no announcement: a question, "let me
know", "I'll be here", a negation ("I will not", "nicht", "kein"), and
other German "ich" sentences ("Ich helfe dir gern.", "Ich starte den
Loader.").

An answer that ends with `stop`, calls no tool, and whose words do not
end with `.`, `!`, `?` or `…` (also before a closing quote) is cut: Gufo
0.27 ended rounds after 1-12 tokens ("I’m setting the", "I’ll send Alt",
2026-10-31). Its last words after the last spoken sentence never play
and never go into the history. One more round runs with the note
"[Your last answer stopped mid-sentence. Answer again in full.]" at the
end of the request; the note does not go into the history, because the
same request gave the same fragment again. That round does not say again
the sentences that already played, and a cut round does not count
toward `tool_rounds`. After two cut rounds in a row, the emergency brake
drops the history before the last words of the user, and one more round
runs. A third cut says `phrases.lost` ("I lost the thread, please say it
again.") and ends the turn. The cut check comes before the announcement
check, and a cut answer never gets the announcement round.

Cancel stops only the current turn: the reply task drops the SSE request
and the MCP call in flight (rmcp sends `notifications/cancelled`), the
history gets the words that played, cut back to the last whole
sentence (else without the last word, which may be cut), and a note:
for a call in flight "cancelled by the user; result unknown" as its tool
result, else `chat::STOPPED` as a message of the user of its own. The
LLM copied a note at the end of its own answer into the next answer, and
it copied answers that end mid-word. The agent never speaks or keeps
these notes: they go from the stream of the LLM and from the stored
answer. Playback stops. A cancel word does not
stop a coding task. A code cancel word ("Coding-Aufgabe abbrechen")
stops it. Coding results wait for `listening` or `working`, and also go
into the history.

Mute: `voice.mute` has two separate switches. A muted mic starts no turn
by VAD, and no barge-in; PTT still records, because the key is an
explicit request. A muted speaker plays no TTS and no earcons; the reply
text still goes to `last_reply` and to the HUD.

## 4. Speech gate, echo, word matcher

Onset: probability over `vad_threshold` for `onset_ms`. End: under
`vad_release` for `silence_ms`. An utterance keeps at most `preroll_ms`
before the first frame of its onset. The PTT key starts an utterance at
once; while the key is down, each frame counts as speech, and only the
release ends the utterance.

Echo (decided in WP6, open question 4): the echo level is the level of
the played sound times a coupling. The played level is the loudest RMS
of the played frames in the last 160 ms (the sound reaches the mic a
little later than it is written). The coupling starts at
`barge_in_coupling` (0.1: the PipeWire echo canceller is assumed) and is
learned during playback: a median tracker of mic RMS over played level
(steps of 5 % per frame; only 1 % up on a frame that counts as speech).
Barge-in needs probability over `barge_in_vad_threshold` and RMS over
`max(floor, echo level × barge_in_echo_ratio)` for `barge_in_ms`. The
floor is `barge_in_min_rms` when it is above 0; else it adapts: 40 % of
the median of the last five turns' median frame RMS, at least 5, and 15
before the first turn. The learned coupling goes down to 0.00001: the
PipeWire echo canceller left an echo of RMS 0.1 under played sound of
RMS 4000, and the user's speech had an RMS of 20-70 (live test
2026-10-31). Silero hears such quiet speech late or not at all, so the
VAD (not the gate or the ASR) gets the mic with a gain that lifts its
recent peak level to 1000: at most 33×, the peak halves in about 30 s. Its utterance keeps that speech and `preroll_ms` before
it: with 100 ms, the ASR heard "verbrechen" for "Abbrechen" (WP6 live).
Each utterance, the pre-roll included, goes to the ASR scaled so that
its peak is at -3 dBFS: at most 40 dB, never down, without clipping.
In the live test of 2026-10-31 the utterances had a peak of 4-63 (RMS
1-8) behind the echo canceller. `pipewire/echo-cancel.conf` turns on the
digital gain control of WebRTC and turns off its noise suppression,
the likely cause of the digital zero between the words (not yet
verified on PipeWire).
A transcript whose words match more than 80 % of the words spoken in the
last 3 s is echo.

Word matcher (no LLM):

1. Only utterances with up to `word_max_ms` of speech and four words
   count. The speech is the VAD frames, without `preroll_ms` and the
   end wait: "Schlaf jetzt." had 1856 ms of speech in an utterance of
   2848 ms and went to the LLM with the old limit on the length
   (live 2026-10-31). A command word acts at once, also while a reply
   thinks, runs tools or speaks.
2. Normalize: lowercase, no punctuation, `ä→ae`, `ö→oe`, `ü→ue`, `ß→ss`.
3. Remove `filler_words`.
4. The rest must equal a phrase. Words of four letters or more may
   differ by one edit; words written together also match
   ("Codingaufgabe").
5. Order: code cancel, cancel, sleep, wake, yes, no. Yes and no count
   only in `confirming`, by the answer rules of section 5 and without
   the limits of rule 1.

While asleep, each short utterance goes to the ASR to find a wake word.
A keyword-spotting model can replace this later.

## 5. Confirmation

A rule matches by `tool` (glob), and optionally by the command kinds of
`apply_commands`, by `argv0` of `run_program`, or by `method` (glob) of
`call_method`. The agent speaks the `prompt` of the rule, with the
program of `run_program` in place of `{program}`, the argument `NAME`
in place of `{NAME}` (a missing one stays as it is) and all text,
number and boolean arguments in place of `{args}` ("Shall I change the
screen to output winit, scale 1.5?"), and waits for
yes or no. The method `confirm` answers from a key or the HUD (WP5;
privileged, so a confined client needs `methods.privileged`),
the spoken yes and no since WP6. Only speech that starts after the
question starts to play is an answer. A queued text never answers.
Without speaker sound, the question counts from when it is asked.
Only a pure yes or no answers: one or two yes phrases, or one or two
no phrases, and filler words. Examples are "Ja, bitte.", "Ja, mach
das." and "Nein, lieber nicht." A text of fillers only, such as
"Bitte.", counts with them. Other text of at most two words is
unclear ("1.5", "Ja, nein"). Two unclear answers are a no. A longer
text is a new request, also with a yes word in it ("Mach das andere
Fenster zu."). A new request is a no, and the text starts the next
turn. No answer within
`confirm_timeout_ms` from the end of the sound of the question is a no
(a question that is never said waits 30 s longer). The LLM gets the
reason as the result:
"The user declined the spoken confirmation", "The user did not answer
the spoken confirmation in time (15 s)" or "The answer to the spoken
confirmation was not understood", each with "; nothing happened".

A yes with an `always_words` phrase ("Ja, immer.", "Always.") allows
the same action until the agent restarts or takes settings: each
`settings.set`, each `settings.reload` and each new connection to
chora ends all of them, so a stricter `confirm` applies at once. Then the agent says
`phrases.allowed` ("Allowed for this session."). The same action is
the same full argv of `run_program`, or the same method `CLIENT.METHOD`
with the same parameters (as canonical JSON, keys sorted). For every
other call, an always word is a single yes: `inject_input`,
`apply_commands`, `code_task`, `agent_start`, `configure_output`, the
other built-in tools, each `settings.set` and `settings.reload`, and
each change of the agent's own settings. A question of a sub-agent
takes an always word as a yes.

Two settings list the actions that never ask. `allow_programs` lists
bare program names, such as `foot`. An entry matches only an argv[0]
that is exactly this name, so `/home/u/proj/foot` asks. `allow_methods`
lists methods `CLIENT.METHOD`, such as `mc.scene.show`, matched
exactly. At load, the agent refuses an empty entry, an entry with a
space, a glob (`*`, `?`, `[`), a program with a `/`, and a method
without a `.`. So `*.settings.set` cannot go in. Neither list applies
to the agent's own settings, which always ask.

Each question also goes out as the event `voice.confirm {prompt,
detail, id}`; `id` is a text, such as `"3"`. `detail` is the full command or value, never cut: the
settings keys and values, the program and its arguments, or the method
and its parameters. A question of a sub-agent has its prompt as
`detail`. When the question gets its answer, ends or times out, the
agent emits `voice.confirm {id, done: true}`. chora-mc shows the
question as a notification, chora-xp as a balloon. The agent cannot
see the subscribers of an event. So, while no plugin claims the
indicator (section 10), the agent shows the prompt and `detail` itself:
a text node `voice-question` at the top left of the first output, on
`NAME-hud`. A text node holds 4 KiB, so a longer `detail` shows only
its start there; the event has all of it. Over the rate limit, the
last `voice.confirm` goes again after 1 s.

The rule of `apply_commands` with `remove` asks only when a target is
not a node of the agent's own client (`NAME-mcp`), or has a node of
another client under it; the agent reads the owners and parents with
`query_scene`, and any doubt asks. `close` always asks.

Own settings: the LLM may call `settings.get`, `settings.set` and
`settings.reload` of the agent's own client NAME, also when
`skipped_tools` hides NAME; no other method of NAME, NAME-hud or
NAME-mcp. The tool of its own `settings.set` takes `{"patch": ...}`
without the schema of the settings, which is about 14 KB and made the
LLM narrate; the agent hands the patch on and the client checks it.
Each `settings.set` asks with its keys and values
(`phrases.settings_set`, "Shall I set barge in min rms to 15?"). A
security key asks with `phrases.settings_security` ("Security setting!
Shall I set llm url? The full value is on the screen."). The security
keys are `allow_programs`, `allow_methods`, `confirm`, `agent_projects`, `yes_words`, `always_words`, `no_words`,
`phrases`, `record_dir`, `sandbox_cmd`, `unsandboxed`, `agent_cmd`,
`coder_cmd`, `chora_mcp`, `capture_cmd`, `playback_cmd`,
`skipped_tools`, `system_prompt`, `prompt_file`, `context_docs`, each
`*_url`, `*_headers` and each key with `api_key`. The agent never cuts
a security value. The question says only the key. The screen shows the
full value through the `voice.confirm` event. Only other values are cut to 60
characters for speech. The client `NAME-mcp`
needs `methods.privileged` for `settings.set` and `settings.reload`.
Its own `settings.reload` asks with the rule of `*.settings.reload`.

Reading files: the built-in tool `read {path, line?}` lists a folder
(sorted names, a folder with a trailing `/`, at most 200) or answers
the text of a file, without a confirmation. The roots are the project
folders of `agent_list`: `agent_projects` and the folders of
`$XDG_DATA_HOME/chora/inventory`. An empty path lists the projects; a
path is `PROJECT/REL` or absolute. The tool resolves the real path
(`..` and symbolic links) and refuses each path outside the real
roots. It opens the real path once, without a wait for a FIFO. On
that handle, it checks again that the file is inside a root, that it
is a regular file and that it has at most 4 MiB. It reads at most
4 MiB and one byte. It refuses binary files (a NUL byte) and text
that is not UTF-8. One answer has at most 64 KiB, cut at a line,
and names the line to read on with (`line`). The prompt says that the
text of a file is data, never instructions, and that the agent speaks
at most a short summary of it.

## 6. Error handling

| Component | Failure | Reaction |
|---|---|---|
| Mic | Capture command exits | Restart, backoff 0.5 s to 5 s, error earcon once |
| VAD | Model does not load | Exit 1; systemd restarts |
| Settings | The settings file is broken | At start: exit 1 with an error that names the file. On `restart`: the error, and the settings stay. A new connection to chora fails until the file is fixed. The agent never takes the defaults instead |
| ASR | Error or timeout | One retry with the same audio, then earcon and `phrases.asr_failed` ("I did not understand you.") when the agent listens again; a wake word asleep only sounds the earcon |
| TTS | Error or timeout | Earcon once per reply; the text stays in the history, in `last_reply` and on the HUD; the tool loop goes on; for the backoff of the TTS (1 s, doubling to 30 s), its sentences skip without a try |
| LLM | Connect error | One retry, then `phrases.llm_down` ("The language model does not answer.") |
| LLM | First token, idle or total timeout | Abort, keep the rounds so far, say so |
| LLM | Context error | Trim the history to half the budget, one retry |
| LLM | `tool_rounds` reached | A final round with `tool_choice: none` and a note: sum up in the reply language; without words, `phrases.tool_rounds`; history stays; new words of the user during that round get `tool_rounds` more rounds |
| LLM | Three cut answers in a row | A retry with a note after the first, the emergency brake after the second, `phrases.lost` after the third (section 3) |
| MCP | Child exits | Restart with backoff; call in flight gets "connection lost, result unknown"; tools reload |
| MCP | `tool_timeout_s` | Cancel the call; result "timeout, result unknown" |
| chora | Connection closed | Runtime reconnects and grabs PTT again; after 30 s, exit 1 |
| Any thread | Panic or end | Exit 1; the unit has `Restart=always` |

The history commits per LLM round; an audio failure never rolls it back.
`voice.health` and the `health` of the state record show `asr`, `tts`,
`llm`, `mcp` and `chora` as `ok`, `down` or `unknown` (before the first
report), with the last error and its Unix time `error_at`. Each actor
reports its successes and failures. At start, the agent warms the LLM
cache, again with the backoff of the LLM until it answers, so the cache
is warm when the LLM comes back. Then it says `phrases.ready`
("Ready.") once, also on the HUD, when it listens; empty turns it off.
A failed warm-up says nothing new. A probe connects to the websockets of
the ASR and the TTS, without a request, at start, after a new URL, every
30 s, and at the end of the backoff of a server that is down. Earcons
are synthesized tones: wake, sleep, cancel, error.

## 7. History and token budget

- `context_tokens` from the setting, else from the server
  (`context_length` in `/v1/models`; Gufo has 128000), else 32768.
- Budget: `context_tokens` minus `reply_reserve_tokens`. The estimate
  counts the characters of the prompt, the tools and the messages (a
  picture counts as 4000), divided by the characters per token of the
  last request: its length over its `usage.prompt_tokens` (3.0 at the
  start; requests with a picture teach nothing). No tokenizer.
- `history_tokens` (default 12000) bounds the messages after the
  system prompt, estimated as characters / 4; `0` keeps only the
  context budget. A long session (dump 2026-10-31: 221 messages, 33k
  tokens, full of cut answers) made the LLM copy its old answers.
- Before each request, drop the oldest whole turns until the estimate
  fits the budget and `history_tokens`; the last turn stays. A turn
  starts at a user message, so a tool call and its results go
  together. Before a new turn, old tool texts
  shrink to `old_tool_chars` and lose their pictures, and long arguments
  of old calls become `{"shortened": true}`; the last `put_asset` of
  each name stays whole.
- The requests of the last `saved_requests` turns go to
  `$XDG_STATE_HOME/chora/voice-history.json` (default
  `~/.local/state`), in the format of the Python agent (a list of user
  messages). At start, they go into the system prompt as quoted lines
  under "Earlier requests of the user", not as chat messages.
- The system prompt: `system_prompt`, or the built-in prompt
  (`src/prompt.md`, from the Python agent); then the `context_docs`
  (`FILE` or `FILE#HEADING` for one `## HEADING` section), or the
  built-in ones: `README.md`, `docs/glossary.md` and ten sections of
  `docs/api.md`, compiled into the binary; then `prompt_file`. A
  missing document or heading is an error at start.

## 8. ASR and TTS protocol (live probe, 2026-10-30)

ASR (`/v1/realtime?intent=transcription`, `turn_detection: null`):

- The server takes only mono PCM16 at 24000 Hz. At 16000 Hz it answers
  `invalid_audio_event`: "realtime input requires mono PCM16 at 24000
  Hz".
- It sends `conversation.item.input_audio_transcription.delta` events
  only after `input_audio_buffer.commit`, all within about 0.4 s, then
  `...completed`. A 3 s sentence took 0.44 s to 0.57 s after the commit.
- So text while the user speaks needs a new transcription of the growing
  buffer. The setting `asr_interim_ms` sets the interval; 0 turns it off.

TTS (`/v1/audio/speech/stream`, voice `aiden`):

- Per sentence: `audio.start` with `sentence_index`, `sentence_text`,
  `format` and `sample_rate` (24000), then binary PCM, then `audio.done`.
  `session.done` ends the session.
- The server does not split a text: "Eins. Zwei." and a text of three
  sentences each came as one `audio.start` with `sentence_index` 0 (WP4
  live check). So the agent splits the reply into sentences itself, one
  socket per sentence, as section 2 plans.
- `audio.start` gives the text and the start of its sound. The HUD
  typewriter spreads the words of `sentence_text` over the PCM length of
  the sentence.

## 9. Settings (D62)

File `$XDG_CONFIG_HOME/chora/chora-voice.yaml`. A missing key takes its
default, so an empty file is valid. `settings.set` applies at once (a
new `capture_cmd` restarts capture, a new `ptt_key` grabs again; WP5: a
new prompt warms the cache). Edits by hand apply on `voice.restart`. The
client name is the flag `--name` (default `voice`). `chora-voice
--print-config` prints all defaults.

| Key | Default |
|---|---|
| `asr_url`, `asr_model` | `ws://127.0.0.1:8881/v1/realtime?intent=transcription`, `gufo` |
| `tts_url`, `tts_model` | `ws://127.0.0.1:8882/v1/audio/speech/stream`, `gufo` |
| `llm_url`, `llm_model` | `http://127.0.0.1:8888/v1`, `gufo` |
| `asr_api_key_env`, `tts_api_key_env`, `llm_api_key_env` | `""` (no key; else the environment variable with the key) |
| `asr_api_key_file`, `tts_api_key_file`, `llm_api_key_file` | `""` (no key; else the file with the key, `%d` is `$CREDENTIALS_DIRECTORY`) |
| `asr_headers`, `tts_headers`, `llm_headers` | `{}` (extra headers, name: value) |
| `asr_language` | `auto` (the ASR finds the language; else a code such as `de`) |
| `asr_prompt` | `Chora, Coder` (plus the wake, sleep and stop words and the projects; `""`: no prompt) |
| `reply_language` | `en` (the language of the answers; `de` also gives German phrases and prompts) |
| `voice`, `tts_language` | `aiden`, `""` (follows `reply_language`: `en` gives `English`) |
| `asr_interim_ms` | `0` (off) |
| `capture_cmd` | `[pw-record, --raw, --rate, "24000", --channels, "1", --format, s16, "-"]` |
| `playback_cmd` | `[pw-play, --raw, --rate, "24000", --channels, "1", --format, s16, "-"]` |
| `ptt_key` | `Super+v` |
| `vad_threshold`, `vad_release`, `onset_ms` | `0.5`, `0.35`, `96` |
| `silence_ms`, `min_speech_ms`, `max_turn_ms` | `800`, `200`, `30000` |
| `preroll_ms` | `300` |
| `barge_in_vad_threshold`, `barge_in_ms` | `0.7`, `250` |
| `barge_in_echo_ratio`, `barge_in_min_rms` | `2.0`, `0` (adaptive; above 0: a fixed floor) |
| `barge_in_coupling` | `0.1` (start of the learned echo coupling) |
| `word_max_ms` | `3000` (speech, without pre-roll and end wait) |
| `sleep_words`, `wake_words` | `[sleep, go to sleep, schlaf, schlaf jetzt]`, `[hey chora, listen, hallo chora, hör zu]` |
| `cancel_words` | `[stop, cancel, never mind, stopp, abbrechen, halt, vergiss es]` |
| `code_cancel_words` | `[cancel the coding task, coding-aufgabe abbrechen, programmieraufgabe abbrechen]` |
| `allow_programs`, `allow_methods` | `[]`, `[]` (bare program names and `CLIENT.METHOD` that never ask; section 5) |
| `always_words` | `[immer, ja immer, always, yes always]` (a yes for the session; may be empty) |
| `yes_words`, `no_words` | `[yes, yeah, yep, yup, okay, ok, do it, please, sure, go ahead, ja, jo, jap, jep, klar, gern, gerne, genau, mach das, bitte]`, `[no, rather not, nein, nicht, lieber nicht]` |
| `filler_words` | `[please, now, bitte, jetzt, mal]` |
| `confirm`, `confirm_timeout_ms` | rules below, `15000` (from the end of the question) |
| `earcons`, `earcon_volume` | `true`, `0.3` |
| `indicator` | `true` (state dot under `screen:OUTPUT`); `false` turns it off; `{corner: bottom-right\|bottom-left\|top-right\|top-left, offset: [x, y]}` places it, `true` is `{corner: bottom-right, offset: [12, 12]}` |
| `system_prompt`, `context_docs` | `""`: the built-in prompt and documents |
| `prompt_file` | `~/.config/chora/voice-prompt.md` |
| `context_tokens`, `reply_reserve_tokens` | `0` (ask the server), `1024` |
| `tool_rounds`, `old_tool_chars`, `history_tokens`, `saved_requests` | `10`, `500`, `12000`, `20` |
| `reasoning_effort` | `none` (`""`: not in the request) |
| `asr_timeout_s`, `tts_timeout_s`, `tool_timeout_s` | `15`, `10`, `30` |
| `llm_first_token_s`, `llm_idle_s`, `llm_total_s` | `30`, `20`, `120` |
| `chora_mcp`, `coder_cmd` | `[choractl]`, `[]` (no coding tools; the task is the last argument) |
| `sandbox_cmd`, `unsandboxed` | `[agent-sandbox.sh]`, `false` (`coder_cmd` and `agent_cmd` run in the sandbox) |
| `agent_cmd`, `agent_projects` | `[]` (no agents; else an ACP command, `{dir}` and `{project}` replaced), `{}` (name: directory, besides the inventory) |
| `agent_voices`, `agent_said_chars`, `agent_permission_s` | `[ryan, vivian, serena, eric]`, `160`, `120` (section 18) |
| `skipped_tools` (globs) | `[subscribe_events, wait_events, capture, "voice.*"]`; they only hide tools: a call of the clients NAME, NAME-hud and NAME-mcp is refused, also through `call_method`, except the settings methods of NAME (section 5) |
| `playback_buffer_ms` | `1000` |
| `phrases` | `llm_down`, `llm_slow`, `tool_rounds`, `asr_failed`, `details`, `agent_asks`, `agent_ended`, `settings_set`, `settings_security`, `heard`, `ack`, `ready`, `allowed`, `lost`: English sentences, German with `reply_language: de` |
| `record_dir` | `""` (off; else records sessions for replay) |

`--target NAME` in `capture_cmd` or `playback_cmd` picks a PipeWire
device.

```yaml
confirm:
  - {tool: apply_commands, commands: [remove, close], prompt: "Shall I remove that?"}
  - {tool: unload_view, prompt: "Shall I remove the view {name}?"}
  - {tool: inject_input, prompt: "Shall I type or click?"}
  - {tool: code_task, prompt: "Shall I start the coding task?"}
  - {tool: agent_start, prompt: "Shall I start an agent for {project}?"}
  - {tool: run_program, prompt: "Shall I start the program {program}?"}
  - {tool: configure_output, prompt: "Shall I change the screen to {args}?"}
  - {tool: "*.restart", prompt: "Shall I restart the plugin?"}
  - {tool: call_method, method: "*restart", prompt: "Shall I restart the plugin {client}?"}
  # Then two rules for each privileged method METHOD of a plugin:
  # settings.set, settings.reload, app.open, scene.save, scene.rename,
  # scene.move, construct.load, construct.save and pane.remove.
  - {tool: "*.METHOD", prompt: "Shall I run {method} of the plugin {client}?"}
  - {tool: call_method, method: METHOD, prompt: "Shall I run {method} of the plugin {client}?"}
```

The grant `methods.privileged` opens every privileged method of every
plugin. The export flag `privileged` does not reach the agent, so the
default rules ask before each privileged method of the plugins. In a
prompt, `{client}` and `{method}` name the plugin and the method, also
for a plugin tool `CLIENT.METHOD`. The method is said with spaces
("app open").

Each server (ASR, TTS, LLM) has its own URL, model, key and headers, so
that each one can move to another provider. The key comes from the
variable of `*_api_key_env` or the file of `*_api_key_file`, never from
the settings file, and goes into `Authorization: Bearer KEY`: over HTTP
for the LLM, and in the websocket handshake of the ASR and the TTS, as
the realtime API of OpenAI takes it. The health probes and the live
commands use the same settings. The LLM must speak the OpenAI chat
completions API; other protocols come later. `https://` and `wss://` URLs use TLS
(rustls with ring, system root certificates); `http://` and `ws://` stay
in plain text.

The ASR gets no language with `asr_language: auto`, and `asr_prompt`
with the wake, sleep and stop words and the project names as the
`transcription.prompt` of the session. Gufo honors the prompt: without
it, a TTS clip of "Hallo Chora" came back as "Hello, Cora", with it as
"Hello, Chora" (3 of 3 tries each, 2026-10-31). On short or quiet sound the ASR
may answer the prompt itself; a transcript with at least half of the
words of the prompt, and at least 4, is dropped as such an echo. The system prompt ends
with a rule to answer in `reply_language`. With `reply_language: de`,
each phrase, confirm prompt, verb of an agent question and coding result
that keeps its English default is German; other languages keep the
English phrases until the settings set them. The default word lists
hold English and German words.

`check()` rejects wrong ranges, empty commands and word lists, a word in
two lists, a `ptt_key` that does not parse, a rule without a prompt, and
both a key variable and a key file for one server.

## 10. Methods and events

| Method of `voice` | Parameters | Result |
|---|---|---|
| `state` | none | the state record below |
| `cancel`, `sleep`, `wake` | none | the new state record |
| `mute` | `mic: bool`, `speaker: bool`, each optional | the new state record |
| `indicator.claim` | none | `{}`; hides the dot while the caller is connected |
| `restart` | none | reads the settings file again |
| `settings.get`, `settings.set` | D62 | D62 |
| `talk` | `down: bool` | PTT from another client, for example `chora-mc` |
| `confirm` (WP5, privileged) | `answer: yes\|no` | the new state record |
| `health` (WP8) | none | the health record of section 6 |
| `agent` (privileged) | `tool` (one of the agent tools) and its arguments | the text of the tool (section 18) |

The state record: `state`, `since_ms` (time in the state), `seq` (one
more per change), `awake`, `muted: {mic, speaker}`, `turn`,
`last_transcript`, `last_reply`, `queued` (texts that wait), `coder`
(the parameters of `agent.set` for the coder, or null before the first
task), `agents` (the parameters of `agent.set` of each ACP agent) and
`health` (section 6).

HUD: the agent emits client events (D65) on its connection `NAME`, so
a HUD subscribes to `voice.*` and needs no polling. The method `state`
stays for a full state record. The
indicator node signals without a HUD: with `indicator`, a dot of 12
pixels in a corner of the first output, under `screen:OUTPUT`, in the
color of the state (input `pass`). `offset` is its distance from the
corner. The `hud` thread places it once per connection, and again
when `indicator` changes.

A plugin that shows the state itself hides the dot with the method
`indicator.claim {}` of the client `voice`. The agent records the
client name of the caller. The dot hides while one claimer or more is
connected. On `client.disconnected` of a claimer, the agent drops its
claim. When no claimer is left, the dot comes back. `indicator: false`
turns the dot off for good. The claims do not survive a restart of
the agent. So a plugin claims again on `client.connected` with the
name `voice`, or when it gets its first `voice.*` event.

The events (D65): `voice.state {state, task, said}` on each change
(`state` mapped to the states of mc: `asleep`, `listening`, `thinking`,
`working`, `question`, `speaking`; `task` the request, `said` the last
reply), and `voice.say {state: speaking, text, duration_ms}` for each
sentence when its sound starts, with the length of its PCM (a late
sentence gets the rest of its length). The data are the parameters of
`mc agent.set` and `mc hud.say`, and chora-mc subscribes to `voice.*`.
A coding task is a sub-agent: `voice.state {name: coder, state, task,
said}` with `state` `working`, then `done` or `blocked` (failed or
cancelled), `task` the task, and `said` its last step, then its result.
An ACP agent (section 18) is a sub-agent too: `voice.state {name,
state, task, said, voice, plan, question, log}`, and `voice.remove
{name}` when it stops.
The `hud` thread waits for the sound of a sentence and hands the events
to the `chora` thread, which emits them on the connection of the
methods: only that connection has the name `voice`. The `chora` thread
looks for them every 50 ms between the events of chora. Over the rate
limit of the core (D65), the last `state` or `remove` of each name
waits 1 s and goes again, with the states that come meanwhile; a
refused `say` is gone. Another refused event logs at debug level.

## 11. Logging and latency

`tracing` with timestamps to stderr (journald), without colors when
stderr is no terminal. `RUST_LOG` filters; the default is
`info,tract_linalg=warn`. The lines are plain text in the form
`key=value`: one call site per level in `log.rs` keeps the `tracing`
macros out of the other functions, because clippy counts their expansion
into the cognitive complexity. With `CHORA_VOICE_LLM_DUMP=FILE` the agent
appends each LLM request body and the raw lines of its stream to FILE
(mode 0600), to see what the server got and sent. Each stage logs start
and end with the turn number. A tool call logs its arguments and the
start of its result, without secrets. Of `read`, the log has only the
path and the byte count. The values of each `*_headers` key, of each
key with `api_key`, `apikey`, `secret` or `password`, of each key that
ends with `token`, and of `authorization` show as "(hidden)". One summary per turn:

```
turn=17 speech=2.1s asr=380ms llm_first=650ms tts_first=310ms
first_sound=1.43s tools=2/3.4s barge_ins=0 outcome=answered
```

`first_sound` (end of speech to first sound) is the main metric. WP2
logs `turn`, `speech`, `length`, `end_wait` (last speech frame to the
end of the turn) and `awake` per utterance, for example
`turn=2 speech=2816ms length=3872ms end_wait=608ms awake=true`, and a
warning when the VAD is slower than real time. Not done: the same
record in `~/.local/state/chora/voice-turns.jsonl` (planned for WP8;
`tests/live-gate.sh` reads the log instead).

## 12. Recording for replay

With `record_dir`, each run writes a directory `DIR/YYYYMMDD-HHMMSS/`:

- `mic.wav`: all mic frames, 24 kHz, without gaps.
- `played.wav`: all played sound (earcons and replies), one piece after
  the other.
- `events.jsonl`: one line per gate event and per played piece, with
  the mic sample at which it happened, for example
  `{"at": 48000, "event": "onset"}` and
  `{"at": 96000, "event": "play", "from": 0, "len": 4800}`.

The WAV headers get their lengths at the end; a reader takes a cut file
up to its last whole sample. `chora-voice gate FILE.wav` runs the speech
gate of the settings over a file and prints the utterances.

For replay, a recording directory also holds `labels.txt`: one label
per line, times in s of `mic.wav`, `#` starts a comment.

```text
turn 1.09 2.83   # the user speaks from 1.09 s to 2.83 s
barge 2.08       # the user starts to speak over the agent
```

`chora-voice replay DIR...` runs the gates of the settings over each
recording and prints the metrics of section 13. It reads the `play`
lines of `events.jsonl` and, for the barge-in level, `played.wav`; a
plain clip needs only `mic.wav` and `labels.txt`.

## 13. Test plan

Unit tests: every transition of `step()`, also cancel in each busy
state; word matcher table with ASR variants and near misses ("schlafen
gehen"); echo filter; sentence splitter; SSE parser; tool call deltas;
confirm rules; token trimming; settings `check()`; the 3:2 low-pass.

Replay tests:

- A recording: the files of section 12, plus `labels.txt` (turns and
  barge-ins). Expected words come with the word matcher (WP6).
- The harness feeds the frames at full speed. While the agent plays,
  only the barge-in gate listens; a barge-in stops the playback for the
  rest of the recording's piece and starts an utterance with the
  barge-in speech and `preroll_ms` before it, as the agent does.
- Metrics against the labels: missed and false turns (a detected turn
  matches the first unmatched labelled turn that it overlaps), onset
  and offset error (detected speech start and last speech frame minus
  the label), missed and false barge-ins (a match lies from 0.1 s
  before to 1 s after the label), false barge-ins per minute of agent
  sound, and barge-in latency. The test `the_corpus_meets_the_thresholds`
  holds them to thresholds.
- The CI corpus is `services/voice/tests/data/corpus/` (1.7 MB):
  `hallo-chora` (the clip of WP2), `turns-noise`, `echo-barge` (echo of
  RMS 150) and `echo-loud` (echo of RMS 600, WP6), the last two with
  `played.wav`. `tests/corpus.py TTS_URL` writes the last three again
  with the live TTS (user voices `vivian` and `ryan`, agent voice
  `aiden`; noise from a fixed seed): `uv run tests/corpus.py
  ws://HOST:8882/v1/audio/speech/stream` in `services/voice`. The TTS
  output changes a little from run to run, so check the metrics after a
  new run. Recordings from `record_dir` stay outside the repo.

Fake servers for CI (`src/fake.rs`, WP4): a scripted ASR websocket
(text, error, silence, a first socket that fails), a TTS websocket
(silence, 60 ms per character, one sentence per `.` or `?`, a wrong
rate, no answer); WP5 added an LLM (`/models`, SSE with scripted words,
tool calls, HTTP errors and silence; a quarter of the request length as
`prompt_tokens`) and an in-process MCP server for `tools.rs`; WP6 added
an ASR with a text per utterance. With the fake tool executor, the test
`a_replay_runs_through_the_whole_agent` (WP6) feeds recorded speech
through the whole agent: a question, a cancel word over the reply, the
next question. It checks the state sequence and that the second LLM
request holds the played words with the cancel note.

Live tests: `services/voice/tests/live.py TTS_URL ASR_URL [CLIP]` checks
the TTS events, the ASR at 24 kHz and 16 kHz, and writes the test clip.
WP4 added the `chora-voice` subcommands `roundtrip [TEXT]` (TTS, then
its sound to the ASR, with the latencies), `tts TEXT [--out FILE]` and
`asr FILE`; WP3 added `replay DIR...`; WP5 added `ask TEXT [--speak]
[--yes]` (one turn with the LLM and the tools of `choractl mcp`, with
the latencies). WP6 added `levels` (mic level and speech probability
twice a second). The URLs come
from the settings. From the agent sandbox, the servers are at
`192.168.178.195`: ASR `:8881`, TTS `:8882`, LLM `:8888` (model `gufo`,
context 128000, text and image), reachable only with the sandbox off.
The live tests also check that `context_tokens: 0` reads 128000 and that
`describe_screen` sends the picture. WP8 added
`services/voice/tests/live-gate.sh [TEXT]`: one voice turn through the
whole agent in the running chora session, against the servers of the
settings, with the latency of each stage (section 15). In the agent
sandbox, it ran in a headless chora session with `chora-tiling`.

## 14. Dependencies and packaging

Reused from `Cargo.lock`: `serde`, `serde_json`, `schemars`,
`serde-saphyr`, `clap`, `tracing`, `tracing-subscriber`, `chora-kit`,
`chora-scene`; WP4 added `tokio` (the workspace features and `net`),
`futures-util` (`sink`) and `base64`; WP5 added `rmcp` (`client`,
`transport-child-process`) and the `tokio` feature `process`.

| New crate | Why |
|---|---|
| `tract-onnx` 0.22 | Silero VAD in pure Rust. 0.23 needs Rust 1.91; the workspace has 1.89 |
| `tokio-tungstenite` 0.28 (`rustls-tls-native-roots`; WP4) | ASR and TTS websockets, fake servers |
| `reqwest` 0.13 (no defaults; `stream`, `rustls-no-provider`; WP5) | LLM SSE and TLS (system roots through `rustls-platform-verifier`, the `rustls` ring provider is installed in `main`); chunked HTTP by hand is a bug source. It brings `hyper`, `tower` and `tower-http` (MIT); 0.13 is the version of the optional `rmcp` dependency |

`tract-linalg` builds its kernels with `cc` and needs `ar` at build time.

Not added: `pipewire`, `ort`, a resampler crate, a tokenizer, `regex`,
`hound`, `strsim`. A few lines replace each.

The Silero model (MIT, `LICENSES/MIT-silero-vad.txt`) goes into the
binary with `include_bytes!`. The crate lives in `services/voice/` (D53)
as a workspace member. No package and no unit until the agent completes
full turns; since WP6, `chora-voice.service` replaces
`voice-agent.service`.

## 15. Results of WP0 to WP8 (2026-10-30)

Silero on `tract`:

| Model | tract 0.22.5 | tract 0.23.9 |
|---|---|---|
| v5.1.2 `silero_vad.onnx` | fails | fails |
| v4.0 `silero_vad.onnx` | fails | not tried |
| v6.2.3 `silero_vad_op18_ifless.onnx` | works | not tried |

- v5: the export holds nested `If` nodes on the rate and on shapes (the
  LSTM squeezes an axis when it is 1). tract analyses both branches of
  each `If`, and the 8 kHz branch fails: "Attempt to squeeze an axis
  which dimension is not one". A fixed rate, fixed shapes or a symbolic
  length do not help. `onnxsim` and the onnxruntime optimizer cannot
  inline the branches either, because the branches use names from outer
  scopes.
- v4: the same pattern fails at the typing step ("then" and "else"
  branches give different ranks).
- v6 `op18_ifless`: Silero publishes this export for runtimes without
  `If` support. Only the rate check stays an `If`, and both of its
  branches analyse once the batch axis is the symbol `batch`. The model
  takes 576 samples (64 of the frame before, then 512) and a state of
  `[2, 1, 128]`. On the test clip, it finds the speech (more than 70 %
  of the frames over 0.5) and no speech in the quiet room. One frame
  takes about 5 ms in a debug build.
- `chora-voice` uses v6 `op18_ifless`. This is not one of the two
  versions of the decision (v5, else v4); see open question 1.

ASR at 16 kHz: refused (section 8). The agent keeps 24 kHz end to end;
only the VAD gets a 16 kHz copy.

`pw-record --raw`: the agent sandbox has no PipeWire and no ALSA tools,
so the spike could not run them. The option `-a, --raw` exists in
`pw-cat` since PipeWire 1.4.0 (not in 1.2.7). The Python agent noted
that `pw-record` without `--raw` "sniffs headers and chokes on raw
pipes". The capture and playback commands are settings: on a host
without `--raw`, set them to `arecord`/`aplay` with `-t raw`. Tests use
WAV files.

WP1 and WP2, live check in a headless chora session (a script feeds
the test clip in real time as the capture command):

- `voice.state`, `mute`, `sleep`, `wake` and `settings.set` answer;
  a wrong value answers code -32602 and changes nothing.
- The PTT grab works, and a new `ptt_key` by `settings.set` grabs
  again. A key press and release within one frame (32 ms) records
  nothing.
- The gate finds each pass of the clip: about 2.8 s of speech, the end
  600 ms after the last speech frame with `silence_ms: 600`.
- `record_dir` writes `mic.wav`, `played.wav` (the earcons) and
  `events.jsonl`. After a kill, the files hold everything up to the last
  second.

WP3, the CI corpus with the default settings (`chora-voice replay`):

| Recording | Turns | Missed | False | Onset error | Offset error | Barge-ins | Missed | False | Latency |
|---|---|---|---|---|---|---|---|---|---|
| `hallo-chora` | 1 | 0 | 0 | 2 ms | 62 ms | 0 | | | |
| `turns-noise` | 2 | 0 | 0 | 26 ms (max 50) | 50 ms (max 82) | 0 | | | |
| `echo-barge` | 1 | 0 | 1 | 160 ms | 82 ms | 1 | 0 | 1 | 416 ms |

- The fan noise (RMS 120) and the keyboard clicks start no turn; the
  short "Stopp." (0.48 s) is a turn.
- Echo of RMS 150 starts no barge-in. The barge-in comes 416 ms after
  the first sound of the user: 256 ms of `barge_in_ms` plus the rise of
  Silero over 0.7.
- Echo of RMS 600 without the user was a false barge-in (15 per minute
  of agent sound) and then a false turn: Silero gives the echo of the
  agent's voice a high probability, so the rule of WP3 never learned the
  echo level from it. WP6 takes the echo level from the played sound
  (section 4); see the WP6 results.
- The harness implements the barge-in gate of section 4 (`BargeIn` in
  `gate.rs`) and `Gate::open` for the utterance that a barge-in starts.
  The agent uses both since WP6.

WP4, live check against the servers (`chora-voice roundtrip`, debug
build, from the agent sandbox):

| Text | Audio | TTS first audio | TTS total | ASR final after commit |
|---|---|---|---|---|
| "Stopp." | 0.56 s | 466 to 472 ms | 0.80 to 0.83 s | 195 to 207 ms |
| "Hallo Chora. Zeig mir bitte die Agenda für heute." | 3.04 s | 401 to 490 ms | 4.07 s | 513 to 545 ms |
| Three sentences | 6.96 s | 444 ms | | 1309 ms |
| `echo-barge/mic.wav` (`asr FILE`) | 7.6 s | | | 666 ms |

- The ASR heard "Hallo Cora" for "Hallo Chora" each time, and
  "Vardamal" for "warte mal" over echo.
- The TTS is slower than real time for this text: 3.04 s of sound took
  4.07 s. Playback that starts at the first PCM runs dry. See open
  question 5.
- The time to the final text grows with the length of the utterance
  (0.2 s for 0.6 s, 1.3 s for 7 s). Interim texts do not shorten it: the
  server transcribes the whole buffer again at the commit.
- The agent with `asr_interim_ms: 1000` and a capture command that feeds
  `echo-barge/mic.wav` in real time: interim texts came every second
  (189 to 410 ms each), and the final text 346 to 416 ms after the end
  of the utterance.
- Deviation: the TTS client takes one text per socket and reports the
  sentences of the server; the split into sentences and one sentence
  ahead come with the reply pipeline (WP5, WP6). Timeouts: the ASR has
  `asr_timeout_s` for each whole try, the TTS has `tts_timeout_s` for
  each message. "Reconnect" is a new socket per request; the ASR retries
  once.
- For the HUD typewriter, `tts::Event::Sentence` comes before the PCM of
  its sentence; `tts::collect` gives the start and length of each
  sentence in samples. The HUD gets them with the speaking state (WP6)
  and the client events (WP8).

WP5, the LLM and the tool loop:

- Modules: `llm.rs` (SSE client), `tools.rs` (MCP client, confirm
  rules), `chat.rs` (prompt, history, budget, tool loop), `speech.rs`
  (sentence splitter, speech with playback buffer), and the reply states
  in `core.rs`.
- The splitter ends a sentence after `.!?;` and a space, or at a line
  break; not after a number ("3. Oktober") or a short abbreviation ("z.
  B."). The first piece of a reply may end at a comma after 40
  characters. It drops the marks of markdown, also backticks. A colon
  ends no sentence: it stands in inline code too ("`modifier: Alt`").
- One TTS socket per sentence, one after the other: the next sentence
  starts when the one before has all its sound, which is while it plays.
- MCP: a call has `tool_timeout_s`; a dropped call sends
  `notifications/cancelled` (rmcp `call_tool_once`). A lost child answers
  "the connection to chora was lost; the result is unknown"; the next
  call or turn connects again (backoff 0.5 s to 5 s) and loads the tools
  again. Tools reload at the start of each turn, so new plugin methods
  need no restart. Dots in tool names become `__` for the LLM.
- Deviations: `finish_reason: length` only logs. A cancel aborts the
  reply task: the round in flight does not go into the history, and the
  note "cancelled by the user; result unknown" comes with WP6. The
  final round keeps the tools and sends `tool_choice: none`: without
  the tools, Gufo read the whole prompt again (21 s instead of 1.4 s).

Live check, debug build, `chora-voice ask` in a headless chora session
with `chora-tiling` and one `foot` window, against Gufo (25 tools,
prompt 22700 tokens, warm cache):

| Request | Tools | First token | Total | Outcome |
|---|---|---|---|---|
| "Welche Fenster sind gerade offen? Schau bitte nach." | `query_scene` | 1.31 s | 3.6 s | answered |
| "Wie viele Fenster sind offen, und wie heißt das Layout?" (`--speak`) | `query_scene`, `tiling.get_layout` | 1.04 s | 14.7 s | answered |
| "Schließ bitte das Terminal-Fenster." (confirm: no) | `query_scene`, `apply_commands` declined | 0.91 s | 5.5 s | answered |
| Layout and plugins with `tool_rounds: 1` | three in one round | 0.55 s | 4.7 s | tool_rounds |
| The same in English | three in one round | 0.63 s | 13.5 s | tool_rounds |

- With `--speak`: the first sentence came 3.4 s after the request (two
  rounds), the TTS sent its first sound 0.56 s later, and playback
  started at 5.9 s, after 1 s of buffer. The TTS needs 3.2 s for the
  2.4 s of "Es ist ein Fenster offen, der Terminal foot.".
- After the declined `apply_commands`, the LLM said that the user
  declined and asked what to do. The final rounds summed up in German
  and in English.
- The agent itself started in the session, connected the tools as
  `voice-mcp`, warmed the cache in 0.4 s and answered `voice.confirm`
  and `voice.state`; a wrong answer gives -32602.

WP6, the state machine and the first full voice turns:

- Modules: `words.rs` (word matcher, echo filter), `mc.rs` (reports to
  chora-mc); the core has all states but `working` (WP7); `speech.rs`
  pauses and resumes the speech (`Control`) and reports the played PCM
  and each sentence with its start and length.
- Replay with the played sound (corpus regenerated, `echo-barge` with
  one coupling for both replies, `echo-loud` new; the corpus cut a
  barge-in reply 0.8 s after the label, not 0.4 s):

| Recording | Turns | Missed | False | Onset error | Offset error | Barge-ins | Missed | False | Latency |
|---|---|---|---|---|---|---|---|---|---|
| `echo-barge` | 1 | 0 | 0 | 160 ms | 82 ms | 1 | 0 | 0 | 416 ms |
| `echo-loud` | 1 | 0 | 0 | 180 ms | 70 ms | 1 | 0 | 0 | 436 ms |
| `hallo-chora` | 1 | 0 | 0 | 2 ms | 62 ms | 0 | | | |
| `turns-noise` | 2 | 0 | 0 | 26 ms (max 50) | 50 ms (max 82) | 0 | | | |

  The learned coupling is about 0.1 on `echo-loud`. With a start of
  0.05, `echo-loud` has one false barge-in before the coupling grows;
  with 0.1 and 0.3, none. The test now wants 0 false turns and
  barge-ins.
- Live, debug build, headless session with `chora-tiling`, one `foot`
  window and `chora-mc`, against Gufo; the mic was a script that writes
  a TTS clip (voice `vivian`) in real time, the speaker `cat >
  /dev/null`. "Welche Fenster sind gerade offen?" (2.2 s), warm cache:

| Stage | Time |
|---|---|
| Speech end to end of the utterance | 800 ms (`silence_ms`) |
| ASR, final text after the commit | 379 to 406 ms |
| LLM first token (a `query_scene` call) | 645 to 669 ms |
| Tool `query_scene` | 28 to 35 ms |
| First sentence after the ASR text (two rounds) | 2.4 to 2.6 s |
| TTS first audio | 408 to 424 ms |
| First sound after the end of the utterance | 4.8 to 5.0 s |

  `first_sound` holds the 1 s of `playback_buffer_ms`: the TTS is slower
  than real time. `mc agent.list` showed the agent `voice` with its
  task and reply; `voice.state` went back to `listening`. A cold cache
  (the LLM after a restart) took 40 s for the warm-up, and a turn waits
  for it.
- Live barge-in: the clip says "Abbrechen." 8 s after the question,
  over the reply. The barge-in came between two sentences (the TTS ran
  dry), the ASR heard "Abbrechen.", the word matcher found the cancel
  word, and the reply stopped. Two fixes came from it: speech in a dry
  gap of up to 1.5 s is a barge-in, not queued text; and the barge-in
  utterance keeps its speech and `preroll_ms` (with 100 ms, the ASR
  heard "verbrechen"). The LLM saw the note: to the next text it said
  "Ich habe meine Antwort nicht zu Ende sagen können."
- The TTS clips of short German phrases with `tts_language: auto` gave
  the ASR "April Rare" for "Abbrechen" and "Stopp auf Thema" for "Stopp,
  warte mal"; with `tts_language: German` it heard "Abbrechen." See open
  question 8.

WP7, the coder port (`coder.rs`):

- The tools of the Python agent: `code_task` (task, optional project),
  `code_status`, `code_cancel` and `list_projects`, for the LLM only
  with a `coder_cmd`, after the tools of chora; the confirm rule of
  `code_task` asks before a task. The prompt gets the coding part of
  the Python agent.
- A task clones `$XDG_DATA_HOME/chora/inventory/NAME` with
  `--no-hardlinks` to `worktrees/NAME-N` on the branch `voice/N` and
  removes `origin`; without a project, `git init`. The harness gets
  the task as its last argument, runs in its own process group with
  stdout and stderr to `worktrees/NAME-N.log`, and a line has at most
  16 MiB. stream-json lines give the steps (tool calls and texts) and
  the result; other lines are steps.
- A stop and the end of the harness end the whole process group (TERM,
  KILL 5 s later). `code_status` gives the time and the last step, or
  the last result; the spoken result is "The coding task in NAME-N is
  done|failed|was cancelled." with up to 400 characters of the result.
- The core: `working` while a task runs and the agent is awake; the
  result waits in a queue until the state is `listening` or `working`
  (no utterance, no text that waits, no reply), then a reply without
  the LLM says it and puts it into the history as an answer. A barge-in
  or a cancel word stops it as any reply. The task shows in chora-mc as
  the agent `coder`.
- Not checked live: a coding task through a spoken turn. The unit
  tests run `sh` scripts as the harness (clone without origin, log,
  result, cancel of a process group with a child).

WP8, error paths, health, indicator and the live gate:

- `health.rs`: one record for all actors; `voice.health` and the state
  record show it. The probes and the backoffs are in section 6.
- `tts_language` is empty by default and follows `language` (`de`
  gives `German`, unknown codes `auto`); open question 8 is decided.
- Live, headless session: the dot `voice-indicator` (owner
  `voice-hud`) sat at 1256, 776 on an output of 1280x800, blue, and
  turned gray with `voice.sleep`. `voice.health` showed all five `ok`
  4 s after the start; after `settings.set` of a `tts_url` without a
  server, `tts` was `down` with "Connection refused" within 2 s.
- `tests/live-gate.sh` against Gufo, debug build, headless session with
  `chora-tiling` and no window, "Welche Fenster sind gerade offen?"
  (clip 2.4 s, voice `vivian`):

| Stage | First run | Second run |
|---|---|---|
| LLM warm-up | 12.4 s (new prompt) | 1.1 s |
| Speech (VAD) | 2.27 s | 2.30 s |
| End of speech to end of the utterance | 800 ms | 800 ms |
| ASR, final text after the commit | 256 ms | 235 ms |
| LLM first token | 13.2 s | 705 ms |
| Tools | 2 (`query_scene`, `describe_screen`), 3 rounds | 1 (`query_scene`) 11 ms, 2 rounds |
| TTS first audio | 409 ms | 196 ms |
| First sound after the end of the utterance | 16.5 s | 2.7 s |

  In the first run, the first token came 13 s after the text, although
  the warm-up had ended: the request missed the cache of the server.
  The cause is open (the server may have served another client).

## 16. Work packages

| # | Package | Size |
|---|---|---|
| WP0 | Spikes: Silero on tract, ASR at 16 kHz, `pw-record --raw` (done) | S |
| WP1 | Crate, settings, `chora` thread, methods, PTT grab, logging (done) | M |
| WP2 | `audio_in` with Silero, `audio_out`, earcons, `record_dir` (done) | M |
| WP3 | Replay harness, labels, metrics with thresholds, CI corpus (done) | M |
| WP4 | ASR and TTS clients, interim text, fake servers (done) | M |
| WP5 | LLM client, tool loop, MCP reconnect and timeouts, token budget (done) | L |
| WP6 | State machine, word matcher, barge-in, cancel, sleep/wake, confirmation; first full voice turn; remove `agent.py` and `voice-agent.service` (done) | L |
| WP7 | Coder port (done) | S |
| WP8 | Error paths, health, indicator, live gate, client events for the HUD (D65) (done) | M |
| WP9 | Unit, README, package | S |
| WP10 | ACP sub-agents, one per project, with mc (section 18) (done) | M |

S: up to 1 day. M: 2 to 3 days. L: about 1 week.

## 17. Open questions

1. Silero version: v5 and v4 do not load in tract (section 15); v6
   `op18_ifless` does. Keep v6 on tract, or use `ort` with v5?
2. Mic mute: is "PTT still records" correct (section 3)?
3. `pw-record --raw` on the host: does the PipeWire of the host have
   1.4 or later? `pw-record --version` shows it.
4. Decided (WP6): the echo level comes from the played sound times a
   learned coupling, with a default that assumes the echo canceller. A
   recording with the echo canceller on the reference hardware would
   show the real coupling and check the default 0.1.
5. Answered for now (WP5): playback starts after `playback_buffer_ms`
   (1 s) of sound or at the end of the sentence, and again so after it
   ran dry.
6. The tools connect as `NAME-mcp` (`voice-mcp`), not as `voice`: the
   core refuses a second connection of a name with other `hello` flags
   (the runtime has `resume`, `choractl mcp` has `keep_nodes`). So the
   nodes of the LLM belong to `voice-mcp`, and the grant for the picture
   of `describe_screen` is `choractl grant voice-mcp screen.capture`.
   Keep this, or give `choractl mcp` a flag for the `hello` of the
   runtime?
7. The built-in prompt and documents are about 22700 tokens. A cold
   warm-up took 41 s on Gufo; with the cache 0.2 s to 10 s. Fewer
   sections of `docs/api.md` would make a cold start faster. A turn
   waits for the warm-up (WP6 live: 40 s on a cold cache).
8. Decided (WP8): `tts_language` follows `language` (`de` gives
   `German`); `auto` made the TTS clips of short German phrases hard
   to understand in WP6.
9. Queued text starts the next reply; it does not go into the running
   reply before its next LLM round (section 3). Is that enough?
10. `first_sound` is about 4.8 s after the end of an utterance with one
    tool call; the 1 s playback buffer and the 0.8 s `silence_ms` are a
    third of it. Lower them on the reference hardware? (WP8 live gate:
    2.7 s with one short sentence.)
11. The coder speaks its result in English ("The coding task in ... is
    done."), as the Python agent did, with the result of the harness in
    its own language. Make it a German phrase, or let the LLM sum it
    up (one more LLM round per task)?
12. Decided (D65): the agent emits `voice.state` and `voice.say`, and
    chora-mc subscribes to `voice.*` (section 10). `tests/voice-events.sh`
    checks it in a headless session.
13. Sub-agents (section 18): should agents start with the voice agent
    (a list of projects and names in the settings), and should they
    survive its restart (a detached supervisor that keeps the ACP
    pipes)? Now they end with it.
14. `agent-sandbox.sh` drops the arguments after the command, so
    `agent-sandbox.sh opencode acp` runs `opencode` without `acp`. Fix
    the script (`*) ARGS+=("$1") ;;`), or give `sandbox_cmd` a wrapper?
15. An ACP agent (opencode 1.19) calls `fs/write_text_file` although
    the client offers no file system; the client answers "method not
    found", and opencode writes the file itself. Offer the file system
    and the terminal of ACP, so that the voice agent sees each write?

## 18. ACP sub-agents (WP10, 2026-10-31)

The voice agent manages coding agents that speak the Agent Client
Protocol (ACP, <https://agentclientprotocol.com>): JSON-RPC 2.0 over the
stdin and stdout of the agent, one message per line. Module `agents.rs`.

- Spawn: one long-lived agent per project, by name. Its command line is
  `sandbox_cmd` and `agent_cmd`, with `{dir}` and `{project}` replaced,
  in the directory of the project, in its own process group. Without
  `unsandboxed: true`, `coder_cmd` and `agent_cmd` always run in
  `sandbox_cmd` (default `agent-sandbox.sh`: podman per start, project
  directory mounted, own home, read-only root, all capabilities
  dropped, network `ai-net`, ACP on stdio without a tty). The projects:
  `agent_projects` and the directories of the inventory; a name matches
  in any case.
- Protocol: `initialize` (protocol version 1, no file system, no
  terminal), `session/new {cwd, mcpServers: []}`, `session/prompt`
  with one text block, `session/update` notifications
  (`agent_message_chunk`, `tool_call`, `plan`; thoughts and the rest
  are ignored), the request `session/request_permission`, and the
  notification `session/cancel`. Other requests of the agent get
  "method not found". Prompts to a busy agent wait in a queue.
- States for mc: starting `thinking`, ready `idle`, prompt `working`,
  question `question`, end of turn `done`, cancelled `idle`, an error or
  a stop reason other than `end_turn` `blocked`; the end of its process
  `blocked` and the phrase `agent_ended`.
- Speech: at the end of a turn, its last message is spoken in the voice
  of the agent (`agent_voices` in turn, or the `voice` of
  `agent_start`), queued as the coder results: never over the user.
  At most 8 texts wait; more drop the oldest. A new question of an
  agent replaces its question that waits, and a question that the
  agent dropped is not spoken.
  More than `agent_said_chars` characters speak the first sentence and
  `phrases.details`. The history of the LLM gets the spoken text.
- Questions: a permission request is spoken as `agent_asks` ("Atlas
  möchte notes.txt ändern. Erlauben?": the file name of a path title
  and a verb for the kind). It waits from when it is spoken, so it
  also waits while the agent sleeps; the state is `confirming`. Yes and no words, `voice.confirm`, or "NAME, ja"
  answer it; two unclear answers or `agent_permission_s` without one
  are a no. Yes picks an `allow_once` option, no a `reject_once` one;
  without a fitting option the outcome is `cancelled`. A request whose
  only allowing option is `allow_always` is spoken as "Atlas wants to
  edit notes.txt. Always allow?" ("… Immer erlauben?" with
  `reply_language: de`), and only then a yes picks `allow_always`.
- Routing: the LLM gets `agent_list`, `agent_start {project, name?,
  voice?}`, `agent_say {name, text}`, `agent_stop {name}` and
  `agent_status {name?}`, and a part of the prompt. `agent_start` asks
  first (its `confirm` rule). An utterance whose first words are the
  name of an agent goes to it without the LLM: the rest is the prompt,
  a cancel word cancels its prompt and its queue, a yes or no answers
  its question. A name word of five letters or more may differ by one
  edit, a shorter one must be exact ("oder" is not "coder").
- Events: each change of an agent is `voice.state {name, state, task,
  said, voice, plan, question, log}` (`log`: the last 20 lines of its
  transcript, `›` for the user, `⚙` for a tool call, `?` for a
  question); `voice.remove {name}` when it stops. chora-mc shows the
  agents in the status line and the `agents` scene; `mc agent.focus`
  shows one agent with its transcript.
- Logs: `$XDG_DATA_HOME/chora/agents/NAME.log`, the stderr of the agent
  and every message (`>` sent, `<` received).
- Lifetime: `agent_stop` and the end of the voice agent (an error,
  `SIGTERM`, `SIGINT`) send `TERM` to the process group of each agent
  and `KILL` after 5 s. At the end, the voice agent waits until no
  process of these groups and of the coder runs; a zombie counts as
  ended. The agents do not survive a restart.

Tests: `tests/fake-acp.sh` is a fake agent (plan, tool call, question,
long answer); the unit test of `agents.rs` runs it through a prompt, a
yes, the short spoken answer and a stop. Core tests cover the question
in the voice of the agent and the address by name.

Live (2026-10-31, headless session with chora-mc, Gufo servers,
`unsandboxed: true` with opencode 1.19 `acp --pure` and Gufo as an
OpenAI-compatible provider, `permission.edit: ask`):

- With `voice.agent` calls: start to `idle` in about 5 s; the prompt
  "Schreib eine Datei notes.txt mit einer Zeile Hallo Welt" gave a
  permission request, spoken "Coder möchte notes.txt ändern. Erlauben?"
  in the voice `ryan`; `voice.confirm yes`; the file was written and
  "Die Datei notes.txt wurde mit der Zeile „Hallo Welt" geschrieben."
  was spoken; `agent_stop` removed the agent from mc.
- Spoken, with TTS clips into a fake mic: "Starte bitte einen Agenten
  für das Projekt demo mit dem Namen Atlas." made the LLM call
  `agent_start` (first with "Demo", now found in any case); "Atlas,
  schreib eine Datei ..." went to Atlas without the LLM; the spoken
  "Ja." answered its question. Its second question (a shell command)
  got no answer and was a no after 120 s. The ASR heard "Coder" as
  "Coda"; that utterance went to the LLM, which sent it on with
  `agent_say`.
