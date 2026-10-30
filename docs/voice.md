# chora-voice: the voice agent in Rust (draft specification)

Status: draft, 2026-10-30. Sources: `services/voice/agent.py`, D53, D58,
D61, D62, `docs/api.md`, the `chora-kit` runtime and settings,
`choractl mcp`.

## 1. Goals and non-goals

Goals:

- One binary `chora-voice`: a chora client on the `chora-kit` runtime,
  with D62 settings in place of `voice.env`.
- Silero VAD (ONNX) as the speech gate.
- Always listening, with the local words "Schlaf" and "Hör zu", plus the
  push-to-talk (PTT) key.
- Voice cancel at any time, also while the agent speaks or works.
- Spoken yes/no confirmation before the tools of a configurable list.
- No silent failure; a latency per stage for each turn; a state for the
  HUD.
- Replay tests from recorded audio that measure speech detection and
  barge-in.

Non-goals: new ASR, TTS or LLM servers; a keyword-spotting model; an own
echo canceller (the PipeWire one stays); a GUI; features beyond the
Python agent and the known weaknesses.

## 2. Architecture

One tokio process. Each actor is one task, and actors talk through
channels. The core holds the state machine as a pure function
`step(state, event) -> (state, effects)`, so unit tests need no IO.

| Actor | Job |
|---|---|
| `audio_in` | `pw-record --raw` at 16 kHz mono, 32 ms frames (512 samples), Silero probability and RMS per frame |
| `audio_out` | `pw-play --raw` at 24 kHz for TTS and earcons; reports `played_until`; stops at once |
| `asr` | One websocket per utterance to Qwen3-ASR realtime, PCM in 1 s pieces |
| `tts` | One websocket per sentence to Qwen3-TTS, one sentence ahead |
| `llm` | `/chat/completions` with SSE and tools |
| `tools` | MCP client of `choractl --name voice mcp`, the coder tools, the confirmation gate |
| `coder` | One background coding task, as in Python |
| `chora` | A std thread with the `chora-kit` runtime: methods, settings, PTT grab, indicator node |
| `core` | State machine, history, word matcher, turn metrics |

Audio: Silero needs 16 kHz, and the ASR gets the same PCM, so no
resampler is necessary. Subprocesses replace the `pipewire` crate, which
needs bindgen and C headers. Fallback: `arecord`/`aplay`.

Tool layer: LLM tools go through MCP (`rmcp` client). The tool schemas
and the plugin tools stay in one place, and `chora_mcp` can still run
tools in a container. The agent's own needs (PTT grab, methods,
settings, indicator) go through the runtime connection. This replaces
the PTT polling through `wait_events`.

`runtime::serve()` blocks on `next_event()`. The `chora` thread loops on
`next_event_timeout(50 ms)` to also take commands from the core, so WP1
makes `runtime::dispatch` public. Methods answer from a shared snapshot
and send commands to the core through a channel.

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

| From | Event | To |
|---|---|---|
| `sleeping` | Short utterance with a wake word | `listening`, earcon |
| `sleeping` | PTT down | `recording`, then back to `sleeping` |
| `listening`, `working` | VAD onset or PTT down | `recording` |
| `recording` | `silence_ms` quiet, PTT up, or `max_turn_ms` | `transcribing` |
| `recording` | Less than `min_speech_ms` speech | previous state |
| `transcribing` | Sleep word | `sleeping`, earcon |
| `transcribing` | Cancel word, empty text or echo | previous state |
| `transcribing` | Other text | `thinking` |
| `thinking` | First sentence | `speaking`; tools run in parallel |
| `thinking` | Tool call on the confirm list | `confirming` |
| `thinking` | Other tool call | `tool` |
| `tool` | Result or timeout | `thinking` |
| `confirming` | Yes | `tool` |
| `confirming` | No, two unclear answers, or timeout | `thinking`; result "declined by the user" |
| `speaking` | Audio and LLM done | `listening` or `working` |
| busy states | Cancel word, `voice.cancel`, PTT down | `listening`, earcon |
| any | Sleep word or `voice.sleep` | `sleeping` |

Barge-in while `speaking`: speech over the barge-in gate (section 4) for
`barge_in_ms` stops playback at once. The unplayed sentences stay
queued. A cancel word drops them and cancels the turn. Empty text or
echo plays them from the cut sentence. Other text ends the reply; the
history keeps only the played text, and the new text starts a turn.

Speech while `thinking`, `tool` or `working`: each utterance goes to the
ASR at once. A cancel word cancels. Other text waits in a queue and goes
into the history before the next LLM round, or starts the next turn. No
speech is dropped.

Cancel aborts the SSE request, sends `notifications/cancelled` for the
MCP call in flight, records "cancelled by the user; result unknown" and
stops playback. It does not stop a coding task. Coding results wait for
`listening` or `working`, and also go into the history.

## 4. Speech gate, echo, word matcher

Onset: probability over `vad_threshold` for `onset_ms`. End: under
`vad_release` for `silence_ms`. An utterance keeps `preroll_ms` before
its onset.

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
5. Order: cancel, sleep, wake, yes, no. Yes and no count only in
   `confirming`.

## 5. Confirmation

A rule matches by `tool` (glob), and optionally by the command kinds of
`apply_commands` or by `argv0` of `run_program`. The agent speaks the
`prompt` of the rule and waits for yes or no. The method `confirm`
answers from a key or the HUD.

## 6. Error handling

| Component | Failure | Reaction |
|---|---|---|
| Mic | `pw-record` exits | Restart, backoff 0.5 s to 5 s, error earcon once |
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
| Any task | Panic or end | Exit 1; the unit has `Restart=always` |

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
- Persistence as in Python: user words of `saved_requests` turns in
  `~/.local/state/chora/voice-history.json`. Both agents share the file.

## 8. Settings (D62)

File `$XDG_CONFIG_HOME/chora/chora-voice.yaml`. `settings.set` applies
at once (a new `source` restarts capture, a new `ptt_key` grabs again, a
new prompt warms the cache). Edits by hand apply on `voice.restart`. The
client name is the flag `--name` (default `voice`).

| Key | Default |
|---|---|
| `asr_url` | `ws://127.0.0.1:8881/v1/realtime?intent=transcription` |
| `tts_url` | `ws://127.0.0.1:8882/v1/audio/speech/stream` |
| `llm_url` | `http://127.0.0.1:8888/v1` |
| `model`, `voice` | `gufo`, `aiden` |
| `language`, `tts_language` | `de`, `auto` |
| `source`, `sink` | `""` (defaults) |
| `start_awake` | `true` |
| `ptt_key` | `Super+v` |
| `vad_threshold`, `vad_release`, `onset_ms` | `0.5`, `0.35`, `96` |
| `silence_ms`, `min_speech_ms`, `max_turn_ms` | `800`, `200`, `30000` |
| `preroll_ms` | `300` |
| `barge_in_vad_threshold`, `barge_in_ms` | `0.7`, `250` |
| `barge_in_echo_ratio`, `barge_in_min_rms` | `2.0`, `300` |
| `word_max_ms` | `2000` |
| `sleep_words`, `wake_words` | `[schlaf, schlaf jetzt]`, `[hör zu]` |
| `cancel_words` | `[stopp, stop, abbrechen, halt, vergiss es]` |
| `yes_words`, `no_words` | `[ja, okay, genau, mach das]`, `[nein, nicht, lieber nicht]` |
| `filler_words` | `[bitte, jetzt, mal]` |
| `confirm`, `confirm_timeout_ms` | rules below, `8000` |
| `earcons`, `earcon_volume` | `true`, `0.3` |
| `indicator` | `true` (state dot under `screen:OUTPUT`) |
| `system_prompt`, `context_docs` | the Python `SYSTEM` and `CONTEXT_DOCS` |
| `prompt_file` | `~/.config/chora/voice-prompt.md` |
| `context_tokens`, `reply_reserve_tokens` | `0` (ask the server), `1024` |
| `tool_rounds`, `old_tool_chars`, `saved_requests` | `10`, `500`, `20` |
| `reasoning_effort` | `none` |
| `asr_timeout_s`, `tts_timeout_s`, `tool_timeout_s` | `15`, `10`, `30` |
| `llm_first_token_s`, `llm_idle_s`, `llm_total_s` | `30`, `20`, `120` |
| `chora_mcp`, `coder_cmd` | `choractl`, `""` |
| `skipped_tools` | `[subscribe_events, wait_events, capture]` |
| `phrases` | German canned sentences by key |
| `record_dir` | `""` (off; else records sessions for replay) |

```yaml
confirm:
  - {tool: apply_commands, commands: [remove, close], prompt: "Soll ich das entfernen?"}
  - {tool: unload_view, prompt: "Soll ich die Ansicht entfernen?"}
  - {tool: inject_input, prompt: "Soll ich tippen oder klicken?"}
  - {tool: code_task, prompt: "Soll ich die Programmieraufgabe starten?"}
  - {tool: run_program, argv0: [systemctl, loginctl, shutdown, reboot, poweroff],
     prompt: "Soll ich den Sitzungsbefehl ausführen?"}
```

`check()` rejects wrong ranges, empty lists and a word in two lists.

## 9. Methods and events

| Method of `voice` | Parameters | Result |
|---|---|---|
| `state` | none | `state`, `since_ms`, `seq`, `awake`, `muted`, `turn`, `last_transcript`, `last_reply`, `confirming`, `coder`, `health` |
| `cancel`, `sleep`, `wake` | none | new state |
| `talk` | `down: bool` | PTT from another client, for example `chora-mc` |
| `confirm` | `answer: yes\|no` | new state |
| `mute` | `mic: bool`, `speaker: bool` | new state |
| `restart` | none | reads the settings file again |
| `settings.get`, `settings.set` | D62 | D62 |

The core has no client events. The HUD polls `state` (4 Hz; `seq` shows
a change). The indicator node signals without a HUD.

## 10. Logging and latency

`tracing` with timestamps to stderr (journald). Each stage logs start
and end with the turn number. One summary per turn:

```
turn=17 speech=2.1s asr=380ms llm_first=650ms tts_first=310ms
first_sound=1.43s tools=2/3.4s barge_ins=0 outcome=answered
```

`first_sound` (end of speech to first sound) is the main metric. The
same record goes to `~/.local/state/chora/voice-turns.jsonl`.

## 11. Test plan

Unit tests: every transition of `step()`, also cancel in each busy
state; word matcher table with ASR variants and near misses ("schlafen
gehen"); echo filter; sentence splitter; SSE parser; tool call deltas;
confirm rules; token trimming; settings `check()`.

Replay tests:

- A recording: a 16 kHz WAV of the mic, a WAV of the played audio, and
  YAML labels (utterances, expected words, barge-ins, playback spans).
- The harness feeds the frames at full speed under a paused tokio clock.
- Metrics: turn boundary precision and recall (200 ms tolerance), onset
  error, false barge-ins per minute of agent speech, barge-in latency,
  word accuracy.
- A port of the Python RMS gate is the baseline. Silero must match or
  beat it on each metric.
- A small synthetic corpus (TTS voices, noise, echo) is in the repo for
  CI. Recordings from `record_dir` stay outside the repo.

Fake servers for CI: a scripted ASR websocket, a TTS websocket (silence,
60 ms per character) and an SSE LLM with scripted tool calls. With a
fake tool executor, an end-to-end test runs a replay through the whole
agent and checks the LLM requests and the state sequence.

Live tests: the subcommands of Python (`roundtrip`, `tts`, `asr FILE`,
`levels`, `tools`) plus `replay DIR`. The URLs come from the settings.
From the agent sandbox, the servers are at `192.168.178.195`: ASR
`:8881`, TTS `:8882`, LLM `:8888` (context 128000, text and image). The
live tests also check that `context_tokens: 0` reads 128000 and that
`describe_screen` sends the picture. `tests/gate-voice.sh` runs them in
a headless chora session.

## 12. Dependencies and packaging

Reused from `Cargo.lock`: `tokio` (add `process`, `io-util`, `net`,
`signal`; `test-util` for tests), `rmcp` (add `client`,
`transport-child-process`), `futures-util`, `serde`, `serde_json`,
`schemars`, `serde-saphyr`, `base64`, `clap`, `tracing`,
`tracing-subscriber`, `chora-kit`, `chora-scene`.

| New crate | Why |
|---|---|
| `tokio-tungstenite` (no TLS) | ASR and TTS websockets, fake servers |
| `reqwest` (no defaults; `json`, `stream`) | LLM SSE; chunked HTTP by hand is a bug source |
| `ort` (`load-dynamic`) | Silero with the system `libonnxruntime` |

Not added: `pipewire`, a resampler, a tokenizer, `regex`, `hound`,
`strsim`. A few lines replace each.

The Silero model (MIT, about 2 MB) goes into the binary with
`include_bytes!`. The crate lives in `services/voice/` (D53) as a
workspace member. `chora-voice.service` has
`Conflicts=voice-agent.service`. No package until the switch.

## 13. Migration

1. Both units exist. The Python agent stays the default.
2. `chora-voice import-env` converts `voice.env` to YAML and reports the
   variables without an equivalent (`RMS_*`, `SPEECH_RATIO`,
   `SETTLE_MS`, `BARGE_MS`).
3. Both agents share the history file.
4. Switch criteria: replay metrics at least as good as the baseline;
   `first_sound` p50 not worse than Python; one week of daily use
   without a crash restart.
5. After the switch: a decision record, a new README, and removal of
   `agent.py` and `voice-agent.service` in the next release.

## 14. Work packages

| # | Package | Size |
|---|---|---|
| WP0 | Spikes: `pw-record --raw`, Gufo ASR at 16 kHz, `ort` loads Silero | S |
| WP1 | Crate, settings, `chora` thread, methods, PTT grab, tracing | M |
| WP2 | `audio_in` with Silero, `audio_out`, earcons, `record_dir` | M |
| WP3 | Replay harness, labels, RMS baseline, CI corpus | M |
| WP4 | ASR and TTS clients, fake servers | M |
| WP5 | LLM client, tool loop, MCP reconnect and timeouts, token budget | L |
| WP6 | State machine, word matcher, barge-in, cancel, sleep/wake, confirmation | L |
| WP7 | Coder port | S |
| WP8 | Error paths, health, indicator, live gate | M |
| WP9 | `import-env`, unit, docs, side-by-side week | S |

S: up to 1 day. M: 2 to 3 days. L: about 1 week.

## 15. Open questions

1. Silero runtime: `ort` with the system `libonnxruntime`, or pure-Rust
   `tract` (no C library, longer build, Silero v5 support to verify)?
2. HUD signal: is polling `voice.state` enough, or should the core get a
   way for a client to emit its own events?
3. Mute: mic only (deaf except PTT), speaker only (text on the HUD), or
   both as separate switches as proposed?
4. Session commands: is the proposed rule (`run_program` with
   `systemctl`, `loginctl`, `shutdown`, `reboot`, `poweroff`) correct?
   Should `configure_output` and plugin `restart` methods also ask?
5. While asleep, each short utterance goes to the ASR server to find the
   wake word. Is that acceptable, or is a keyword-spotting model
   necessary later?
6. Does a cancel word also stop a running coding task, or only the
   current turn (proposed)?
7. After login, should the agent start awake (as now) or asleep?
8. Location and name: `services/voice/` with the binary `chora-voice`,
   unpackaged until the switch?
