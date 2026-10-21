# Voice agent (M6)

`Microphone -> Qwen3-ASR -> LLM -> Qwen3-TTS -> speaker`, one Python file,
no Pipecat. The three models run as Gufo containers on the AMD GPU.

## Services

`systemd/` holds the quadlets, the systemd form of `gufo.sh`:

| unit | port | model |
|---|---|---|
| `gufo-asr.container` | 8881 | `Qwen/Qwen3-ASR-1.7B` |
| `gufo-tts.container` | 8882 | `Qwen/Qwen3-TTS-12Hz-1.7B-CustomVoice` |
| `gufo-llm.container` | 8888 | your GGUF, first shard |

All three serve the name `gufo` and listen on `127.0.0.1` only.

```bash
ln -s "$PWD/services/voice/systemd"/* ~/.config/containers/systemd/
systemctl --user daemon-reload
systemctl --user edit gufo-llm   # point MODEL at your GGUF
systemctl --user start gufo-asr gufo-tts gufo-llm
journalctl --user -u gufo-tts -f  # "Uvicorn running" after the load
```

`gufo.sh` stays for ad hoc runs with extra arguments, one foreground
container per call.

## Agent

```bash
uv run services/voice/agent.py levels  # speak, read your RMS_SPEECH
uv run services/voice/agent.py         # the conversation loop
```

The loop records a turn when the mic level passes `RMS_SPEECH`, waits
`SILENCE_MS` of quiet, transcribes, answers and speaks. Speaking while
the agent answers interrupts it (`RMS_BARGE_IN`); the part already said
stays in the history, the last six turns are kept. Everything is
environment: `ASR_URL`, `TTS_URL`, `LLM_URL`, `MODEL`, `VOICE`,
`LANGUAGE`, `RMS_SPEECH`, `RMS_BARGE_IN`, `SILENCE_MS`.

`voice-agent.service` keeps the loop alive across logouts of the
terminal; `systemctl --user start voice-agent` once the models are up.

## Checks

```bash
uv run services/voice/agent.py test        # turn cutting, no servers
uv run services/voice/agent.py tts hallo   # one stage at a time
uv run services/voice/agent.py asr speech.pcm
```

The stage that stays silent or errors is the one to look at; the logs
show what the server sends. A WAV file works for `asr` too.
