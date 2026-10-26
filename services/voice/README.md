# Voice agent

`Microphone -> Qwen3-ASR -> LLM with tools -> Qwen3-TTS -> speaker`, one
Python file. The three models run as Gufo containers on the AMD GPU. The
LLM drives chora through `choractl mcp`. The agent needs `uv`, `arecord`
and `aplay`, and `choractl` for the chora tools.

## Services

`systemd/` holds the quadlets of the models and the user unit of the
agent, `voice-agent.service`. The unit runs the agent from the checkout
in `~/Projekte/chora`.

| quadlet | port | model |
|---|---|---|
| `gufo-asr.container` | 8881 | `/var/models/Qwen/Qwen3-ASR-1.7B` |
| `gufo-tts.container` | 8882 | `/var/models/Qwen/Qwen3-TTS-12Hz-1.7B-CustomVoice` |
| `gufo-llm.container` | 8888 | `/var/models/gufo-llm`, a link to the first shard of your GGUF |

All three models serve the name `gufo` and listen on `127.0.0.1` only.
Their internal networks keep them off the internet. A drop-in such as
`~/.config/containers/systemd/gufo-tts.container.d/model.conf`, with a
`[Container]` section and a new `Exec=` line, changes the ASR or TTS
model. `gufo.sh` stays for ad hoc runs with extra arguments.

## Setup

From the root of the repository:

```bash
sudo ln -s /var/models/<dir>/<first shard>.gguf /var/models/gufo-llm
mkdir -p ~/.config/containers/systemd
ln -s "$PWD"/services/voice/systemd/*.{container,network} ~/.config/containers/systemd/
systemctl --user link "$PWD/services/voice/systemd/voice-agent.service"
systemctl --user daemon-reload
systemctl --user start gufo-asr gufo-tts gufo-llm
```

The first start pulls the image and loads the models, which takes
minutes. Then check the models and the mic:

```bash
services/voice/agent.py roundtrip   # the three models, without a mic
services/voice/agent.py levels      # the mic level
```

Set `RMS_SPEECH` above the level of the quiet room and below the level
of your voice, in `~/.config/gufo/agent.env`. Then start the agent:

```bash
systemctl --user enable --now voice-agent
journalctl --user -u voice-agent -f
```

After a change to `agent.env`, run `systemctl --user restart voice-agent`.
`services/voice/agent.py --help` lists the other commands. The settings
are the environment variables in the globals section of `agent.py`.

## Turns

A turn starts when the mic level passes `RMS_SPEECH` after `SETTLE_MS`
of quiet, and ends after `SILENCE_MS` of quiet. Under chora, holding
`PTT_KEY` (`Super+v`) also records, whatever the level. The turn ends
when Super goes up, because chora reports the release with the modifier.

Talking over the reply above `RMS_BARGE_IN` (twice `RMS_SPEECH` by
default) for `BARGE_MS` stops the reply and starts a turn, and so does
the push-to-talk key. The mic also hears the speaker, so headphones or
the echo cancel module of PipeWire keep the reply from stopping itself.
After each full reply, the log line `mic peak during the reply` shows
the level that `RMS_BARGE_IN` must stay above.

## Tools

The LLM gets the tools of `choractl --name voice mcp`. The service finds
chora through `CHORA_SOCKET`, which a chora flavor gives the user
manager. `CODER_CMD`, for example `pi -p` or `claude -p`, adds the tool
`delegate`. It runs bigger tasks in the background in your home
directory and speaks the result.

The tools that drive the session need grants for the client name
`voice`. A grant lasts until the session ends:

```bash
choractl grant voice input.inject activation.token
```

These grants and `CODER_CMD` let the LLM type and run programs as you.
Any voice that the mic hears can start a turn, also from a video, and
also while the screen is locked. Text on the screen that a tool reads
can also steer the LLM. The journal keeps each transcript and each tool
call.
