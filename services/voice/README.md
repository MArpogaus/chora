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
Their internal networks keep them off the internet. Any program and any
web page on this machine can still use them. A program can also take a
port while its model is down. It then answers in place of the model, and
its tool calls run with the grants of the agent. A drop-in such as
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
an LLM from the [model table of
Gufo](https://github.com/gufo-org/gufo#models-and-benchmarks). The GPU
needs the groups and the `crun` runtime of the [quickstart of
Gufo](https://github.com/gufo-org/gufo#quickstart). The GGUF must be
under `/var/models`, which the containers mount. Then link and start the
units. Run the steps in the checkout `~/Projekte/chora`, which the unit
expects:

```bash
sudo install -d -o "$USER" /var/models
# The revision that Gufo tests.
uvx --from huggingface_hub hf download Qwen/Qwen3-ASR-1.7B \
    --revision 7278e1e70fe206f11671096ffdd38061171dd6e5 \
    --local-dir /var/models/Qwen/Qwen3-ASR-1.7B
uvx --from huggingface_hub hf download Qwen/Qwen3-TTS-12Hz-1.7B-CustomVoice \
    --local-dir /var/models/Qwen/Qwen3-TTS-12Hz-1.7B-CustomVoice
# An LLM from the model table of Gufo, with its revision. For a GGUF in
# several files, download them all (--include '<folder>/*') and use a
# drop-in instead of the link, as in Services.
uvx --from huggingface_hub hf download <repo> <model>.gguf \
    --revision <revision> --local-dir /var/models/<dir>
ln -s /var/models/<dir>/<model>.gguf /var/models/gufo-llm
# GBs, once; the digest of the image in the quadlets
podman pull ghcr.io/gufo-org/toolboxes/gufo-runtime:latest@sha256:af997f31d5a825ad02fd44cbb6744e71a21df6149c02c5ed3bb0bb843b3f0013
mkdir -p ~/.config/containers/systemd
ln -s "$PWD"/services/voice/systemd/*.{container,network} ~/.config/containers/systemd/
systemctl --user link "$PWD/services/voice/systemd/voice-agent.service"
systemctl --user daemon-reload
systemctl --user start gufo-asr gufo-tts gufo-llm
```

The quadlets pin the image by its digest. For a newer runtime, change
the digest in the three quadlets. The models load for some minutes. A
model is ready when its log, for example `journalctl --user -u gufo-tts`, shows `event=listening`. Then
check the models and the mic:

```bash
services/voice/agent.py roundtrip   # the three models, without a mic
services/voice/agent.py levels      # the mic level
services/voice/agent.py tools       # the connection to chora
```

The agent measures the level of the quiet room for 750 ms at its start,
so keep quiet then. Then start the agent, which runs only in a chora
session:

```bash
systemctl --user enable --now voice-agent
journalctl --user -u voice-agent -f
```

If the log says `turn at level` with an empty `you:` after it, raise
`SPEECH_RATIO` (3). If the agent does not react to your voice, lower it.
After a login, the agent starts the models itself; until they are up, a
turn fails.

The system prompt of the LLM, `SYSTEM`, explains chora and the limits of
the tools. The sections that `CONTEXT_DOCS` names follow it;
`FILE#HEADING` picks one section of a file. The agent sends the prompt
to the LLM once at its start, so that the server keeps it in its cache.
Facts of your machine, such as the installed programs, go into
`~/.config/chora/voice-prompt.md`, which follows them too.

The settings are the environment variables in the globals section of
`agent.py`, such as `ASR_LANGUAGE` (`de`) and `VOICE`. The unit reads
them from `~/.config/chora/voice.env`; quote values with spaces, such as
`CODER_CMD`. A `PATH=` line there replaces the PATH of the unit. After a
change, run `systemctl --user restart voice-agent`. A command in a
terminal reads them from its environment, for example `SPEECH_RATIO=4
services/voice/agent.py`. Before such a test, stop the unit, or both
agents answer. `set -a; . ~/.config/chora/voice.env; set +a` gives the
terminal the settings of the unit. `services/voice/agent.py --help`
lists the commands. The models keep the GPU memory until `systemctl
--user stop voice-agent gufo-asr gufo-tts gufo-llm`.

## Turns

Speech is louder than `SPEECH_RATIO` times the level of the quiet room,
and at least `RMS_SPEECH` (5). The agent follows that level with a
running mean of the quiet mic between turns. The line `speak now` shows
the level of speech. A turn starts when the mic level passes it after
`SETTLE_MS` (300) of quiet. It ends after `SILENCE_MS` (1000) of quiet
or after 30 s. During a turn, half the level of speech still counts as
speech, so soft word ends keep it going. A turn with less than
`MIN_SPEECH_MS` (200) of speech, soft speech included, is a noise, such
as a breath, and the agent drops it.

Under chora, holding `PTT_KEY` (`Super+v`) also records, whatever the
level. The key needs Ctrl, Alt or Super. The turn ends when the modifier
goes up, because chora reports the release with it. After a pause of
`SILENCE_MS`, the agent loses the words until its first sound. To add
words, hold the key during the transcription, or raise `SILENCE_MS`. A
turn by level or by barge-in keeps the last `PREROLL_MS` (1000) of the
mic before its start. A key press after the transcription stops the
reply. Before the first words of the LLM, it also withdraws the request.
After them, the request and the words and tool calls so far stay in the
history.

While the agent speaks, speech above `RMS_BARGE_IN` (twice
`RMS_SPEECH`) and the level
of speech for `BARGE_MS` (300) stops the reply, and a turn starts. While
the agent works without sound, the speech must last 1 s, so that a short
"yes" does not stop it. Dips down to half the level of speech, for at
most 100 ms, do not reset the count, since the echo canceller damps the
voice in bursts.
When that turn has no words, it was a noise: the LLM answers the request
again, or goes on with the cut reply. The push-to-talk key also stops a
reply. The mic also hears the speaker, so use headphones or the echo
cancellation below. After each full reply, the log line `mic peak during
the reply` shows the level that the barge-in level must stay above. When
the reply stops itself, the line `barge-in at level` shows the level of
the echo.

The agent keeps your requests of the last `HISTORY` (20) turns in
`~/.local/state/chora/voice-history.json`. At its next start, they go
into the system prompt as a quoted note. Replies, tool calls and results
stay out: a reply can repeat text from the screen or from a coder, and
node ids describe a scene that may be gone. Delete the file to start a
new conversation.

## Echo cancellation

`pipewire/echo-cancel.conf` loads the echo canceller of PipeWire. It
adds the source `echo-cancel-source`, a mic without the sound of the
speakers. Make it the default source, so that the agent records from it:

```bash
mkdir -p ~/.config/pipewire/pipewire.conf.d
ln -s "$PWD"/services/voice/pipewire/echo-cancel.conf ~/.config/pipewire/pipewire.conf.d/
systemctl --user restart pipewire
pactl set-default-source echo-cancel-source
```

To check it, run `services/voice/agent.py levels` while music plays from
the speakers. The level must stay near the level of the quiet room, and
rise when you speak. Then set `RMS_BARGE_IN` close to the level of
speech, so that you can talk over a reply at a normal level.
`SETTLE_MS=0` keeps the words that you say right after a reply.

## Tools

The LLM gets the tools of `choractl --name voice mcp`, except the event
tools and `capture`. `CHORA_MCP` is the command of `choractl`, so
that the tools run in another place, for example `podman exec -i NAME
choractl` in a container. Its programs then start in the container.
When the LLM reads images and `voice` has the grant `screen.capture`,
`describe_screen` gives it the screenshot too. Gufo says so in `input_modalities` of `/v1/models`, and the log at
the start. A text-only LLM gets the windows and nodes as text only. The
unit finds chora through `CHORA_SOCKET`, which a chora flavor gives the
user manager.

To type and to start programs, the client name `voice` (`CHORA_NAME`)
needs grants. A grant lasts until the session ends, until `choractl
revoke voice`, the revoke binding or the emergency reset:

```bash
choractl grant voice input.inject activation.token
```

Each of these grants alone, and `CODER_CMD`, lets the LLM run commands
as you. A coding task needs no grant, so `choractl revoke` does not stop
it; `code_cancel` does. Without grants, the LLM still closes windows and changes the
outputs, the scene and the plugins, and overwrites the saved views. It
also starts the plugins of installed views. Any voice that the mic hears
can start a turn, also from a video, and also while the screen is
locked. Text on the screen that a tool reads can also steer the LLM. So
can the plugins, which write the descriptions and results of their
tools. The journal keeps each transcript and each tool call, and the
history file your requests. The emergency reset does not
undo output changes, and the reserved space
returns with the next tool call; `wlr-randr` resets the outputs. The
emergency reset and the revoke binding do not stop the agent; after the
reset, it connects again. Only `systemctl --user stop voice-agent` stops
the agent; it and `code_cancel` stop a running coding task.

## Coding tasks

`CODER_CMD` names a coding harness, which gets the task as its last
argument, for example:

```bash
CODER_CMD="claude -p --dangerously-skip-permissions --output-format stream-json --verbose"
```

It adds the tools `code_task`, `code_status`, `code_cancel` and
`list_projects`. A task runs in the background, one at a time:

- `code_task` with a project clones `~/.local/share/chora/inventory/NAME`
  to `~/.local/share/chora/worktrees/NAME-N`, on the branch `voice/N`.
  Without a project, it starts an empty repository `worktrees/N`.
- The harness runs in that clone. Its output goes to the file of the
  clone with `.log` after its name, such as `worktrees/chora-3.log`. The
  agent speaks the result and keeps it in the history.
- The inventory holds the projects that you keep, as git repositories
  or links to them, for example `ln -s ~/Projekte/chora
  ~/.local/share/chora/inventory/chora`. The clone has no remote. A result reaches the branch
  `voice/N` only when the harness commits it; the agent asks for that.
  Take a good result into a project with `git fetch` from its clone.

JSON lines such as the `stream-json` of Claude Code give the steps and
the result; other harnesses give plain lines. Without a sandbox of its
own, the harness of the example changes and deletes files without
asking, also outside the clone, and runs any command as you. The task
and the files that it reads go to the provider of the model.
