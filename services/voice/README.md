# Voice agent

`chora-voice` is the voice agent of chora, one Rust binary:
`mic -> Silero VAD -> Qwen3-ASR -> LLM with tools -> Qwen3-TTS ->
speaker`. The three models run as Gufo containers on the AMD GPU. The
LLM drives chora through `choractl mcp`. The specification, the results
and the open questions are in `docs/voice.md`.

The agent records with `pw-record` and plays with `pw-play` (PipeWire
1.4 or later, for `--raw`), and it needs `choractl` for the tools. The
unit finds them in `~/.cargo/bin`, `~/.local/bin`, `/usr/local/bin` and
`/usr/bin`.

## Services

`systemd/` holds the quadlets of the models and the user unit of the
agent, `chora-voice.service`.

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
under `/var/models`, which the containers mount. Run the steps in the
checkout of chora:

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
systemctl --user daemon-reload
systemctl --user start gufo-asr gufo-tts gufo-llm
```

The quadlets pin the image by its digest. For a newer runtime, change
the digest in the three quadlets. The models load for some minutes. A
model is ready when its log, for example `journalctl --user -u
gufo-tts`, shows `event=listening`.

Then install the agent and its unit. The build needs `ar` (binutils)
and a C compiler for the kernels of `tract`. `choractl` comes with
chora:

```bash
cargo install --locked --path services/voice
systemctl --user link "$PWD/services/voice/systemd/chora-voice.service"
systemctl --user daemon-reload
```

Check the models and the mic before the first start:

```bash
chora-voice roundtrip          # TTS, then its sound to the ASR, with latencies
chora-voice ask "Welche Fenster sind offen?"   # one turn with the LLM and the tools
chora-voice levels             # the mic level and the speech probability
```

`chora-voice --help` lists all commands. Then start the agent, which
runs only in a chora session:

```bash
systemctl --user enable --now chora-voice
journalctl --user -u chora-voice -f
```

After a login, the agent starts the models itself; until they are up, a
turn fails. `systemctl --user stop chora-voice gufo-asr gufo-tts
gufo-llm` frees the GPU memory. Stop the unit before you run
`chora-voice` in a terminal, or both agents answer.

## Settings

The settings are in `~/.config/chora/chora-voice.yaml`. A missing key
keeps its default, so an empty file is valid. `chora-voice
--print-config` prints all keys with their defaults; `docs/voice.md`
(section 9) explains them. `choractl call voice settings.set` changes
them at once; after an edit by hand, run `choractl call voice restart`.
For example, the models of another host:

```yaml
asr_url: ws://HOST:8881/v1/realtime?intent=transcription
tts_url: ws://HOST:8882/v1/audio/speech/stream
llm_url: http://HOST:8888/v1
```

The built-in system prompt explains chora and the limits of the tools.
The context documents (`context_docs`) follow it: `README.md`,
`docs/glossary.md` and sections of `docs/api.md`. The agent sends the
prompt to the LLM at its start, so that the server keeps it in its
cache; a cold cache takes about 40 s. Facts of your machine, such as the
installed programs, go into `~/.config/chora/voice-prompt.md`
(`prompt_file`), which follows them.

## Turns

The agent starts awake and listens all the time. Silero VAD finds
speech: an utterance starts after `onset_ms` (96) of speech and ends
after `silence_ms` (800) of quiet. The push-to-talk key `ptt_key`
(`Super+v`) records while it is down, also when the agent sleeps; it
also stops a reply. `choractl call voice talk '{"down": true}'` does
the same for another client.

The agent knows these words without the LLM, in an utterance of up to 2
s and four words. One wrong letter in a word of four letters or more is
fine, so the ASR's "Hallo Cora" is "Hallo Chora".

| Words | Effect |
|---|---|
| `stopp`, `abbrechen`, `halt`, `vergiss es` | Stops the current turn: the reply, its sound and its tool call |
| `schlaf`, `schlaf jetzt` | The agent sleeps: it hears only its wake words and the key |
| `hör zu`, `hallo chora` | The agent wakes up |
| `ja`, `okay`, `genau`, `mach das` / `nein`, `nicht`, `lieber nicht` | The answer to a question before a tool call |
| `coding-aufgabe abbrechen` | Stops the coding task |

While the agent speaks, you can talk over it (barge-in): the sound
pauses, and your words decide. A stop word ends the reply. Echo of the
agent's own words, or nothing, lets it go on from the cut sentence.
Other words end the reply and start the next one. Speech while the agent
thinks or runs a tool waits in a queue and starts the next turn. A
stopped turn stays in the history with the words that played and the
note "cancelled by the user; result unknown".

Before the tools of the `confirm` rules, such as closing a window or
typing, the agent asks. Say yes or no, or call `choractl call voice
confirm '{"answer": "yes"}'`. No answer within 8 s, or two unclear
answers, is a no.

`choractl call voice state` shows the state: `sleeping`, `listening`,
`recording`, `transcribing`, `thinking`, `tool`, `confirming`,
`speaking` or `working` (listening while a coding task runs). A dot at
the bottom right of the first output shows it too (`indicator`): blue
listens, red records, amber thinks, violet asks, green speaks, teal
works, gray sleeps. The agent emits its state as the event `voice.state`
and each spoken sentence as `voice.say` (D65); `chora-mc` follows
them, so its agents scene and its HUD show it, and
`choractl events voice.*` prints them. The log has one line per stage with
its latency and one summary per reply.

`choractl call voice health` shows each server (`asr`, `tts`, `llm`,
`mcp` and `chora`) as `ok`, `down` or `unknown`, with its last error.
The agent probes the websockets of the ASR and the TTS every 30 s, and
sooner while one is down. When a server fails, you hear it: the ASR
sounds the error earcon and says "Ich habe dich nicht verstanden.", the
LLM says that it does not answer, and the TTS sounds the error earcon;
its text still goes to the HUD. A TTS that is down gets no request for
1 s to 30 s.

The agent keeps your requests of the last `saved_requests` (20) turns in
`~/.local/state/chora/voice-history.json`. At its next start, they go
into the system prompt as a quoted note. Replies, tool calls and results
stay out. Delete the file to start a new conversation.

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

To check it, run `chora-voice levels` while music plays from the
speakers. The level must stay low, and rise when you speak. The
barge-in compares the mic with the level of the agent's own sound: it
learns how much of that sound reaches the mic, from
`barge_in_coupling` (0.1, which assumes the echo canceller). Without
the echo canceller, use headphones, or raise `barge_in_coupling` to the
echo level over the played level.

## Tools

The LLM gets the tools of `choractl --name voice-mcp mcp`, except the
event tools, `capture` and the methods of the agent itself
(`skipped_tools`). `chora_mcp` is the command of `choractl`, so that the
tools run in another place, for example `[podman, exec, -i, NAME,
choractl]` in a container. Its programs then start in the container.
When the LLM reads images and `voice-mcp` has the grant
`screen.capture`, `describe_screen` gives it the screenshot too. The
unit finds chora through `CHORA_SOCKET`, which a chora flavor gives the
user manager.

`choractl mcp` confines its connection: although it uses the main
socket, it has the capability rules of a client socket (D33). Each
power of the LLM beyond the scene needs a grant of the client name
`voice-mcp`. A grant lasts until the session ends, until `choractl
revoke voice-mcp`, the revoke binding or the emergency reset:

| grant | what the LLM can do with it |
|---|---|
| `methods.call` | call the methods of the plugins: `call_method` and the tools `client.method`, for example the focus of chora-tiling |
| `methods.privileged` | with `methods.call`, call privileged methods: `settings.set`, `settings.reload` and methods that run commands, such as `app.open` of chora-mc |
| `activation.token` | start any program with `run_program` |
| `input.inject` | type and click with `inject_input` |
| `screen.capture` | see the screenshot in `describe_screen` |

```bash
choractl grant voice-mcp methods.call
```

`methods.call` alone keeps the plugins at their plain methods. Each of
`methods.privileged`, `activation.token` and `input.inject` lets the
LLM run commands as you: a setting such as the terminal of chora-xp or
a binding with `run:` holds a command, and typing reaches a terminal.
The agent asks for a spoken yes before each `run_program` and each
`inject_input` (the `confirm` rules), not before a privileged method.
Without grants, the LLM still closes windows and changes the outputs
and the scene, and overwrites the saved views. It also starts the
plugins of installed views. Any voice that the mic hears can start a
turn, also from a video, and also while the screen is locked. Text on
the screen that a tool reads can also steer the LLM. So can the plugins,
which write the descriptions and results of their tools. The journal
keeps each transcript and each tool call, and the history file your
requests. The emergency reset does not undo output changes, and the
reserved space returns with the next tool call; `wlr-randr` resets the
outputs. The emergency reset and the revoke binding do not stop the
agent; after the reset, it connects again. Only `systemctl --user stop
chora-voice` stops the agent.

## Coding tasks

An empty `coder_cmd` (the default) turns the coding tools off. With a
harness in `coder_cmd`, the LLM gets the tools `code_task`,
`code_status`, `code_cancel` and `list_projects`. The harness is any
command that takes the task as its last argument. Make it a sandbox
wrapper, such as `agent-sandbox.sh` (a podman container, read-only,
without capabilities), around the coding agent:

```yaml
coder_cmd: [agent-sandbox.sh, claude, -p, --output-format, stream-json, --verbose]
```

Do not give a bare command such as `[claude, -p, ...]`. The task comes
from the LLM, and so from any voice or text that steers it. A bare
harness runs that task as you, with access to all your files and keys.
The clone below does not protect your home directory.

A task runs in the background, one at a time, in a clone of a project
of `~/.local/share/chora/inventory/NAME` (a git repository), in
`~/.local/share/chora/worktrees/NAME-N` on the branch `voice/N`. The
clone has no hardlinks and no `origin`, so a push does not reach the
project. Without a project, the task gets an empty repository. Its
output goes to `worktrees/NAME-N.log`. Lines of stream-json give the
steps and the result; other lines are steps. Before a task starts, the
agent asks (the `confirm` rule of `code_task`).

While the task runs, the state is `working`, and chora-mc shows it as
the agent `coder` with its last step. When it ends, the agent says the
result once you and its reply are done, and keeps it in the history.
`code_status` repeats the last result. "Coding-Aufgabe abbrechen" stops
the task and all processes of its process group.

## Tests

`cargo test -p chora-voice` runs the unit tests, the replay of the
corpus in `tests/data/corpus` and an end-to-end replay through the whole
agent with fake servers. `chora-voice replay DIR...` replays recordings
of `record_dir`. `tests/corpus.py TTS_URL` writes the corpus again with
the live TTS, and `tests/live.py TTS_URL ASR_URL` checks the servers.

`tests/live-gate.sh [TEXT]` runs one voice turn through the whole agent
in the running chora session, against the servers of your settings,
and prints the latency of each stage. The question is a TTS clip, which
a capture command plays into the agent `voice-gate`; the agent `voice`
goes on as it is. `GATE_PLAYBACK=1` plays the reply on the speakers.

```bash
services/voice/tests/live-gate.sh "Welche Fenster sind gerade offen?"
```
