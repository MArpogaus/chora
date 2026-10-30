# chora-voice: the voice agent in Rust (draft specification)

Status: draft, 2026-10-30. WP0 to WP4 are done. Sources:
`services/voice/agent.py`, D53, D58, D61, D62, `docs/api.md`, the
`chora-kit` runtime and settings, `choractl mcp`, and live probes of the
Gufo servers.

`chora-voice` is the first voice agent of chora for real use. The Python
agent `services/voice/agent.py` is not in production use. It is only a
reference for the protocols of the servers and for the lessons of its
turns. When `chora-voice` completes a full voice turn (end of WP6), the
Python agent and `voice-agent.service` go.

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
| `tools` | MCP client of `choractl --name voice mcp`, the coder tools, the confirmation gate |
| `coder` | One background coding task, as in Python |
| `chora` | A thread with the `chora-kit` runtime: methods, settings, PTT grab, later the indicator node |
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
| `sleeping` | Hears only wake words and PTT |
| `listening` | Waits for speech |
| `recording` | Collects an utterance |
| `transcribing` | Waits for the ASR |
| `thinking` | Waits for the LLM, no sound |
| `tool` | A tool call runs |
| `confirming` | Waits for yes or no |
| `speaking` | Plays the reply |
| `working` | Listening while a coding task runs |

The agent starts in `listening` (awake).

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
| `confirming` | No, two unclear answers, or timeout | `thinking`; result "declined by the user" |
| `speaking` | Audio and LLM done | `listening` or `working` |
| busy states | Cancel word, `voice.cancel`, PTT down | `listening`, earcon |
| any | Sleep word or `voice.sleep` | `sleeping`, earcon |
| `sleeping` | `voice.wake` | `listening`, earcon |

Until WP6, `recording` ends in `transcribing` (asleep: `sleeping`), and
the transcript (or an ASR error with the error earcon) returns to
`listening`. The log and `last_transcript` show the text. The final
text of an older turn still counts; a late interim text drops.

Barge-in while `speaking`: speech over the barge-in gate (section 4) for
`barge_in_ms` stops playback at once. The unplayed sentences stay
queued. A cancel word drops them and cancels the turn. Empty text or
echo plays them from the cut sentence. Other text ends the reply; the
history keeps only the played text, and the new text starts a turn.

Speech while `thinking`, `tool` or `working`: each utterance goes to the
ASR at once. A cancel word cancels. Other text waits in a queue and goes
into the history before the next LLM round, or starts the next turn. No
speech is dropped.

Cancel stops only the current turn: it aborts the SSE request, sends
`notifications/cancelled` for the MCP call in flight, records "cancelled
by the user; result unknown" and stops playback. A cancel word does not
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

Echo: during playback, frames without speech update an echo level (a
running mean of RMS). Barge-in needs probability over
`barge_in_vad_threshold` and RMS over `max(barge_in_min_rms, echo level ×
barge_in_echo_ratio)`. Its pre-roll is at most 100 ms, which keeps the
agent's own words ("Ich" fragments) out. A transcript whose words match
more than 80 % of the words spoken in the last 3 s is echo.

Word matcher (no LLM):

1. Only utterances up to `word_max_ms` and four words count.
2. Normalize: lowercase, no punctuation, `ä→ae`, `ö→oe`, `ü→ue`, `ß→ss`.
3. Remove `filler_words`.
4. The rest must equal a phrase. Words of four letters or more may
   differ by one edit.
5. Order: code cancel, cancel, sleep, wake, yes, no. Yes and no count
   only in `confirming`.

While asleep, each short utterance goes to the ASR to find a wake word.
A keyword-spotting model can replace this later.

## 5. Confirmation

A rule matches by `tool` (glob), and optionally by the command kinds of
`apply_commands`, by `argv0` of `run_program`, or by `method` (glob) of
`call_method`. The agent speaks the `prompt` of the rule and waits for
yes or no. The method `confirm` answers from a key or the HUD.

## 6. Error handling

| Component | Failure | Reaction |
|---|---|---|
| Mic | Capture command exits | Restart, backoff 0.5 s to 5 s, error earcon once |
| VAD | Model does not load | Exit 1; systemd restarts |
| ASR | Error or timeout | One retry with the same audio, then earcon and "Ich habe dich nicht verstanden." |
| TTS | Error or timeout | Earcon; the text stays in the history and in `last_reply`; the tool loop goes on |
| LLM | Connect error | One retry, then "Das Sprachmodell antwortet nicht." |
| LLM | First token, idle or total timeout | Abort, keep the rounds so far, say so |
| LLM | Context error or `finish_reason: length` | Trim the history to half the budget, one retry |
| LLM | `tool_rounds` reached | German phrase from `phrases`; history stays |
| MCP | Child exits | Restart with backoff; call in flight gets "connection lost, result unknown"; tools reload |
| MCP | `tool_timeout_s` | Cancel the call; result "timeout, result unknown" |
| chora | Connection closed | Runtime reconnects and grabs PTT again; after 30 s, exit 1 |
| Any thread | Panic or end | Exit 1; the unit has `Restart=always` |

The history commits per LLM round; an audio failure never rolls it back.
`state.health` shows `asr`, `tts`, `llm` and `chora` as `ok` or `down`.
At start, the agent warms the LLM cache and checks ASR and TTS. Earcons
are synthesized tones: wake, sleep, cancel, question, error.

## 7. History and token budget

- `context_tokens` from the setting, else from the server (`/props` or
  `/v1/models`; Gufo has 128000), else 32768.
- Budget: `context_tokens` minus `reply_reserve_tokens`, the prompt and
  the tools. The estimate uses characters per token, learned from the
  last `usage.prompt_tokens`; no tokenizer.
- Before each request, drop the oldest whole turns until the estimate
  fits. Old tool texts shrink to `old_tool_chars`; the last `put_asset`
  of each name stays whole.
- The user words of `saved_requests` turns go to
  `~/.local/state/chora/voice-history.json`.

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
| `asr_url` | `ws://127.0.0.1:8881/v1/realtime?intent=transcription` |
| `tts_url` | `ws://127.0.0.1:8882/v1/audio/speech/stream` |
| `llm_url` | `http://127.0.0.1:8888/v1` |
| `model`, `voice` | `gufo`, `aiden` |
| `language`, `tts_language` | `de`, `auto` |
| `asr_interim_ms` | `0` (off) |
| `capture_cmd` | `[pw-record, --raw, --rate, "24000", --channels, "1", --format, s16, "-"]` |
| `playback_cmd` | `[pw-play, --raw, --rate, "24000", --channels, "1", --format, s16, "-"]` |
| `ptt_key` | `Super+v` |
| `vad_threshold`, `vad_release`, `onset_ms` | `0.5`, `0.35`, `96` |
| `silence_ms`, `min_speech_ms`, `max_turn_ms` | `800`, `200`, `30000` |
| `preroll_ms` | `300` |
| `barge_in_vad_threshold`, `barge_in_ms` | `0.7`, `250` |
| `barge_in_echo_ratio`, `barge_in_min_rms` | `2.0`, `300` |
| `word_max_ms` | `2000` |
| `sleep_words`, `wake_words` | `[schlaf, schlaf jetzt]`, `[hör zu]` |
| `cancel_words` | `[stopp, stop, abbrechen, halt, vergiss es]` |
| `code_cancel_words` | `[coding-aufgabe abbrechen, programmieraufgabe abbrechen]` |
| `yes_words`, `no_words` | `[ja, okay, genau, mach das]`, `[nein, nicht, lieber nicht]` |
| `filler_words` | `[bitte, jetzt, mal]` |
| `confirm`, `confirm_timeout_ms` | rules below, `8000` |
| `earcons`, `earcon_volume` | `true`, `0.3` |
| `indicator` | `true` (state dot under `screen:OUTPUT`) |
| `system_prompt`, `context_docs` | `""`: the built-in prompt and documents |
| `prompt_file` | `~/.config/chora/voice-prompt.md` |
| `context_tokens`, `reply_reserve_tokens` | `0` (ask the server), `1024` |
| `tool_rounds`, `old_tool_chars`, `saved_requests` | `10`, `500`, `20` |
| `reasoning_effort` | `none` |
| `asr_timeout_s`, `tts_timeout_s`, `tool_timeout_s` | `15`, `10`, `30` |
| `llm_first_token_s`, `llm_idle_s`, `llm_total_s` | `30`, `20`, `120` |
| `chora_mcp`, `coder_cmd` | `[choractl]`, `[]` |
| `skipped_tools` | `[subscribe_events, wait_events, capture]` |
| `record_dir` | `""` (off; else records sessions for replay) |

`--target NAME` in `capture_cmd` or `playback_cmd` picks a PipeWire
device. The German canned sentences (`phrases`) come with WP5.

```yaml
confirm:
  - {tool: apply_commands, commands: [remove, close], prompt: "Soll ich das entfernen?"}
  - {tool: unload_view, prompt: "Soll ich die Ansicht entfernen?"}
  - {tool: inject_input, prompt: "Soll ich tippen oder klicken?"}
  - {tool: code_task, prompt: "Soll ich die Programmieraufgabe starten?"}
  - {tool: run_program, argv0: [systemctl, loginctl, shutdown, reboot, poweroff],
     prompt: "Soll ich den Sitzungsbefehl ausführen?"}
  - {tool: configure_output, prompt: "Soll ich den Bildschirm ändern?"}
  - {tool: "*.restart", prompt: "Soll ich das Plugin neu starten?"}
  - {tool: call_method, method: "*.restart", prompt: "Soll ich das Plugin neu starten?"}
```

`check()` rejects wrong ranges, empty commands and word lists, a word in
two lists, a `ptt_key` that does not parse, and a rule without a prompt.

## 10. Methods and events

| Method of `voice` | Parameters | Result |
|---|---|---|
| `state` | none | the state record below |
| `cancel`, `sleep`, `wake` | none | the new state record |
| `mute` | `mic: bool`, `speaker: bool`, each optional | the new state record |
| `restart` | none | reads the settings file again |
| `settings.get`, `settings.set` | D62 | D62 |
| `talk` (WP6) | `down: bool` | PTT from another client, for example `chora-mc` |
| `confirm` (WP6) | `answer: yes\|no` | the new state record |

The state record: `state`, `since_ms` (time in the state), `seq` (one
more per change), `awake`, `muted: {mic, speaker}`, `turn`,
`last_transcript`, `last_reply`. Later packages add `confirming`,
`coder` and `health`.

HUD: the core will get client-emitted events (a new decision record,
implemented by another package). Then the agent emits the event
`voice.state` with the state record on each change of `seq`. Until then,
a HUD polls the method `state` (4 Hz; `seq` shows a change). The
indicator node signals without a HUD.

## 11. Logging and latency

`tracing` with timestamps to stderr (journald), without colors when
stderr is no terminal. `RUST_LOG` filters; the default is
`info,tract_linalg=warn`. The lines are plain text in the form
`key=value`: one call site per level in `log.rs` keeps the `tracing`
macros out of the other functions, because clippy counts their expansion
into the cognitive complexity. Each stage logs start and end with the
turn number. One summary per turn:

```
turn=17 speech=2.1s asr=380ms llm_first=650ms tts_first=310ms
first_sound=1.43s tools=2/3.4s barge_ins=0 outcome=answered
```

`first_sound` (end of speech to first sound) is the main metric. WP2
logs `turn`, `speech`, `length`, `end_wait` (last speech frame to the
end of the turn) and `awake` per utterance, for example
`turn=2 speech=2816ms length=3872ms end_wait=608ms awake=true`, and a
warning when the VAD is slower than real time. WP8 writes the same
record to `~/.local/state/chora/voice-turns.jsonl`.

## 12. Recording for replay

With `record_dir`, each run writes a directory `DIR/YYYYMMDD-HHMMSS/`:

- `mic.wav`: all mic frames, 24 kHz, without gaps.
- `played.wav`: all played sound, one piece after the other.
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
recording and prints the metrics of section 13. It reads only the
`play` lines of `events.jsonl` (the agent's sound) and not
`played.wav`; a plain clip needs only `mic.wav` and `labels.txt`.

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
  rest of the recording's piece and starts an utterance with 100 ms of
  pre-roll.
- Metrics against the labels: missed and false turns (a detected turn
  matches the first unmatched labelled turn that it overlaps), onset
  and offset error (detected speech start and last speech frame minus
  the label), missed and false barge-ins (a match lies from 0.1 s
  before to 1 s after the label), false barge-ins per minute of agent
  sound, and barge-in latency. The test `the_corpus_meets_the_thresholds`
  holds them to thresholds.
- The CI corpus is `services/voice/tests/data/corpus/` (648 KB of new
  clips): `hallo-chora` (the clip of WP2), `turns-noise` and
  `echo-barge`. `tests/corpus.py TTS_URL` writes the last two again
  with the live TTS (user voices `vivian` and `ryan`, agent voice
  `aiden`; noise from a fixed seed): `uv run tests/corpus.py
  ws://HOST:8882/v1/audio/speech/stream` in `services/voice`. The TTS
  output changes a little from run to run, so check the metrics after a
  new run. Recordings from `record_dir` stay outside the repo.

Fake servers for CI (`src/fake.rs`, WP4): a scripted ASR websocket
(text, error, silence, a first socket that fails), a TTS websocket
(silence, 60 ms per character, one sentence per `.` or `?`, a wrong
rate, no answer); WP5 adds an SSE LLM with scripted tool calls. With a
fake tool executor, an end-to-end test runs a replay through the whole
agent and checks the LLM requests and the state sequence.

Live tests: `services/voice/tests/live.py TTS_URL ASR_URL [CLIP]` checks
the TTS events, the ASR at 24 kHz and 16 kHz, and writes the test clip.
WP4 added the `chora-voice` subcommands `roundtrip [TEXT]` (TTS, then
its sound to the ASR, with the latencies), `tts TEXT [--out FILE]` and
`asr FILE`; WP3 added `replay DIR...`. WP5 and WP6 add `levels` and
`tools`. The URLs come
from the settings. From the agent sandbox, the servers are at
`192.168.178.195`: ASR `:8881`, TTS `:8882`, LLM `:8888` (model `gufo`,
context 128000, text and image), reachable only with the sandbox off.
The live tests also check that `context_tokens: 0` reads 128000 and that
`describe_screen` sends the picture. `tests/gate-voice.sh` runs them in
a headless chora session.

## 14. Dependencies and packaging

Reused from `Cargo.lock`: `serde`, `serde_json`, `schemars`,
`serde-saphyr`, `clap`, `tracing`, `tracing-subscriber`, `chora-kit`,
`chora-scene`; WP4 added `tokio` (the workspace features and `net`),
`futures-util` (`sink`) and `base64`; WP5 adds `rmcp` (`client`,
`transport-child-process`) and the `tokio` features it needs.

| New crate | Why |
|---|---|
| `tract-onnx` 0.22 | Silero VAD in pure Rust. 0.23 needs Rust 1.91; the workspace has 1.89 |
| `tokio-tungstenite` 0.28 (default features: no TLS; WP4) | ASR and TTS websockets, fake servers |
| `reqwest` (no defaults; `json`, `stream`, WP5) | LLM SSE; chunked HTTP by hand is a bug source |

`tract-linalg` builds its kernels with `cc` and needs `ar` at build time.

Not added: `pipewire`, `ort`, a resampler crate, a tokenizer, `regex`,
`hound`, `strsim`. A few lines replace each.

The Silero model (MIT, `LICENSES/MIT-silero-vad.txt`) goes into the
binary with `include_bytes!`. The crate lives in `services/voice/` (D53)
as a workspace member. No package and no unit until the agent completes
full turns; then `chora-voice.service` replaces `voice-agent.service`.

## 15. Results of WP0 to WP4 (2026-10-30)

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
- Echo of RMS 600 without the user is a false barge-in (15 per minute of
  agent sound in this corpus) and then a false turn. Silero gives the
  echo of the agent's voice a high probability, so the rule of section
  4 never learns the echo level from it, and RMS 600 is over
  `barge_in_min_rms`. See open question 4. The test allows this one
  false barge-in and false turn; the other thresholds are 0 missed
  turns and barge-ins, 200 ms onset and offset error, 450 ms barge-in
  latency.
- The harness implements the barge-in gate of section 4 (`BargeIn` in
  `gate.rs`; the echo level is a running mean over about 0.5 s) and
  `Gate::open` for the utterance that a barge-in starts. WP6 uses both.

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

## 16. Work packages

| # | Package | Size |
|---|---|---|
| WP0 | Spikes: Silero on tract, ASR at 16 kHz, `pw-record --raw` (done) | S |
| WP1 | Crate, settings, `chora` thread, methods, PTT grab, logging (done) | M |
| WP2 | `audio_in` with Silero, `audio_out`, earcons, `record_dir` (done) | M |
| WP3 | Replay harness, labels, metrics with thresholds, CI corpus (done) | M |
| WP4 | ASR and TTS clients, interim text, fake servers (done) | M |
| WP5 | LLM client, tool loop, MCP reconnect and timeouts, token budget | L |
| WP6 | State machine, word matcher, barge-in, cancel, sleep/wake, confirmation; first full voice turn; remove `agent.py` and `voice-agent.service` | L |
| WP7 | Coder port | S |
| WP8 | Error paths, health, indicator, client events for the HUD, live gate | M |
| WP9 | Unit, README, package | S |

S: up to 1 day. M: 2 to 3 days. L: about 1 week.

## 17. Open questions

1. Silero version: v5 and v4 do not load in tract (section 15); v6
   `op18_ifless` does. Keep v6 on tract, or use `ort` with v5?
2. Mic mute: is "PTT still records" correct (section 3)?
3. `pw-record --raw` on the host: does the PipeWire of the host have
   1.4 or later? `pw-record --version` shows it.
4. Barge-in over loud echo (section 15, WP3): the echo level of section
   4 learns only from frames that Silero calls quiet, and the echo of a
   voice is speech for Silero. Take the echo level from the played sound
   instead (the RMS of the played frames times a coupling that the
   agent learns), or keep the rule and trust the echo canceller? A
   recording with the echo canceller on the reference hardware would
   show the real echo level.
5. The TTS is slower than real time for longer sentences (section 15,
   WP4). Start playback only after a buffer (for example 1 s, or the
   whole first sentence), or accept gaps?
