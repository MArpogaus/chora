#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["aiohttp>=3.9,<4", "mcp>=2.3,<3"]
#
# [tool.uv]
# # A new environment takes no release after this date; raise it on purpose.
# exclude-newer = "2026-10-29T00:00:00Z"
# ///
"""Voice agent: microphone -> ASR -> LLM with tools -> TTS -> speaker.

Without a command, it runs the conversation loop. services/voice/README.md
describes the turns. The settings are the environment variables in the
globals section of agent.py.
"""

# %% imports
import argparse
import array
import asyncio
import base64
import collections
import contextlib
import json
import math
import os
import pathlib
import re
import shlex
import signal
import sys
import time
from collections.abc import AsyncIterator
from dataclasses import dataclass, field

import aiohttp
from mcp import ClientSession, MCPError, StdioServerParameters
from mcp.client.stdio import stdio_client

# %% globals
ENV = os.environ.get
ASR_URL = ENV("ASR_URL", "ws://127.0.0.1:8881/v1/realtime?intent=transcription")
TTS_URL = ENV("TTS_URL", "ws://127.0.0.1:8882/v1/audio/speech/stream")
LLM_URL = ENV("LLM_URL", "http://127.0.0.1:8888/v1")
MODEL = ENV("MODEL", "gufo")  # the served name of all three models
VOICE = ENV("VOICE", "aiden")  # a voice of the TTS server
# Not LANGUAGE, which gettext reads.
ASR_LANGUAGE = ENV("ASR_LANGUAGE", "de")
# Speech is louder than SPEECH_RATIO times the level of the quiet room,
# and at least RMS_SPEECH.
RMS_SPEECH = int(ENV("RMS_SPEECH", "5"))
SPEECH_RATIO = float(ENV("SPEECH_RATIO", "3"))
SILENCE_MS = int(ENV("SILENCE_MS", "1000"))
SETTLE_MS = int(ENV("SETTLE_MS", "300"))  # quiet before a turn may start
# Speech that stops the reply: also at least RMS_BARGE_IN, since the mic
# also hears the speaker, and for BARGE_MS.
RMS_BARGE_IN = int(ENV("RMS_BARGE_IN", str(2 * RMS_SPEECH)))
BARGE_MS = int(ENV("BARGE_MS", "300"))
# While the agent works without sound, only longer speech stops it: a
# sentence, not a "yes" to what it does.
WORKING_BARGE_MS = 1000
MAX_DIP_MS = 100  # a dip of the voice that the count of a barge-in bridges
PTT_KEY = ENV("PTT_KEY", "Super+v")  # chora reports its release with the modifier
CHORA_NAME = ENV("CHORA_NAME", "voice")  # the client name of the agent
# The command of choractl; its tools, also run_program, run where it runs,
# for example "podman exec -i chora-sandbox choractl" for a container.
CHORA_MCP = ENV("CHORA_MCP", "choractl")
# A coding harness, the task as its last argument, for example "claude -p
# --dangerously-skip-permissions --output-format stream-json --verbose".
CODER_CMD = ENV("CODER_CMD", "")
SYSTEM = ENV(
    "SYSTEM",
    """You are the voice assistant of chora, a Wayland compositor on Linux.
You hear the user through speech recognition, so expect misheard words,
and ask when a request makes no sense. Answer in one or two short spoken
sentences, without markdown. Keep the language that the user asked for.

How chora works:
- The core shows a scene of nodes: windows (kind surface), groups, rects,
  text, images and shaders. query_scene and describe_screen show it.
- The window manager is a plugin, a separate program such as chora-tiling
  or chora-xp. It places the windows. list_plugins shows the plugins and
  the methods that each exports; call them with call_method or with their
  own tool, for example tiling.set_layout. Change the layout of windows
  only through these methods. Never attach, remove or move a window
  node: the window manager owns it, and a window you take is gone from
  its layout.
- You cannot stop, restart or swap a plugin. To switch to another window
  manager, such as the XP flavor, the user restarts chora; say so.
- Put your own nodes under one group with an id that starts with voice-,
  under screen:OUTPUT for a HUD, so that one remove takes them away.
- Your nodes stay when the agent restarts. At the start of a talk, look
  for them with query_scene owner:voice, and ask before you remove them.
- Do not take windows with the give method of a window manager to style
  them: they leave its layout, and their style goes when they return.
  You cannot change the look of a window yet; say so.
- A group with input pass or block takes the input from everything under
  it; never put a window under such a group.
- At an output scale above 1, keep shader and effect layers small; after
  you create one, check left_out_of_the_screen in describe_screen.
- apply_commands is one transaction: all commands apply, or none. After
  an error nothing changed, so fix the named command and send the whole
  list again; do not assume that the commands before it applied.
- Each command is an object with one key, such as {"create": {...}} or
  {"animate": {...}}; two changes are two commands.
- A shader node needs the shader property, the path of put_asset. An
  effect goes into effects of a group, rect or image; it is no shader
  node.
- Read the error of a node that belongs to another client: you can change
  only your own nodes.

Where to look:
- describe_screen: the windows, the outputs and the plugins with their
  methods, as text, and a picture of the screen when you read images.
  Without the picture you cannot tell colors, shapes or how a shader
  looks; then never describe more than these facts, and ask the user.
- query_scene: the nodes and their properties.
- list_plugins: the methods of each plugin, with their parameters.
- The error of a tool says why it refused. Read it before you try again.
- The documents after this prompt describe chora: the API, its commands
  and node properties, the configuration and the words of the project.

Limits:
- run_program starts a graphical program. Its output never comes back,
  only whether it started, or the error. It answers when the program
  starts, not when it ends. When it fails, do not start the same program
  again; say what failed.
- Store a shader or a PNG with put_asset; it answers the path for the
  node, and a second put of the name replaces the file. A shader that
  runs too long hangs the whole of chora: use loops with a fixed, small
  number of steps only, at most 64. If a shader does not compile, the
  error names the line; fix that line.
- You cannot read files or the output of commands. Do not write files
  to inspect them.
- Start no program that keeps running in the background to watch or
  change chora, such as a loop or choractl events. It would go on after
  your turn, out of sight. When something has to happen again and again,
  say that you cannot do it.
- Look once, act, then check once. When a task fails twice, stop and say
  what failed.
- Never end a reply with a promise such as "I check that now": a reply
  without a tool call ends your turn, and nothing happens until the user
  speaks again. Call the tool in the same reply, or say that you are done
  and what is left.""",
)
# Documents that follow the system prompt, relative to the checkout and
# separated by commas; FILE#HEADING takes one "## HEADING" section. The
# facts of this machine in ~/.config/chora/voice-prompt.md follow them.
CONTEXT_DOCS = ENV(
    "CONTEXT_DOCS",
    "README.md, docs/glossary.md, docs/api.md#Model, docs/api.md#Nodes, "
    "docs/api.md#Commands, docs/api.md#Window managers, docs/api.md#Selectors, "
    "docs/api.md#Programs, docs/api.md#Input injection, docs/api.md#Plugin methods, "
    "docs/api.md#Plugins, docs/api.md#MCP tools",
)
REPO = pathlib.Path(__file__).resolve().parents[2]
# The projects that coding tasks clone, which the user keeps, and the
# clones in which the tasks work.
DATA = pathlib.Path(ENV("XDG_DATA_HOME", "") or pathlib.Path.home() / ".local/share") / "chora"
INVENTORY, WORKTREES = DATA / "inventory", DATA / "worktrees"
# The conversation without the system prompt, kept for the next start;
# delete the file to start anew.
STATE = pathlib.Path(ENV("XDG_STATE_HOME", "") or pathlib.Path.home() / ".local/state")
HISTORY_FILE = STATE / "chora/voice-history.json"
if CODER_CMD:
    SYSTEM += """

Coding tasks:
- Hand a task that needs code, a program or many steps to code_task. It
  runs in the background in a clone of a project of list_projects, or in
  an empty repository, and you hear its result when it ends.
- Say the task in full: the coder hears nothing else. Ask it to commit
  its work, or the clone keeps it uncommitted. One task runs at a time;
  code_status says how far it is, code_cancel stops it.
- When the user asks about a spoken coding result, call code_status."""

RATE = 24000
FRAME_MS = 20
FRAME_BYTES = RATE * FRAME_MS // 1000 * 2
PCM = ("-q", "-r", str(RATE), "-c", "1", "-f", "S16_LE", "-t", "raw", "-")  # aplay, arecord
# The mic before a turn starts, which the turn keeps.
PREROLL_FRAMES = int(ENV("PREROLL_MS", "1000")) // FRAME_MS
NO_QUIET_MS = 2000  # when the agent says that the room stays loud
CALIBRATE_MS = 750  # the mic at the start, which sets the level of the quiet room
FLOOR_MS = 5000  # the quiet frames between turns that the level follows
# A turn with less speech, also soft, is a noise, such as a breath.
MIN_SPEECH_MS = int(ENV("MIN_SPEECH_MS", "200"))
MAX_TURN_MS = 30000  # a turn that music or a TV keeps loud ends here
HISTORY = int(ENV("HISTORY", "20"))  # turns kept, each with its tool calls
OLD_TOOL_TEXT = 500  # characters of a tool result or call of an earlier turn
TOOL_ROUNDS = 10  # LLM calls per turn while it asks for tools
FIRST_PIECE = 40  # characters before the comma that may end the first piece
SENTENCE_END = re.compile(r"(?<=[.!?:;])\s+|\n+")
# The agent uses the event tools itself, and describe_screen gives the
# picture that capture would.
SKIPPED_TOOLS = {"subscribe_events", "wait_events", "capture"}
GO_ON = "(A noise cut your answer; that was not me. Go on where you stopped.)"
LINE_LIMIT = 16 * 2**20  # a line of the coder, such as a tool result of stream-json
TURN_ERRORS = (aiohttp.ClientError, TimeoutError, KeyError, ValueError, OSError, MCPError)

SPEAKING = asyncio.Lock()  # one voice at a time: replies and coding results


# %% private helpers
def log(*args: object) -> None:
    """Print a line to stderr."""
    print(*args, file=sys.stderr, flush=True)


def rms(frame: bytes) -> float:
    """Return the root mean square of a frame of 16 bit samples."""
    samples = array.array("h", frame)
    return math.sqrt(sum(s * s for s in samples) / len(samples))


async def one(text: str) -> AsyncIterator[str]:
    """Yield one text, as a stream of one sentence."""
    yield text


def function(name: str, description: str, parameters: dict) -> dict:
    """Return an OpenAI function tool."""
    return {
        "type": "function",
        "function": {"name": name, "description": description, "parameters": parameters},
    }


# %% audio
class Mic:
    """The microphone as one continuous stream of 20 ms frames.

    A task reads arecord into a queue, so arecord never waits on a full
    pipe while the agent speaks or thinks. Use it as an async context
    manager. It also keeps the level of the quiet room, from which the
    thresholds of speech follow.
    """

    def __init__(self) -> None:
        self.queue: asyncio.Queue[bytes | Exception] = asyncio.Queue()
        self.floor = 0.0  # the level of the quiet room

    @property
    def speech(self) -> float:
        """The level from which the mic hears speech."""
        return max(RMS_SPEECH, SPEECH_RATIO * self.floor)

    @property
    def hold(self) -> float:
        """The level that keeps a turn going: soft word ends are speech too."""
        return self.speech / 2

    @property
    def barge_in(self) -> float:
        """The level from which speech stops a reply."""
        return max(RMS_BARGE_IN, self.speech)

    def learn(self, level: float) -> None:
        """Move the level of the quiet room toward a quiet frame.

        It falls 25 times faster than it rises, so that a voice during
        the start leaves the level high for a moment only.
        """
        span = FLOOR_MS if level > self.floor else FLOOR_MS / 25
        self.floor += (level - self.floor) * FRAME_MS / span

    async def __aenter__(self) -> "Mic":
        """Start arecord and the reader task."""
        # pw-record sniffs headers and chokes on raw pipes; arecord takes
        # an explicit raw format.
        self.proc = await asyncio.create_subprocess_exec(
            "arecord", *PCM, stdout=asyncio.subprocess.PIPE
        )
        self.reader = asyncio.create_task(self._read())
        # A voice during the start raises the level; the running mean of
        # the quiet frames brings it down again.
        levels = [rms(await self.frame()) for _ in range(CALIBRATE_MS // FRAME_MS)]
        self.floor = sum(levels) / len(levels)
        return self

    async def __aexit__(self, *exc: object) -> None:
        """Stop the reader task and arecord."""
        self.reader.cancel()
        with contextlib.suppress(ProcessLookupError):  # arecord died
            self.proc.kill()
        await self.proc.wait()

    async def _read(self) -> None:
        try:
            while True:
                self.queue.put_nowait(await self.proc.stdout.readexactly(FRAME_BYTES))
        except asyncio.IncompleteReadError as err:
            self.queue.put_nowait(err)  # arecord ended: the next frame() raises

    async def frame(self) -> bytes:
        """Return the next frame."""
        frame = await self.queue.get()
        if isinstance(frame, Exception):
            raise frame
        return frame

    def flush(self) -> None:
        """Drop the frames that came while nobody listened."""
        while not self.queue.empty():
            if isinstance(frame := self.queue.get_nowait(), Exception):
                self.queue.put_nowait(frame)  # the end of arecord stays
                return


# When the sound written to aplay ends, so that talk() knows whether the
# agent is heard.
SOUND = {"end": 0.0}


async def play(pcm_chunks: AsyncIterator[bytes]) -> None:
    """Play raw PCM chunks through aplay.

    Parameters
    ----------
    pcm_chunks : AsyncIterator[bytes]
        16 bit mono samples at RATE.
    """
    # aplay reports each gap between two sentences as an underrun.
    player = await asyncio.create_subprocess_exec(
        "aplay", *PCM, stdin=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.DEVNULL
    )
    end = time.monotonic()  # when the sound written so far ends
    try:
        async for chunk in pcm_chunks:
            player.stdin.write(chunk)
            end = max(end, time.monotonic()) + len(chunk) / (RATE * 2)
            SOUND["end"] = end
            await player.stdin.drain()
        player.stdin.close()
        # aplay on PipeWire may never exit after its input ends: wait for
        # the sound it got, plus a second.
        with contextlib.suppress(TimeoutError):
            await asyncio.wait_for(player.wait(), max(end - time.monotonic() + 1, 1))
    finally:
        # A barge-in, also while the sound plays out, or an aplay that
        # did not end: stop the sound at once.
        if player.returncode is None:
            player.kill()
            await player.wait()
        SOUND["end"] = min(SOUND["end"], time.monotonic())


# %% speech servers
async def tts(http: aiohttp.ClientSession, text: str) -> AsyncIterator[bytes]:
    """Yield the PCM of one text from the TTS websocket.

    Parameters
    ----------
    http : aiohttp.ClientSession
        The session for the requests.
    text : str
        The text; the agent sends one text per websocket.
    """
    async with http.ws_connect(TTS_URL) as ws:
        # The server refuses stream_audio without response_format pcm.
        # Without a language, Gufo speaks English; auto lets the model choose.
        config = {"model": MODEL, "voice": VOICE, "response_format": "pcm", "language": "auto"}
        await ws.send_json({"type": "session.config", **config, "stream_audio": True})
        await ws.send_json({"type": "input.text", "text": text})
        await ws.send_json({"type": "input.done"})
        async for chunk in _tts_audio(ws):
            yield chunk


async def _tts_audio(ws: aiohttp.ClientWebSocketResponse) -> AsyncIterator[bytes]:
    while True:
        msg = await asyncio.wait_for(ws.receive(), 30)
        if msg.type == aiohttp.WSMsgType.BINARY:
            yield msg.data
            continue
        if msg.type != aiohttp.WSMsgType.TEXT:
            return
        # audio.done ends the audio, session.done the session.
        kind = json.loads(msg.data).get("type", "")
        if "error" in kind:
            log("tts:", msg.data[:200])
        if kind == "session.done" or "error" in kind:
            return


async def speak(http: aiohttp.ClientSession, sentences: AsyncIterator[str]) -> None:
    """Speak sentences one after the other through one aplay.

    A task reads the sentences ahead into a queue, so the tool calls and
    the next LLM round run while TTS speaks.

    Parameters
    ----------
    http : aiohttp.ClientSession
        The session for the requests.
    sentences : AsyncIterator[str]
        The sentences, as they come.
    """
    queue: asyncio.Queue[str | None] = asyncio.Queue()

    async def read() -> None:
        try:
            async for sentence in sentences:
                queue.put_nowait(sentence)
        finally:
            queue.put_nowait(None)

    reader = asyncio.create_task(read())

    async def sound() -> AsyncIterator[bytes]:
        while (sentence := await queue.get()) is not None:
            log(f"agent: {sentence}")
            async for chunk in tts(http, sentence):
                yield chunk
        await reader  # its errors belong to the reply

    try:
        async with SPEAKING:
            await play(sound())
    finally:
        reader.cancel()
        await asyncio.wait([reader])  # answer() writes the history first


async def transcribe(http: aiohttp.ClientSession, pcm: bytes) -> str:
    """Return the transcript of raw PCM from the ASR websocket.

    Parameters
    ----------
    http : aiohttp.ClientSession
        The session for the requests.
    pcm : bytes
        16 bit mono samples at RATE.

    Returns
    -------
    str
        The transcript, empty when the ASR heard nothing or sent an error.
    """
    async with http.ws_connect(ASR_URL) as ws:
        audio = {
            "format": {"type": "audio/pcm", "rate": RATE},
            "transcription": {"model": MODEL, "language": ASR_LANGUAGE},
            "turn_detection": None,
        }
        session = {"type": "transcription", "audio": {"input": audio}}
        await ws.send_json({"type": "session.update", "session": session})
        # Pieces of a second: Gufo closes a socket with 128 waiting messages.
        for i in range(0, len(pcm), RATE * 2):
            chunk = base64.b64encode(pcm[i : i + RATE * 2]).decode()
            await ws.send_json({"type": "input_audio_buffer.append", "audio": chunk})
        await ws.send_json({"type": "input_audio_buffer.commit"})
        while True:
            msg = await asyncio.wait_for(ws.receive(), 30)
            if msg.type != aiohttp.WSMsgType.TEXT:
                return ""
            event = json.loads(msg.data)
            kind = event.get("type", "")
            if kind.endswith("transcription.completed"):
                return event.get("transcript", "").strip()
            if "error" in kind or kind.endswith("failed"):
                log("asr:", msg.data[:200])
                return ""


# %% llm and tools
class Tools:
    """The tools of the LLM: those of `choractl mcp` and of the coder.

    Parameters
    ----------
    session : ClientSession | None
        The MCP session of chora; None gives no chora tools.
    http : aiohttp.ClientSession
        The session for the LLM and for speaking the results of the coder.
    """

    def __init__(self, session: ClientSession | None, http: aiohttp.ClientSession) -> None:
        self.session = session
        self.http = http
        self.specs: list[dict] = []
        self.names: dict[str, str] = {}  # OpenAI function name -> MCP tool name
        self.vision = False  # whether the LLM reads the picture of describe_screen
        self.coder = Coder(http) if CODER_CMD else None

    async def load(self) -> None:
        """Ask chora for its tools, add those of the coder, and ask the LLM for images."""
        with contextlib.suppress(aiohttp.ClientError, ValueError, TimeoutError):
            async with self.http.get(f"{LLM_URL}/models") as r:
                models = (await r.json()).get("data") or [{}]
            modalities = models[0].get("architecture", {}).get("input_modalities", [])
            self.vision = "image" in modalities
        log(f"the LLM {'sees' if self.vision else 'does not see'} the pictures of describe_screen")
        if self.session:
            for tool in (await self.session.list_tools()).tools:
                if tool.name in SKIPPED_TOOLS:
                    continue
                # OpenAI function names take no dots, plugin methods have one.
                name = tool.name.replace(".", "__")
                self.names[name] = tool.name
                self.specs.append(function(name, tool.description, tool.input_schema))
        if self.coder:
            self.specs += Coder.SPECS

    async def call(self, name: str, arguments: dict) -> str | list[dict]:
        """Run a tool and return its content for the LLM: text, and images if it sees them."""
        if self.coder and name in self.coder.TOOLS:
            return self.coder.call(name, arguments)
        if name not in self.names:
            return f"no tool {name}"
        result = await self.session.call_tool(self.names[name], arguments)
        text = "\n".join(c.text for c in result.content if c.type == "text")
        text = ("error: " if result.is_error else "") + (text or "done")
        images = [
            {"type": "image_url", "image_url": {"url": f"data:{c.mime_type};base64,{c.data}"}}
            for c in result.content
            if c.type == "image" and self.vision
        ]
        return [{"type": "text", "text": text}, *images] if images else text


class Coder:
    """Coding tasks of CODER_CMD in clones of the projects in INVENTORY.

    One task runs at a time, in the background, in WORKTREES/NAME-N on the
    branch voice/N, or in WORKTREES/N without a project. Its output goes to
    the file of the clone with .log after its name, outside the repository. A line of JSON,
    as the stream-json of Claude Code, gives the steps and the result;
    plain lines count as steps.
    """

    TOOLS = ("code_task", "code_status", "code_cancel", "list_projects")
    SPECS = [  # noqa: RUF012 - one list for all instances
        function(
            "code_task",
            "Start a coding task in the background, in a clone of a project, or in an empty "
            "repository without one. Its result is spoken when it ends.",
            {
                "type": "object",
                "required": ["task"],
                "properties": {
                    "task": {"type": "string", "description": "the task in full"},
                    "project": {"type": "string", "description": "a name of list_projects"},
                },
            },
        ),
        function(
            "code_status",
            "Whether a coding task runs, for how long, and its last step; else the last result.",
            {},
        ),
        function("code_cancel", "Stop the running coding task.", {}),
        function("list_projects", "The projects that code_task can clone.", {}),
    ]

    def __init__(self, http: aiohttp.ClientSession) -> None:
        self.http = http
        self.job: asyncio.Task | None = None  # asyncio keeps only weak references
        self.proc: asyncio.subprocess.Process | None = None
        self.workdir: pathlib.Path | None = None
        self.started = 0.0
        self.step = ""
        self.last = ""  # the spoken result of the last task

    def call(self, name: str, arguments: dict) -> str:
        """Run one of TOOLS and return its answer for the LLM."""
        if name == "code_task":
            return self.start(arguments["task"], arguments.get("project"))
        if name == "list_projects":
            return ", ".join(sorted(p.name for p in INVENTORY.glob("*") if p.is_dir())) or (
                f"no projects in {INVENTORY}"
            )
        running = self.job is not None and not self.job.done()
        if name == "code_cancel" and running:
            self.job.cancel()
            return "the coding task stops"
        if not running:
            return "no coding task runs" + (f"; the last one: {self.last}" if self.last else "")
        minutes = (time.monotonic() - self.started) / 60
        return f"it runs for {minutes:.0f} min in {self.workdir}; last step: {self.step}"

    def start(self, task: str, project: str | None) -> str:
        """Start a task in a new clone of `project`, or in an empty repository."""
        if self.job and not self.job.done():
            return "a coding task runs already; ask for its status or cancel it"
        if project and project not in {p.name for p in INVENTORY.glob("*") if p.is_dir()}:
            return f"no project {project}; list_projects names them"
        if task.lstrip().startswith("-"):  # it would be an option of the harness
            return "a task must not start with -"
        WORKTREES.mkdir(parents=True, exist_ok=True)
        number = 1 + max(
            (
                int(p.name.rsplit("-", 1)[-1])
                for p in WORKTREES.glob("*")
                if p.name.rsplit("-", 1)[-1].isdigit()
            ),
            default=0,
        )
        self.workdir = WORKTREES / (f"{project}-{number}" if project else str(number))
        self.job = asyncio.create_task(self._run(task, project, number), name="coding task")
        self.job.add_done_callback(report)
        return f"the coding task started in {self.workdir}"

    async def _run(self, task: str, project: str | None, number: int) -> None:
        log(f"coding task in {self.workdir}: {task}")
        self.started, self.step, result, state = time.monotonic(), "starting", "", "failed"
        try:
            if project:
                source, clone = str(INVENTORY / project), str(self.workdir)
                await _git("clone", "--quiet", "--no-hardlinks", source, clone)
                await _git("-C", clone, "switch", "--quiet", "-c", f"voice/{number}")
                # No remote: a push from the clone must not reach the project.
                await _git("-C", clone, "remote", "remove", "origin")
            else:
                await _git("init", "--quiet", "-b", f"voice/{number}", str(self.workdir))
            # Its own process group, so that a cancel ends what it started too.
            self.proc = await asyncio.create_subprocess_exec(
                *shlex.split(CODER_CMD),
                task,
                cwd=self.workdir,
                start_new_session=True,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.STDOUT,
                limit=LINE_LIMIT,
            )
            with (self.workdir.parent / f"{self.workdir.name}.log").open("wb") as out:
                async for line in self.proc.stdout:
                    out.write(line)
                    step, result = _step(line.decode(errors="replace"), result)
                    self.step = step or self.step
            state = "is done" if await self.proc.wait() == 0 else "failed"
        except asyncio.CancelledError:
            state = "was cancelled"
        except (OSError, ValueError) as err:  # ValueError: a line over LINE_LIMIT
            result = str(err)
        finally:
            # A coder that ends here, or ignores the signal, must not run on
            # next to the next task.
            if self.proc and self.proc.returncode is None:
                os.killpg(self.proc.pid, signal.SIGTERM)
                try:
                    await asyncio.wait_for(self.proc.wait(), 5)
                except TimeoutError:
                    os.killpg(self.proc.pid, signal.SIGKILL)
                    await self.proc.wait()
        self.last = f"The coding task in {self.workdir.name} {state}. {(result or self.step)[:400]}"
        await speak(self.http, one(self.last))


async def _git(*args: str) -> None:
    """Run git and raise its error."""
    proc = await asyncio.create_subprocess_exec(
        "git", *args, stdout=asyncio.subprocess.DEVNULL, stderr=asyncio.subprocess.PIPE
    )
    _, err = await proc.communicate()
    if proc.returncode:
        raise OSError(f"git {args[0]}: {err.decode(errors='replace').strip()}")


def _step(line: str, result: str) -> tuple[str, str]:
    """Return the step that a line of coder output shows, and the result so far.

    A JSON line of stream-json shows a tool call or a text, or the result;
    any other line is a step as it is.
    """
    try:
        event = json.loads(line)
    except ValueError:
        return line.strip()[:200], result
    if not isinstance(event, dict):
        return line.strip()[:200], result
    if event.get("type") == "result":
        return "", str(event.get("result") or result)
    for part in (event.get("message") or {}).get("content") or []:
        if isinstance(part, dict) and part.get("type") == "tool_use":
            target = next(iter((part.get("input") or {}).values()), "")
            return f"{part.get('name')} {target}"[:200], result
        if isinstance(part, dict) and part.get("type") == "text":
            return part.get("text", "")[:200], result
    return "", result


@dataclass
class Reply:
    """One streamed answer of the LLM: its words and its tool calls."""

    text: str = ""
    calls: dict[int, dict] = field(default_factory=dict)
    results: list[dict] = field(default_factory=list)  # tool messages, in call order

    def add(self, delta: dict) -> str:
        """Add one delta of the stream and return its words."""
        # A tool call comes in parts: the id and name first, then the
        # arguments piece by piece.
        for part in delta.get("tool_calls") or []:
            empty = {"id": "", "name": "", "args": ""}
            call = self.calls.setdefault(part.get("index", 0), empty)
            fn = part.get("function") or {}
            call["id"] = part.get("id") or call["id"]
            call["name"] += fn.get("name") or ""
            call["args"] += fn.get("arguments") or ""
        words = delta.get("content") or ""
        self.text += words
        return words

    async def run_calls(self, tools: Tools) -> None:
        """Run the tool calls one after the other.

        chora still finishes a call that a barge-in cuts, so the call stays
        with an unknown result.
        """
        for call in self.calls.values():
            content = "cut off by the user; the result is unknown"
            try:
                content = await _run_call(tools, call)
            finally:
                self.results.append(
                    {"role": "tool", "tool_call_id": call["id"], "content": content}
                )

    def messages(self) -> list[dict]:
        """Return the messages for the history: words, calls that ran, results.

        A failed or cut round keeps its words and the calls that ran, so no
        call is left without its result.
        """
        answered = {r["tool_call_id"] for r in self.results}
        calls = [c for c in self.calls.values() if c["id"] in answered]
        message: dict = {"role": "assistant", "content": self.text}
        if calls:
            message["tool_calls"] = [
                {
                    "id": c["id"],
                    "type": "function",
                    "function": {"name": c["name"], "arguments": c["args"]},
                }
                for c in calls
            ]
        return ([message] if self.text or calls else []) + self.results


async def _check(r: aiohttp.ClientResponse) -> None:
    """Raise with the answer of the server, which says why it refused."""
    if not r.ok:
        raise ValueError(f"llm: {r.status} {(await r.text())[:500]}")


async def _deltas(http: aiohttp.ClientSession, body: dict) -> AsyncIterator[dict]:
    finish, tokens = None, None
    async with http.post(f"{LLM_URL}/chat/completions", json=body) as r:
        await _check(r)
        async for raw in r.content:
            line = raw.decode().strip()
            if line.startswith("data:") and line != "data: [DONE]":
                event = json.loads(line[5:])
                if "error" in event:
                    raise ValueError(f"llm: {event['error']}")
                tokens = (event.get("usage") or {}).get("prompt_tokens", tokens)
                choice = (event.get("choices") or [{}])[0]
                finish = choice.get("finish_reason") or finish
                yield choice.get("delta") or {}
    # Gufo stops a reply with length where the context ends.
    log(f"llm: {finish}, prompt of {tokens} tokens")


def text_of(content: str | list[dict]) -> str:
    """Return the text of a message content, without its images."""
    if isinstance(content, str):
        return content
    return " ".join(p.get("text", "[picture]") for p in content)


async def _run_call(tools: Tools, call: dict) -> str | list[dict]:
    try:
        arguments = json.loads(call["args"] or "{}")
        log(f"tool {call['name']} {arguments}")
        result = await tools.call(call["name"], arguments)
    except (*TURN_ERRORS, TypeError) as err:  # the LLM reads the error and goes on
        result = f"error: {err}"
    log(f"  -> {' '.join(text_of(result).split())[:300]}")
    return result


async def answer(
    http: aiohttp.ClientSession, tools: Tools, history: list[dict]
) -> AsyncIterator[str]:
    """Yield the words of the LLM as they come, running its tool calls.

    Parameters
    ----------
    http : aiohttp.ClientSession
        The session for the requests.
    tools : Tools
        The tools the LLM may call.
    history : list[dict]
        The conversation; the answer and the tool results join it. A reply
        that a barge-in cuts short keeps the words that the LLM wrote.
    """
    for _ in range(TOOL_ROUNDS):
        # Gufo thinks by default, which would keep each reply silent for seconds.
        body: dict = {
            "model": MODEL,
            "messages": history,
            "stream": True,
            "stream_options": {"include_usage": True},
            "reasoning_effort": "none",
        }
        if tools.specs:
            body["tools"] = tools.specs
        reply = Reply()
        try:
            async for delta in _deltas(http, body):
                if words := reply.add(delta):
                    yield words
            if reply.text and reply.calls:
                yield "\n"  # so the words before the tools are spoken while they run
            await reply.run_calls(tools)
        finally:
            history += reply.messages()
        if not reply.calls:
            if not reply.text:
                log("the LLM gave no words and no tool calls")
                if history[-1]["role"] == "user":  # no answer, so no merge later
                    history.pop()
            return
    yield "I stopped after too many tool calls."


async def sentences(words: AsyncIterator[str]) -> AsyncIterator[str]:
    """Group streamed words into sentences.

    The agent sends one sentence per TTS websocket, so TTS starts on the
    first sentence while the LLM still writes. The first sentence of a
    reply may end at a comma, so the first sound comes sooner.

    Parameters
    ----------
    words : AsyncIterator[str]
        The words of the LLM, as they come.
    """
    buffer, first = "", True
    async for word in words:
        buffer += word
        *done, buffer = SENTENCE_END.split(buffer)
        if first and not done and (comma := buffer.find(", ", FIRST_PIECE)) > 0:
            done, buffer = [buffer[: comma + 1]], buffer[comma + 2 :]
        for sentence in done:
            if sentence.strip():
                first = False
                yield sentence.strip()
    if buffer.strip():
        yield buffer.strip()


# %% listening
async def push_to_talk(session: ClientSession, down: asyncio.Event) -> None:
    """Keep `down` set while the push-to-talk key is held in chora.

    chora ends the subscription when it restarts; then this subscribes
    again until chora is back.
    """
    args = {"events": ["input.key"], "keys": [PTT_KEY]}
    first = True
    while True:
        result = await session.call_tool("subscribe_events", args)
        if result.is_error and first:
            log(f"no push-to-talk: {result.content[0].text}")
            return
        if not result.is_error:  # else chora is not back yet
            first = False
            try:
                await _follow_key(session, down)
            finally:
                down.clear()  # a key that nobody follows is up
        await asyncio.sleep(1)


async def _follow_key(session: ClientSession, down: asyncio.Event) -> None:
    # The subscription grabs only PTT_KEY, and chora sends input.key only
    # to the grab, so every event is the key.
    while True:
        result = await session.call_tool("wait_events", {"timeout_ms": 30000})
        batch = json.loads(result.content[0].text)
        for event in batch.get("events", []):
            if event.get("state") == "pressed":
                down.set()
            elif event.get("state") == "released":
                down.clear()
        if batch.get("ended"):
            log(f"push-to-talk ended: {batch['ended']}")
            return


async def settle(mic: Mic, ptt: asyncio.Event) -> None:
    """Wait for SETTLE_MS of quiet, or for the push-to-talk key.

    The frames that came while the agent was busy go first, unless the
    push-to-talk key holds them. The quiet keeps the loud start of the mic
    and the end of a reply out.
    """
    # A held key keeps what it recorded, but not a spoken coding result.
    echo = SPEAKING.locked()
    async with SPEAKING:
        if echo or not ptt.is_set():
            mic.flush()
    log("listening")
    quiet, waited = 0, 0  # in ms
    while quiet < SETTLE_MS and not ptt.is_set():
        level = rms(await mic.frame())
        quiet = 0 if level >= mic.speech else quiet + FRAME_MS
        waited += FRAME_MS
        if waited == NO_QUIET_MS:
            log(f"no quiet below {mic.speech:.0f}, level {level:.0f}")
    log(f"speak now, speech from level {mic.speech:.0f}")


async def record(mic: Mic, ptt: asyncio.Event, preroll: list[bytes]) -> bytes:
    """Record one turn: by level, or while the push-to-talk key is held.

    Parameters
    ----------
    mic : Mic
        The microphone.
    ptt : asyncio.Event
        Set while the push-to-talk key is held.
    preroll : list[bytes]
        The frames of a barge-in; the turn goes on from them without a
        gap. Empty for a new turn, which waits for quiet first.

    Returns
    -------
    bytes
        The PCM of the turn.
    """
    if not preroll:
        await settle(mic, ptt)
    frames, quiet, pushed = list(preroll), 0, False  # quiet in ms
    voiced = MIN_SPEECH_MS if preroll else 0  # a barge-in was speech already
    # The start of a word is quieter than speech; the turn keeps it.
    before: collections.deque[bytes] = collections.deque(maxlen=PREROLL_FRAMES)
    while True:
        frame = await mic.frame()
        if SPEAKING.locked():  # a coding result: wait for its end and for quiet
            return await record(mic, ptt, [])
        level, down = rms(frame), ptt.is_set()
        loud = level >= mic.speech
        pushed = pushed or down
        if not (frames or loud or down):
            before.append(frame)
            mic.learn(level)
            continue
        if not frames:
            log("turn by key" if down else f"turn at level {level:.0f}")
            frames = [] if down else [*before]  # the key marks the start itself
        frames.append(frame)
        if down:
            continue
        if pushed:  # the key went up
            return b"".join(frames)
        voiced += FRAME_MS if level >= mic.hold else 0
        quiet = 0 if level >= mic.hold else quiet + FRAME_MS
        if quiet >= SILENCE_MS and voiced < MIN_SPEECH_MS:
            log(f"noise of {voiced} ms, no turn")
            frames, quiet, voiced = [], 0, 0
            before.clear()
            continue
        if quiet >= SILENCE_MS or len(frames) * FRAME_MS >= MAX_TURN_MS:
            return b"".join(frames)


async def talk(
    http: aiohttp.ClientSession, mic: Mic, words: AsyncIterator[str], ptt: asyncio.Event
) -> list[bytes] | None:
    """Speak the streamed words while the mic listens for a barge-in.

    Parameters
    ----------
    http : aiohttp.ClientSession
        The session for the requests.
    mic : Mic
        The microphone.
    words : AsyncIterator[str]
        The words of the LLM, as they come.
    ptt : asyncio.Event
        Set while the push-to-talk key is held.

    Returns
    -------
    list[bytes] | None
        The last PREROLL_FRAMES frames when the user talked over the reply,
        which stopped it; None after a stop by key; else an empty list,
        also when the key went down during the ASR, so the user goes on.
    """
    if ptt.is_set():  # pressed during the ASR: no reply, the turns merge
        return []
    reply = asyncio.create_task(speak(http, sentences(words)))
    recent: collections.deque[bytes] = collections.deque(maxlen=PREROLL_FRAMES)
    loud, dip, peak = 0, 0, 0.0  # loud and dip in ms
    mic.flush()  # the ASR time is not a barge-in
    while not reply.done():  # the mic gives a frame every 20 ms
        recent.append(await mic.frame())
        level = rms(recent[-1])
        peak = max(peak, level)
        # The echo canceller damps the voice in bursts: a short dip down to
        # half the level of speech keeps the count, quiet or a long dip,
        # such as between two echo peaks, resets it.
        if level >= mic.barge_in:
            loud, dip = loud + FRAME_MS, 0
        else:
            dip += FRAME_MS
            if level < mic.hold or dip > MAX_DIP_MS:
                loud = 0
        by_key = ptt.is_set()
        heard = time.monotonic() < SOUND["end"] + 0.5  # and its echo
        if loud >= (BARGE_MS if heard else WORKING_BARGE_MS) or by_key:
            log("barge-in by key" if by_key else f"barge-in at level {level:.0f}")
            reply.cancel()
            with contextlib.suppress(asyncio.CancelledError):
                await reply
            # By key, the recent frames hold the reply, not the user.
            return None if by_key else list(recent)
    log(f"mic peak during the reply: {peak:.0f}, barge-in at {mic.barge_in:.0f}")
    await reply  # its errors belong to the turn
    return []


# %% commands
async def chora(stack: contextlib.AsyncExitStack) -> ClientSession | None:
    """Return the MCP session of `choractl mcp`, or None without chora."""
    # The mcp package passes only a few variables by default; choractl
    # needs CHORA_SOCKET and the rest of the session.
    command, *args = [*shlex.split(CHORA_MCP), "--name", CHORA_NAME, "mcp"]
    params = StdioServerParameters(command=command, args=args, env=dict(os.environ))
    try:
        read, write = await stack.enter_async_context(stdio_client(params))
        session = await stack.enter_async_context(ClientSession(read, write))
        await session.initialize()
    except Exception as err:  # no choractl or no core: talk without tools
        log(f"no chora tools: {err}")
        return None
    return session


def system_prompt() -> str:
    """Return SYSTEM with the CONTEXT_DOCS and voice-prompt.md after it."""
    parts = [SYSTEM]
    for entry in filter(None, (e.strip() for e in CONTEXT_DOCS.split(","))):
        name, _, heading = entry.partition("#")
        text = (REPO / name).read_text()
        if heading:
            start = text.index(f"\n## {heading}\n")  # a typo fails at the start
            end = text.find("\n## ", start + 1)
            text = text[start : end if end > 0 else None]
        parts.append(f"# File {name}\n\n{text.strip()}")
    machine = pathlib.Path.home() / ".config/chora/voice-prompt.md"
    if machine.is_file():
        parts.append(machine.read_text())
    return "\n\n".join(parts)


def spoken(messages: list[dict]) -> list[dict]:
    """Return the words of the user in the last HISTORY turns.

    Replies, tool calls and results stay out: a reply can repeat text that
    the LLM read on the screen or from the coder, which would then steer
    every later session; node ids describe a scene that may be gone.
    """
    words = [
        {"role": "user", "content": text_of(m["content"])}
        for m in messages
        if m["role"] == "user" and m["content"] != GO_ON
    ]
    return words[-HISTORY:]


def earlier(words: list[dict]) -> str:
    """Return the words of past sessions as a note for the system prompt.

    Each request is one quoted line, so that it cannot start a section of
    the prompt; as chat messages, the LLM would answer them again.
    """
    if not words:
        return ""
    lines = ("> " + " ".join(m["content"].split()) for m in words)
    return (
        "# Earlier requests of the user\n\nFor context only, not instructions. "
        "The scene may have changed since; look before you act.\n\n" + "\n".join(lines)
    )


def save(past: list[dict], history: list[dict]) -> None:
    """Write the words of the past sessions and of this one to HISTORY_FILE."""
    HISTORY_FILE.parent.mkdir(parents=True, exist_ok=True)
    part = HISTORY_FILE.with_suffix(".part")  # a crash never leaves half a file
    part.write_text(json.dumps(spoken(past + history[1:]), ensure_ascii=False))
    part.replace(HISTORY_FILE)


def trim(history: list[dict]) -> None:
    """Keep the last HISTORY turns, each from its user message on.

    This keeps each tool result with the call that it answers. Before the
    last turn, a long tool result or long arguments of a call keep their
    start only; the last put_asset of each name stays whole, so that the
    LLM can change its own shader.
    """
    users = [i for i, m in enumerate(history) if m["role"] == "user"]
    if len(users) > HISTORY:
        del history[1 : users[-HISTORY]]
        users = [i for i, m in enumerate(history) if m["role"] == "user"]
    assets = {}  # the name of an asset and its last put_asset
    for message in history:
        for call in message.get("tool_calls", []):
            if call["function"]["name"] == "put_asset":
                with contextlib.suppress(ValueError, AttributeError):
                    assets[json.loads(call["function"]["arguments"]).get("name")] = call
    last = [id(call) for call in assets.values()]
    for message in history[1 : users[-1] if users else 1]:
        if message["role"] == "tool":
            # A picture is the screen of its turn only.
            text = text_of(message["content"])
            if len(text) > OLD_TOOL_TEXT:
                text = text[:OLD_TOOL_TEXT] + " [shortened]"
            message["content"] = text
        for call in message.get("tool_calls", []):
            if len(call["function"]["arguments"]) > OLD_TOOL_TEXT and id(call) not in last:
                # Arguments stay JSON, which the chat template of the LLM reads.
                call["function"]["arguments"] = json.dumps({"shortened": True})


async def turn(
    mic: Mic,
    tools: Tools,
    history: list[dict],
    ptt: asyncio.Event,
    preroll: list[bytes],
) -> list[bytes]:
    """Record one turn, transcribe it and speak the answer.

    A request without an answer, after a cut or a key held during the ASR,
    merges with the next words. A stop by key withdraws it while the LLM
    has written no words; after them, the words and tool calls so far stay
    in the history. A barge-in without words was a noise: the LLM answers
    the request again, or goes on with the reply that the noise cut.

    Returns
    -------
    list[bytes]
        The frames of a barge-in, which start the next turn.
    """
    barged, http = bool(preroll), tools.http
    pcm = await record(mic, ptt, preroll)
    # Gufo refuses less than 100 ms, such as the rest of a key tap.
    said = await transcribe(http, pcm) if len(pcm) >= RATE * 2 // 10 else ""
    log(f"you: {said}")
    pending = history[-1]["role"] == "user"
    if said and pending and history[-1]["content"] == GO_ON:
        history[-1]["content"] = said  # the user speaks after all
    elif said and pending:
        history[-1]["content"] += " " + said
    elif said:
        history.append({"role": "user", "content": said})
    elif barged and not pending:
        log("a noise cut the reply; the LLM goes on")
        history.append({"role": "user", "content": GO_ON})
    preroll = []
    # A held key leaves its request to answer, and a noise the cut reply.
    if said or pending or barged:
        preroll = await talk(http, mic, answer(http, tools, history), ptt)
    if preroll is None:  # a stop by key
        if history[-1]["role"] == "user":
            history.pop()
        return []
    return preroll


async def warm(http: aiohttp.ClientSession, tools: Tools, system: list[dict]) -> None:
    """Let the LLM read the system prompt and the tools before the first turn.

    The server keeps that start of the prompt in its cache, so the first
    answer begins without reading it.
    """
    started = time.monotonic()
    # The chat template of Qwen refuses a prompt without a user message.
    messages = [*system, {"role": "user", "content": "Hello"}]
    body: dict = {"model": MODEL, "messages": messages, "max_tokens": 1, "reasoning_effort": "none"}
    if tools.specs:
        body["tools"] = tools.specs  # the same start as the requests of answer()
    async with http.post(f"{LLM_URL}/chat/completions", json=body) as r:
        await _check(r)
    log(f"the LLM read the system prompt in {time.monotonic() - started:.0f} s")


def report(task: asyncio.Task) -> None:
    """Log the error of a background task that ended."""
    if not task.cancelled() and task.exception():
        log(f"{task.get_name()} failed: {task.exception()}")


async def run_loop(
    http: aiohttp.ClientSession, stack: contextlib.AsyncExitStack, args: argparse.Namespace
) -> None:
    """Run the conversation loop until Ctrl-C."""
    session = await chora(stack)
    tools = Tools(session, http)
    await tools.load()
    ptt = asyncio.Event()
    if session:
        task = asyncio.create_task(push_to_talk(session, ptt), name="push-to-talk")
        task.add_done_callback(report)
    key = PTT_KEY if session else "off"
    log(f"RMS_SPEECH {RMS_SPEECH}, RMS_BARGE_IN {RMS_BARGE_IN}, PTT_KEY {key}")
    history = [{"role": "system", "content": system_prompt()}]
    log(f"system prompt: {len(history[0]['content'])} characters")
    past = spoken(json.loads(HISTORY_FILE.read_text())) if HISTORY_FILE.is_file() else []
    if note := earlier(past):
        history[0]["content"] += "\n\n" + note
        log(f"the earlier conversation of {HISTORY_FILE}: {len(note)} characters")
    warming = asyncio.create_task(warm(http, tools, history[:1]), name="warm-up")
    warming.add_done_callback(report)
    preroll: list[bytes] = []
    async with Mic() as mic:
        while True:
            # A failed server or tool costs one turn, not the conversation.
            start = len(history)
            try:
                preroll = await turn(mic, tools, history, ptt, preroll)
            except TURN_ERRORS as err:
                log(f"turn failed, the history drops it: {err!r}")
                del history[start:]
                preroll = []
            trim(history)
            save(past, history)


async def run_roundtrip(
    http: aiohttp.ClientSession, stack: contextlib.AsyncExitStack, args: argparse.Namespace
) -> None:
    """Speak a text into the ASR and answer the transcript."""
    text = " ".join(args.text) or "Wie viel Uhr haben wir in Tokio, wenn es in Berlin Mittag wird?"
    pcm = b"".join([chunk async for chunk in tts(http, text)])
    log(f"tts gave {len(pcm) / (RATE * 2):.2f} s for: {text}")
    said = await transcribe(http, pcm)
    log(f"asr heard: {said}")
    history = [{"role": "system", "content": SYSTEM}, {"role": "user", "content": said}]
    await speak(http, sentences(answer(http, Tools(None, http), history)))


async def run_tts(
    http: aiohttp.ClientSession, stack: contextlib.AsyncExitStack, args: argparse.Namespace
) -> None:
    """Speak a text."""
    await speak(http, one(" ".join(args.text) or "Hallo, die Sprachausgabe läuft."))


async def run_asr(
    http: aiohttp.ClientSession, stack: contextlib.AsyncExitStack, args: argparse.Namespace
) -> None:
    """Print the transcript of a raw PCM file: 24 kHz, 16 bit, mono.

    `arecord -r 24000 -c 1 -f S16_LE -t raw FILE` records one.
    """
    with open(args.file, "rb") as f:
        print(await transcribe(http, f.read()))


async def run_levels(
    http: aiohttp.ClientSession, stack: contextlib.AsyncExitStack, args: argparse.Namespace
) -> None:
    """Show the live mic level until Ctrl-C."""
    log(f"RMS_SPEECH {RMS_SPEECH}, RMS_BARGE_IN {RMS_BARGE_IN}, SPEECH_RATIO {SPEECH_RATIO}")
    peak = 0.0
    async with Mic() as mic:
        try:
            while True:
                level = rms(await mic.frame())
                peak = max(peak, level)
                if level < mic.speech:
                    mic.learn(level)
                print(
                    f"\rlevel {level:7.0f} peak {peak:7.0f} quiet {mic.floor:6.1f}"
                    f" speech {mic.speech:6.0f}",
                    end="",
                    flush=True,
                )
        finally:
            print()  # the prompt after Ctrl-C goes on a new line


async def run_tools(
    http: aiohttp.ClientSession, stack: contextlib.AsyncExitStack, args: argparse.Namespace
) -> None:
    """Print the names of the tools of the LLM."""
    tools = Tools(await chora(stack), http)
    await tools.load()
    for spec in tools.specs:
        print(spec["function"]["name"])


async def main() -> None:
    """Parse the command line and run a command."""
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    sub = parser.add_subparsers()
    text = ("text", {"nargs": "*", "help": "the text; a German sentence without one"})
    commands = {
        "roundtrip": (run_roundtrip, [text]),
        "tts": (run_tts, [text]),
        "asr": (run_asr, [("file", {"help": "the raw PCM file"})]),
        "levels": (run_levels, []),
        "tools": (run_tools, []),
    }
    for name, (func, arguments) in commands.items():
        doc = func.__doc__
        command = sub.add_parser(
            name,
            help=doc.splitlines()[0],
            description=doc,
            formatter_class=argparse.RawDescriptionHelpFormatter,
        )
        command.set_defaults(func=func)
        for argument, options in arguments:
            command.add_argument(argument, **options)
    args = parser.parse_args()
    func = getattr(args, "func", run_loop)
    async with aiohttp.ClientSession() as http, contextlib.AsyncExitStack() as stack:
        await func(http, stack, args)


if __name__ == "__main__":
    with contextlib.suppress(KeyboardInterrupt):
        asyncio.run(main())
