# Voice agent: requirements and planned features (11-11)

Sources: docs/voice.md (spec, sections 1-19), design/split-review-voice.md,
backlog (V1, V2, E1, Track A 0/3, Track B 3), persona round 3.

## Requirements (standing)

R1  One Rust binary `chora-voice`, a chora client on chora-kit, D62 settings.
R2  Always listening with local sleep/wake words, plus push-to-talk; a cancel
    word stops any turn, also while speaking or working.
R3  Silero VAD on tract as the speech gate; PipeWire echo canceller; no own
    ASR/TTS/LLM servers.
R4  The person decides (P5): risky acts take a key or a click, never a spoken
    yes; the voice says so ("I need a key press or a click"); one question,
    one place.
R5  Every task by voice (P2): open/close the chat, ask and stop a helper,
    answer what voice may answer, move windows, the cube, everyday settings.
R6  Plain words in the person's language (P4, P7): English and German, formal
    "Sie", "helper" not "agent"; no silent failure; a state word in each look.
R7  Security: tools run as the confined `voice-mcp`; helpers (ACP) in the
    sandbox; an LLM can never take the keyboard (P11) or answer a question.
R8  Latency per stage logged; replay tests with labelled recordings.
R9  No telemetry; recordings only when `record_dir` is set.
R10 Interchangeable backends (user, 11-11): the reference setup runs local
    models, but ASR, TTS and LLM are each a configurable API endpoint
    (OpenAI-compatible where a standard exists), so any local or remote
    model can replace any other without code changes.

## Status

Done: WP0-WP8, WP10 (ACP helpers per project), the chat field, voice opens
the chat, the need-key phrase for helpers, German words (11-11).
Open from the spec: WP9 (unit, README, package).

## Planned features

Foundation (before the rc, opus this week):
F1  WP9: systemd user unit, README, RPM and Arch packaging of chora-voice.
F2  V2: the need-key phrase also for voice's own key-only questions.
F3  E1: chat epoch unique per voice start.
F4  V1: a spoken command counts as the person's act (By::Voice), distinct
    from an LLM choice; designed with the security view.
F5  Split-review steps 1-5: one `decide`/`may_answer`/`may_call` crate for
    every confirm path (chat, core, ACP permission).

Feature sprint (Fable, next week, after the rc):
F6  Tool registry and tool plugins: sandboxed MCP servers with one grant
    each (`tool_plugins`), split-review steps 6-7.
F7  Layered context (scene, task, conversation) in context.rs; the HUD and
    the prompt follow the look and the task (steps 8-9).
F8  chora-system: Wi-Fi, Bluetooth, battery, brightness, volume, power over
    D-Bus as voice tools with the confirm rules (everyday tasks by voice).
F9  Latency: first sound from about 4.8 s to under 1.5 s (playback buffer,
    silence_ms, a smaller prompt, a warm cache); measured on the reference
    hardware.
F10 Barge-in package and replay metrics on real recordings.
F11 Settings changed by voice, confirmed by a key.
F12 `run_script` tool (sandboxed, confirmed).
F13 Helpers: start with voice from a list, survive a voice restart; offer
    ACP fs/terminal so voice sees each write.
F14 Core event `permission.requested` with an answer method, a timeout and
    a default deny.
F15 Voice HUD in nested sessions.

Research (user, 11-11):
F16 Research the best local models for our use case: LLM (tool calling,
    short turns, German and English, low first-token latency), ASR and TTS;
    a benchmark harness on our replay corpus and a tool-call task set, so
    candidates are compared by numbers on our tasks.
F17 Fine-tuning study: whether a small model (1-8 B) tuned on our tool
    schema and phrases (LoRA, synthetic and recorded dialogues) beats a
    larger general model on latency and accuracy; a plan with data,
    hardware and cost, before any training.

## Open questions for the user

Q-V1 Silero v6 on tract (today) or v5 on ort?
Q-V2 Coder results: a German phrase, or one LLM round to sum up?
Q-V3 Reference hardware for latency and echo: which machine?
