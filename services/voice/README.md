# Voice agent

The voice agent is one Python file:
`Microphone -> Qwen3-ASR -> LLM with tools -> Qwen3-TTS -> speaker`. The
three models run as Gufo containers on the AMD GPU. The LLM drives chora
through `choractl mcp`. The agent needs `uv`, `arecord` and `aplay`, and
`choractl` for the chora tools. The unit finds them, and the program of
`CODER_CMD`, in `~/.local/bin`, `/usr/local/bin` and `/usr/bin`.

## Services

`systemd/` holds the quadlets of the models and the user unit of the
agent, `voice-agent.service`.

| quadlet | port | model |
|---|---|---|
| `gufo-asr.container` | 8881 | `/var/models/Qwen/Qwen3-ASR-1.7B` |
| `gufo-tts.container` | 8882 | `/var/models/Qwen/Qwen3-TTS-12Hz-1.7B-CustomVoice` |
| `gufo-llm.container` | 8888 | `/var/models/gufo-llm`, a link to your GGUF file |

All three models serve the name `gufo` and listen on `127.0.0.1` only.
Their internal networks keep them off the internet. A drop-in such as
`~/.config/containers/systemd/gufo-tts.container.d/model.conf` changes a
model: a `[Container]` section with a copy of the `Exec=` line of the
quadlet and another model path. Then run `systemctl --user
daemon-reload` and restart the unit of the model, such as `gufo-tts`.
For a GGUF in several files, give the path of the first file in such a
drop-in. The server finds the other files by their numbered names, which
a link hides. `gufo.sh` runs one model for a test with extra arguments.
It publishes the port on all interfaces, and it needs the unit of that
model stopped.

## Setup

Download the ASR and TTS models from Hugging Face, and link the GGUF of
an LLM that can call tools. The GGUF must be under `/var/models`, which
the containers mount. Then link and start the units. Run the steps
in the checkout `~/Projekte/chora`, which the unit expects:

```bash
sudo install -d -o "$USER" /var/models
for m in Qwen/Qwen3-ASR-1.7B Qwen/Qwen3-TTS-12Hz-1.7B-CustomVoice; do
    uvx --from huggingface_hub hf download "$m" --local-dir "/var/models/$m"
done
ln -s /var/models/<dir>/<model>.gguf /var/models/gufo-llm
mkdir -p ~/.config/containers/systemd
ln -s "$PWD"/services/voice/systemd/*.{container,network} ~/.config/containers/systemd/
systemctl --user link "$PWD/services/voice/systemd/voice-agent.service"
systemctl --user daemon-reload
systemctl --user start gufo-asr gufo-tts gufo-llm
```

The first start pulls the image and loads the models, which takes
minutes, also after each login. Each model is ready when its log, for
example `journalctl --user -u gufo-tts`, shows `Uvicorn running`; until
then a turn of the agent fails. Then check the models and the mic:

```bash
services/voice/agent.py roundtrip   # the three models, without a mic
services/voice/agent.py levels      # the mic level
services/voice/agent.py tools       # the connection to chora
```

Set `RMS_SPEECH` above the level of the quiet room and below the level
of your voice. Then start the agent, which runs only in a chora
session:

```bash
mkdir -p ~/.config/gufo
echo RMS_SPEECH=<level> >>~/.config/gufo/agent.env
systemctl --user enable --now voice-agent
journalctl --user -u voice-agent -f
```

If the log says `no quiet below RMS_SPEECH`, or `turn at level` with an
empty `you:` after it, the value is too low. If the agent does not
react to your voice, the value is too high.

The settings are the environment variables in the globals section of
`agent.py`, such as `ASR_LANGUAGE` (`de`) and `VOICE`. The unit reads
them from `~/.config/gufo/agent.env`; quote values with spaces, such as
`CODER_CMD`. After a change, run `systemctl --user restart voice-agent`.
A command in a terminal reads them from its environment, for example
`RMS_SPEECH=3000 services/voice/agent.py`. Before such a test, stop the
unit, or both agents answer. `set -a; . ~/.config/gufo/agent.env; set
+a` gives the terminal the settings of the unit. `agent.py --help` lists
the commands. The models keep the GPU memory until `systemctl --user
stop voice-agent gufo-asr gufo-tts gufo-llm`.

## Turns

A turn starts when the mic level passes `RMS_SPEECH` after `SETTLE_MS`
of quiet, and ends after `SILENCE_MS` of quiet or after 30 s. Under
chora, holding `PTT_KEY` (`Super+v`) also records, whatever the level.
The key needs Ctrl, Alt or Super. The turn ends when the modifier goes
up, because chora reports the release with it.

When the mic level stays above `RMS_BARGE_IN` for `BARGE_MS` while the
agent answers, the reply stops and a turn starts. The push-to-talk key
does the same. The mic also hears the speaker, so headphones or the echo
cancel module of PipeWire keep the reply from stopping itself. After
each full reply, the log line `mic peak during the reply` shows the
level that `RMS_BARGE_IN` must stay above. When the reply stops itself,
the line `barge-in at level` shows the level of the echo.

## Tools

The LLM gets the tools of `choractl --name voice mcp`, except the event
tools and `capture`. The unit finds chora through `CHORA_SOCKET`, which
a chora flavor gives the user manager. `CODER_CMD`, for example `claude
-p --permission-mode acceptEdits`, adds the tool `delegate`. It runs
bigger tasks in the background, in the working directory of the agent,
and speaks the result. The unit runs in your home directory. There the
coder of the example changes and deletes files without asking, and runs
what your Claude settings allow. It also sends the task and the files
that it reads to Anthropic. The journal keeps its whole output. A
drop-in with `WorkingDirectory=` gives the coder a smaller directory.

To type and to start programs, the client name `voice` needs grants. A
grant lasts until the session ends or until `choractl revoke voice`:

```bash
choractl grant voice input.inject activation.token
```

Each of these grants alone, and `CODER_CMD`, lets the LLM run commands
as you. `delegate` needs no grant, so `choractl revoke` does not stop
it. `systemctl --user stop voice-agent` stops the agent and a running
coder. Without grants, the LLM still closes windows and changes the
outputs, the scene and the plugins, and overwrites the saved views. It
also writes assets and starts the plugins of installed views. Any voice
that the mic hears can start a turn, also from a video, and also while
the screen is locked. Text on the screen that a tool reads can also
steer the LLM. So can the plugins, which write the descriptions and
results of their tools. The journal keeps each transcript and each tool
call.
