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

The packages of chora install `chora-voice` in `/usr/bin`, its unit
`chora-voice.service` as a user unit, and the quadlets, networks and
`pipewire/echo-cancel.conf` as examples in `/usr/share/chora/voice/`.
With the package, link the quadlets from there and skip the build below:
`ln -s /usr/share/chora/voice/quadlets/* ~/.config/containers/systemd/`.
At the end, start the agent with `systemctl --user enable --now
chora-voice`. The packages leave out `gufo.sh` and `agent-sandbox.sh`.
`agent-sandbox.sh`, the default `sandbox_cmd`, is not part of chora. Put
your own sandbox on the `PATH` of the unit, or set `sandbox_cmd`.

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

Without the package, install the agent and its unit. The build needs
`ar` (binutils) and a C compiler for the kernels of `tract`. `choractl` comes with
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
them at once; after an edit by hand, run `choractl call voice settings.reload`.
For example, the models of another host:

```yaml
asr_url: ws://HOST:8881/v1/realtime?intent=transcription
tts_url: ws://HOST:8882/v1/audio/speech/stream
llm_url: http://HOST:8888/v1
```

### Servers and providers

Each server has its own settings: `asr_url`, `asr_model`, `tts_url`,
`tts_model`, `llm_url` and `llm_model`, an API key and extra headers.
The defaults are the local Gufo servers. Do not put a key into the
settings file. Give the name of an environment variable
(`llm_api_key_env`) or the path of a file (`llm_api_key_file`); `%d` in
the path is the credentials directory of systemd. The agent sends the key
as `Authorization: Bearer KEY`. `llm_headers`, `asr_headers` and
`tts_headers` add headers, for example the app name for OpenRouter.

The LLM must have an OpenAI-compatible `/chat/completions` endpoint.
`reasoning_effort: ""` leaves that key out, for a server that refuses
it. The agent connects to `http://` and `ws://` URLs in plain text, and
to `https://` and `wss://` URLs with TLS (rustls with ring). It trusts
the root certificates of the system, so a CA that you added to the
system store also works.

A local llama.cpp server (`llama-server -m MODEL.gguf --port 8080
--jinja`; `--jinja` enables tool calls):

```yaml
llm_url: http://127.0.0.1:8080/v1
llm_model: local             # llama-server serves one model under any name
```

Ollama:

```yaml
llm_url: http://127.0.0.1:11434/v1
llm_model: qwen3:8b          # a model of `ollama list` with tool support
reasoning_effort: ""
```

OpenRouter:

```yaml
llm_url: https://openrouter.ai/api/v1
llm_model: anthropic/claude-haiku-5.5
llm_api_key_env: OPENROUTER_API_KEY
llm_headers: {HTTP-Referer: "https://github.com/MArpogaus/chora", X-Title: chora}
reasoning_effort: ""
```

OpenAI:

```yaml
llm_url: https://api.openai.com/v1
llm_model: gpt-4.1-mini
llm_api_key_env: OPENAI_API_KEY
reasoning_effort: ""
```

For the systemd unit, keep the key in a credential instead of the
environment: `LoadCredential=llm-key:%h/.config/chora/llm-key` in a
drop-in of `chora-voice.service`, and `llm_api_key_file: "%d/llm-key"`.

A Claude Pro or Max subscription is not an API key, and this agent
cannot use it. For a Claude model here, use an API key: from OpenRouter
as above, or from the Anthropic API through an OpenAI-compatible
endpoint. Coding agents over ACP (`agent_cmd`, such as Claude Code) are
different: they run as their own program and log in with their own
account, so Claude Code can use the subscription there.

### Languages

The ASR finds the language of the speaker (`asr_language: auto`); set a
code such as `de` to fix it. `asr_prompt` tells the ASR the words that it
must know, by default "Chora, Coder"; the agent adds the wake, sleep and
stop words and the names of the projects. An empty `asr_prompt` sends no
prompt. Gufo honors it: without it, "Hallo Chora" came back as "Hello,
Cora".

The agent answers in `reply_language` (`en`), also to a question in
another language, unless you ask for another one. The TTS speaks that
language unless `tts_language` is set. With `reply_language: de`, the
spoken errors, the confirm prompts and the questions of the agents are
German; any other language keeps them English unless `phrases` and
`confirm` set them. The word lists hold English and German words.

### From 0.9.0-rc.1

`model` is now `llm_model`, and `language` is now `asr_language`; an old
key stops the agent with an error. The agent now answers in English: set
`reply_language: de` for German answers and phrases. To keep the old ASR
behavior, set `asr_language: de`. `asr_model` and `tts_model` keep
`gufo`.

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
| `stop`, `cancel`, `never mind`, `stopp`, `abbrechen`, `halt`, `vergiss es` | Stops the current turn: the reply, its sound and its tool call |
| `sleep`, `go to sleep`, `schlaf`, `schlaf jetzt` | The agent sleeps: it hears only its wake words and the key |
| `hey chora`, `listen`, `hallo chora`, `hör zu` | The agent wakes up |
| `yes`, `okay`, `do it`, `ja`, `genau`, `mach das` / `no`, `rather not`, `nein`, `nicht`, `lieber nicht` | The answer to a question before a tool call |
| `cancel the coding task`, `coding-aufgabe abbrechen` | Stops the coding task |

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
sounds the error earcon and says "I did not understand you.", the
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
(`skipped_tools`). `skipped_tools` only hides tools: the agent itself
refuses a call of its own clients `voice`, `voice-hud` and `voice-mcp`,
also through `call_method`. `chora_mcp` is the command of `choractl`, so that the
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
command that takes the task as its last argument. The agent starts it
in `sandbox_cmd` (default `[agent-sandbox.sh]`: a podman container,
read-only, without capabilities), in the directory of the task:

```yaml
coder_cmd: [claude, -p, --output-format, stream-json, --verbose]
```

`unsandboxed: true` starts `coder_cmd` and `agent_cmd` without the
sandbox. Do not set it on your host. The task comes from the LLM, and
so from any voice or text that steers it. Without the sandbox, the
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

## Agents

Agents are long-lived coding agents that speak the Agent Client
Protocol (ACP), one per project, each with a name and a voice of its
own. An empty `agent_cmd` (the default) turns them off. Set the ACP
command of your agent; the voice agent starts it in `sandbox_cmd`, in
the directory of the project:

```yaml
agent_cmd: [opencode, acp]
agent_projects: {chora: ~/Projekte/chora, notes: ~/Notes}
agent_voices: [ryan, vivian, serena, eric]
```

So the command line of an agent is `agent-sandbox.sh opencode acp`,
started in `~/Projekte/chora`. `agent-sandbox.sh` mounts the current
directory into its container, keeps ACP on stdin and stdout without a
tty, and has its own home in `~/.config/sandbox-agent-home`. Configure
the agent there, for example the model of opencode. Note: the version
of `agent-sandbox.sh` of 2026-10-31 drops the arguments after the
command (its option loop never fills `ARGS`), so `acp` and the task of
`coder_cmd` do not reach the container. Its `case` needs a last branch
`*) ARGS+=("$1") ;;`. `{dir}` and
`{project}` in `agent_cmd` are replaced by the project. The projects
are those of `agent_projects` and the directories of
`~/.local/share/chora/inventory`.

Use it like this:

- "Starte einen Agenten für chora mit dem Namen Atlas." The LLM calls
  `agent_start` (it asks first, the `confirm` rule of `agent_start`).
- "Atlas, schreib Tests für den Parser." An utterance that starts with
  the name of an agent goes to that agent at once, without the LLM.
  The LLM can also send work with `agent_say`.
- "Atlas, stopp." cancels the request of Atlas and the requests that
  wait. Requests to a busy agent wait for its turn.
- The answer of an agent is spoken in its voice when you and the voice
  agent are quiet. A long answer speaks its first sentence and
  `phrases.details` ("Details on the screen."); chora-mc shows all
  of it.
- A permission request of an agent is spoken as "Atlas wants to edit
  notes.txt. Allow?". Say yes or no (or "Atlas, yes"), or call
  `voice.confirm` (privileged). No answer within `agent_permission_s`
  (120 s) from when it is spoken is a no.
- "Was macht Atlas?" The LLM calls `agent_status`: state, task, plan,
  question and last words. `agent_stop` ends the agent.

Pick names that the ASR hears well and that are not wake words or
common first words: a name of five letters or more may differ by one
letter, a shorter one must be exact. If the ASR mishears a name, the
utterance goes to the LLM, which can still route it with `agent_say`.

Each agent logs its stderr and all ACP messages (`>` sent, `<`
received) to `~/.local/share/chora/agents/NAME.log`. The agents end
with the voice agent: a stop, `SIGTERM` or `SIGINT` sends `TERM` to
the process group of each agent (`podman run --rm` then removes its
container), and `KILL` after 5 s; the voice agent exits once the
groups ended. They do not survive a restart of the
voice agent; start them again.

chora-mc shows each agent in the status line and in the `agents` scene:
its state, its question, its task, its plan and its last words.
`mc agent.focus {name: atlas}` shows one agent with its transcript.
The method `voice.agent {tool, ...}` runs the agent tools without the
LLM, for example from a key binding:

```bash
choractl call voice agent '{"tool": "agent_start", "project": "chora", "name": "atlas"}'
choractl call voice agent '{"tool": "agent_say", "name": "atlas", "text": "Lies das README."}'
```

## Tests

`cargo test -p chora-voice` runs the unit tests, the replay of the
corpus in `tests/data/corpus` and an end-to-end replay through the whole
agent with fake servers. `chora-voice replay DIR...` replays recordings
of `record_dir`. `tests/corpus.py TTS_URL` writes the corpus again with
the live TTS, and `tests/live.py TTS_URL ASR_URL` checks the servers.

`tests/voice-acp.sh` (in the repository root) runs the fake ACP agent
`tests/fake-acp.sh` through chora-voice and chora-mc in a headless
session: start, work, question, yes, done, stop. `SHOTS=DIR` saves a
screenshot of the agents scene at each step.

`tests/live-gate.sh [TEXT]` runs one voice turn through the whole agent
in the running chora session, against the servers of your settings,
and prints the latency of each stage. The question is a TTS clip, which
a capture command plays into the agent `voice-gate`; the agent `voice`
goes on as it is. `GATE_PLAYBACK=1` plays the reply on the speakers.

```bash
services/voice/tests/live-gate.sh "Welche Fenster sind gerade offen?"
```
