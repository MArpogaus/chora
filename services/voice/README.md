# Voice agent

`Microphone -> Qwen3-ASR -> LLM with tools -> Qwen3-TTS -> speaker`, one
Python file. The three models run as Gufo containers on the AMD GPU. The
LLM drives chora through `choractl mcp`.

## Services

`systemd/` holds the quadlets, the systemd form of `gufo.sh`:

| unit | port | model |
|---|---|---|
| `gufo-asr.container` | 8881 | `/var/models/Qwen/Qwen3-ASR-1.7B` |
| `gufo-tts.container` | 8882 | `/var/models/Qwen/Qwen3-TTS-12Hz-1.7B-CustomVoice` |
| `gufo-llm.container` | 8888 | `/var/models/gufo-llm`, a link to the first shard of your GGUF |

All three serve the name `gufo` and listen on `127.0.0.1` only. Their
internal networks keep them off the internet.

```bash
cd services/voice/systemd
ln -s "$PWD"/*.container "$PWD"/*.network ~/.config/containers/systemd/
systemctl --user link "$PWD/voice-agent.service"
systemctl --user daemon-reload
systemctl --user start gufo-asr gufo-tts gufo-llm
journalctl --user -u gufo-tts -f  # "Uvicorn running" after the load
systemctl --user enable --now voice-agent
```

`/usr/libexec/podman/quadlet -dryrun -user` shows the units that the
quadlets become. Another model goes into a drop-in such as
`~/.config/containers/systemd/gufo-llm.container.d/model.conf`, with a
`[Container]` section, an empty `Exec=` line and a new `Exec=` line.
`gufo.sh` stays for ad hoc runs with extra arguments.

## Agent

```bash
services/voice/agent.py roundtrip   # TTS -> ASR -> LLM -> TTS, no mic
services/voice/agent.py levels      # speak, read your RMS_SPEECH
services/voice/agent.py             # the conversation loop
```

A turn starts when the mic level passes `RMS_SPEECH` after `SETTLE_MS`
of quiet, and ends after `SILENCE_MS` of quiet. Under chora, holding
`PTT_KEY` (`Super+v`) also records, whatever the level, and the turn
ends when Super goes up.

Talking over the reply above `RMS_BARGE_IN` (twice `RMS_SPEECH` by
default) for `BARGE_MS` stops the reply and starts a turn, and so does
the push-to-talk key. The mic also hears the speaker, so headphones or
the echo cancel module of PipeWire keep the reply from stopping itself.

The LLM gets the tools of `choractl --name voice mcp`. The service finds
chora through `CHORA_SOCKET`, which a chora flavor gives the user
manager. `CODER_CMD`, for example `pi -p` or `claude -p`, adds the tool
`delegate`. It runs bigger tasks in the background and speaks the
result. The client name `voice` needs grants for the tools that see or
drive the session:

```bash
choractl grant voice screen.capture input.inject activation.token
```

Settings are environment variables, in `~/.config/gufo/agent.env` for
the service: `ASR_URL`, `TTS_URL`, `LLM_URL`, `MODEL`, `LANGUAGE`, `RMS_SPEECH`,
`SILENCE_MS`, `SETTLE_MS`, `RMS_BARGE_IN`, `BARGE_MS`, `PTT_KEY`,
`CHORA_NAME`, `CODER_CMD` and `SYSTEM`.

## Checks

```bash
services/voice/agent.py tts hallo        # one stage at a time
services/voice/agent.py asr speech.pcm
services/voice/agent.py tools            # the tools of the LLM
```
