# Voice agent

`Microphone -> Qwen3-ASR -> LLM with tools -> Qwen3-TTS -> speaker`, one
Python file. The three models run as Gufo containers on the AMD GPU. The
LLM drives chora through `choractl mcp`. The agent needs `uv`, `arecord`
and `aplay`, and `choractl` for the chora tools.

## Services

`systemd/` holds the quadlets of the models and the user unit of the
agent:

| unit | port | model |
|---|---|---|
| `gufo-asr.container` | 8881 | `/var/models/Qwen/Qwen3-ASR-1.7B` |
| `gufo-tts.container` | 8882 | `/var/models/Qwen/Qwen3-TTS-12Hz-1.7B-CustomVoice` |
| `gufo-llm.container` | 8888 | `/var/models/gufo-llm`, a link to the first shard of your GGUF |
| `voice-agent.service` | | the agent, from the checkout in `~/Projekte/chora` |

All three models serve the name `gufo` and listen on `127.0.0.1` only.
Their internal networks keep them off the internet.

```bash
sudo ln -s /var/models/<dir>/<first shard>.gguf /var/models/gufo-llm
cd services/voice/systemd
mkdir -p ~/.config/containers/systemd
ln -s "$PWD"/*.container "$PWD"/*.network ~/.config/containers/systemd/
systemctl --user link "$PWD/voice-agent.service"
systemctl --user daemon-reload
systemctl --user enable --now voice-agent
```

The agent starts the three models. The first start pulls the image and
loads the models, which takes minutes. Another model goes into a drop-in
such as `~/.config/containers/systemd/gufo-llm.container.d/model.conf`,
with a `[Container]` section and a new `Exec=` line. `gufo.sh` stays for
ad hoc runs with extra arguments.

## Agent

`services/voice/agent.py --help` lists the commands. Start with
`roundtrip`, which checks the three models without a mic, and with
`levels`, which shows the mic level. Set `RMS_SPEECH` above the level of
the quiet room and below the level of your voice.

A turn starts when the mic level passes `RMS_SPEECH` after `SETTLE_MS`
of quiet, and ends after `SILENCE_MS` of quiet. Under chora, holding
`PTT_KEY` (`Super+v`) also records, whatever the level. The turn ends
when Super goes up, because chora reports the release with the modifier.

Talking over the reply above `RMS_BARGE_IN` (twice `RMS_SPEECH` by
default) for `BARGE_MS` stops the reply and starts a turn, and so does
the push-to-talk key. The mic also hears the speaker, so headphones or
the echo cancel module of PipeWire keep the reply from stopping itself.
After each reply, the log line `mic peak during the reply` shows the
level that `RMS_BARGE_IN` must stay above.

The LLM gets the tools of `choractl --name voice mcp`. The service finds
chora through `CHORA_SOCKET`, which a chora flavor gives the user
manager. `CODER_CMD`, for example `pi -p` or `claude -p`, adds the tool
`delegate`. It runs bigger tasks in the background and speaks the
result. The client name `voice` needs grants for the tools that drive
the session. A grant lasts until the session ends, so give it again
after each login:

```bash
choractl grant voice input.inject activation.token
```

`activation.token` lets the LLM start any program as you.

Settings are environment variables, in `~/.config/gufo/agent.env` for
the service: `ASR_URL`, `TTS_URL`, `LLM_URL`, `MODEL`, `VOICE`,
`LANGUAGE`, `RMS_SPEECH`, `SILENCE_MS`, `SETTLE_MS`, `RMS_BARGE_IN`,
`BARGE_MS`, `PTT_KEY`, `CHORA_NAME`, `CODER_CMD` and `SYSTEM`.
