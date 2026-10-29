#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["aiohttp>=3.9,<4", "mcp>=2.3,<3"]
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
import re
import shlex
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
# Not LANGUAGE, which gettext reads. Without a language, the ASR guesses.
ASR_LANGUAGE = ENV("ASR_LANGUAGE", "de")
RMS_SPEECH = int(ENV("RMS_SPEECH", "5000"))
SILENCE_MS = int(ENV("SILENCE_MS", "700"))
SETTLE_MS = int(ENV("SETTLE_MS", "300"))  # quiet before a turn may start
# Speech that stops the reply: louder than RMS_SPEECH, since the mic also
# hears the speaker, and for BARGE_MS.
RMS_BARGE_IN = int(ENV("RMS_BARGE_IN", str(2 * RMS_SPEECH)))
BARGE_MS = int(ENV("BARGE_MS", "300"))
PTT_KEY = ENV("PTT_KEY", "Super+v")  # chora reports its release with Super
CHORA_NAME = ENV("CHORA_NAME", "voice")  # the client name of the agent
CODER_CMD = ENV("CODER_CMD", "")  # for example "claude -p --permission-mode acceptEdits"
SYSTEM = ENV(
    "SYSTEM",
    "You are a voice assistant on a Linux desktop named chora. Answer in the "
    "language of the user, in one or two short spoken sentences, without "
    "markdown. Use the tools to look at and change the desktop.",
)
if CODER_CMD:
    SYSTEM += " Hand a task that needs code or many steps to delegate, then say that it runs."

RATE = 24000
FRAME_MS = 20
FRAME_BYTES = RATE * FRAME_MS // 1000 * 2
PCM = ("-q", "-r", str(RATE), "-c", "1", "-f", "S16_LE", "-t", "raw", "-")  # aplay, arecord
PREROLL_MS = 1000  # speech before a barge-in that the next turn keeps
NO_QUIET_MS = 2000  # when the agent says that the room stays loud
HISTORY = 12  # messages kept besides the system prompt
TOOL_ROUNDS = 5  # LLM calls per turn while it asks for tools
FIRST_PIECE = 40  # characters after which a comma may end a piece
SENTENCE_END = re.compile(r"(?<=[.!?:;])\s+")
# The agent uses the event tools itself, and the LLM reads no images.
SKIPPED_TOOLS = {"subscribe_events", "wait_events", "capture"}
TURN_ERRORS = (aiohttp.ClientError, TimeoutError, KeyError, ValueError, OSError, MCPError)

SPEAKING = asyncio.Lock()  # one voice at a time: replies and delegate results


# %% private helpers
def log(*args: object) -> None:
    """Print a line to stderr."""
    print(*args, file=sys.stderr, flush=True)


def rms(frame: bytes) -> float:
    """Return the root mean square of a frame of 16 bit samples."""
    samples = array.array("h", frame)
    return math.sqrt(sum(s * s for s in samples) / len(samples))


def ms(frames: int) -> int:
    """Return the length of a number of frames in milliseconds."""
    return frames * FRAME_MS


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

    A task reads arecord into a queue, so a turn that starts with a
    barge-in goes on from the reply without a gap. Use it as an async
    context manager.
    """

    def __init__(self) -> None:
        self.queue: asyncio.Queue[bytes | Exception] = asyncio.Queue()

    async def __aenter__(self) -> "Mic":
        """Start arecord and the reader task."""
        # pw-record sniffs headers and chokes on raw pipes; arecord takes
        # an explicit raw format.
        self.proc = await asyncio.create_subprocess_exec(
            "arecord", *PCM, stdout=asyncio.subprocess.PIPE
        )
        self.reader = asyncio.create_task(self._read())
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
            self.queue.get_nowait()


async def play(pcm_chunks: AsyncIterator[bytes]) -> None:
    """Play raw PCM chunks through aplay.

    Parameters
    ----------
    pcm_chunks : AsyncIterator[bytes]
        16 bit mono samples at RATE.
    """
    player = await asyncio.create_subprocess_exec("aplay", *PCM, stdin=asyncio.subprocess.PIPE)
    end = time.monotonic()  # when the sound written so far ends
    try:
        async for chunk in pcm_chunks:
            player.stdin.write(chunk)
            end = max(end, time.monotonic()) + len(chunk) / (RATE * 2)
            await player.stdin.drain()
    except asyncio.CancelledError:
        player.kill()  # a barge-in: stop the sound at once
        raise
    finally:
        player.stdin.close()
        # aplay on PipeWire may never exit after its input ends: wait for
        # the sound it got, plus a second, then end it.
        try:
            await asyncio.wait_for(player.wait(), max(end - time.monotonic() + 1, 1))
        except TimeoutError:
            log("aplay did not end after the sound; ended it")
            player.kill()
            await player.wait()


# %% speech servers
async def tts(http: aiohttp.ClientSession, text: str) -> AsyncIterator[bytes]:
    """Yield the PCM of one text from the TTS websocket.

    Parameters
    ----------
    http : aiohttp.ClientSession
        The session for the requests.
    text : str
        The text; the server reads it whole.
    """
    async with http.ws_connect(TTS_URL) as ws:
        # The server refuses stream_audio without response_format pcm.
        config = {"model": MODEL, "voice": VOICE, "response_format": "pcm"}
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

    A task reads the sentences ahead into a queue, so the LLM writes on
    while TTS speaks.

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
        for i in range(0, len(pcm), FRAME_BYTES):
            chunk = base64.b64encode(pcm[i : i + FRAME_BYTES]).decode()
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
            if "error" in kind:
                log("asr:", msg.data[:200])
                return ""


# %% llm and tools
class Tools:
    """The tools of the LLM: those of `choractl mcp` and `delegate`.

    Parameters
    ----------
    session : ClientSession | None
        The MCP session of chora; None gives no chora tools.
    http : aiohttp.ClientSession
        The session for speaking the results of `delegate`.
    """

    def __init__(self, session: ClientSession | None, http: aiohttp.ClientSession) -> None:
        self.session = session
        self.http = http
        self.specs: list[dict] = []
        self.names: dict[str, str] = {}  # OpenAI function name -> MCP tool name
        self.jobs: set[asyncio.Task] = set()

    async def load(self) -> None:
        """Ask chora for its tools and add `delegate`."""
        if self.session:
            for tool in (await self.session.list_tools()).tools:
                if tool.name in SKIPPED_TOOLS:
                    continue
                # OpenAI function names take no dots, plugin methods have one.
                name = tool.name.replace(".", "__")
                self.names[name] = tool.name
                self.specs.append(function(name, tool.description, tool.input_schema))
        if CODER_CMD:
            task = {"type": "string", "description": "the task in full"}
            parameters = {"type": "object", "required": ["task"], "properties": {"task": task}}
            description = (
                "Hand a bigger task, such as code changes, to a coding agent. "
                "It runs in the background and the result is spoken when it ends."
            )
            self.specs.append(function("delegate", description, parameters))

    async def call(self, name: str, arguments: dict) -> str:
        """Run a tool and return its text for the LLM."""
        if name == "delegate" and CODER_CMD:  # else the task would be the program
            return self.delegate(arguments["task"])
        if name not in self.names:
            return f"no tool {name}"
        result = await self.session.call_tool(self.names[name], arguments)
        # The LLM reads text only, so the image of describe_screen stays out.
        text = "\n".join(c.text for c in result.content if c.type == "text")
        return ("error: " if result.is_error else "") + (text or "done")

    def delegate(self, task: str) -> str:
        """Start CODER_CMD with a task in the background."""
        job = asyncio.create_task(self._run_coder(task), name="delegate")
        self.jobs.add(job)
        job.add_done_callback(self.jobs.discard)
        job.add_done_callback(report)
        return "the coding agent started"

    async def _run_coder(self, task: str) -> None:
        log(f"delegate: {task}")
        try:
            proc = await asyncio.create_subprocess_exec(
                *shlex.split(CODER_CMD),
                task,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.STDOUT,
            )
            out, _ = await proc.communicate()
            lines = [line for line in out.decode(errors="replace").splitlines() if line.strip()]
            last = lines[-1][:200] if lines else "no output"
            state = "is done" if proc.returncode == 0 else "failed"
        except OSError as err:
            last, state = str(err), "did not start"
        try:
            await speak(self.http, one(f"The coding task {state}. {last}"))
        except TURN_ERRORS as err:
            log(f"cannot speak the result: {err}")


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

    def message(self) -> dict | None:
        """Return the message for the history, with the calls that have results."""
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
        return message if self.text or calls else None


async def _deltas(http: aiohttp.ClientSession, body: dict) -> AsyncIterator[dict]:
    async with http.post(f"{LLM_URL}/chat/completions", json=body) as r:
        r.raise_for_status()
        async for raw in r.content:
            line = raw.decode().strip()
            if line.startswith("data:") and line != "data: [DONE]":
                event = json.loads(line[5:])
                if "error" in event:
                    raise ValueError(f"llm: {event['error']}")
                yield (event.get("choices") or [{}])[0].get("delta") or {}


async def _run_call(tools: Tools, call: dict) -> dict:
    try:
        arguments = json.loads(call["args"] or "{}")
        log(f"tool {call['name']} {arguments}")
        result = await tools.call(call["name"], arguments)
    except TURN_ERRORS as err:  # the LLM reads the error and goes on
        result = f"error: {err}"
    return {"role": "tool", "tool_call_id": call["id"], "content": result}


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
        body: dict = {"model": MODEL, "messages": history, "stream": True}
        if tools.specs:
            body["tools"] = tools.specs
        reply = Reply()
        try:
            async for delta in _deltas(http, body):
                if words := reply.add(delta):
                    yield words
            if reply.text and reply.calls:
                yield " "  # so the words before the tools are spoken while they run
            for call in reply.calls.values():
                reply.results.append(await _run_call(tools, call))
        finally:
            # A failed or cut round keeps its words and the calls that ran,
            # so no call is left without its result.
            if message := reply.message():
                history.append(message)
            history += reply.results
        if not reply.calls:
            if not reply.text:
                log("the LLM gave no words and no tool calls")
            return
    yield "I stopped after too many tool calls."


async def sentences(words: AsyncIterator[str]) -> AsyncIterator[str]:
    """Group streamed words into sentences.

    TTS reads a text whole, so a sentence is the smallest piece it can
    start on. The first sentence of a reply may end at a comma, so the
    first sound comes sooner.

    Parameters
    ----------
    words : AsyncIterator[str]
        The words of the LLM, as they come.
    """
    buffer, first = "", True
    async for word in words:
        buffer += word
        *done, buffer = SENTENCE_END.split(buffer)
        if first and not done and len(buffer) > FIRST_PIECE and ", " in buffer:
            head, buffer = buffer.rsplit(", ", 1)
            done = [head + ","]
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
        if result.is_error:  # chora is not back yet
            await asyncio.sleep(1)
            continue
        first = False
        binding = json.loads(result.content[0].text)["keys"][0]  # in chora's spelling
        try:
            await _follow_key(session, down, binding)
        finally:
            down.clear()  # a key that nobody follows is up
        await asyncio.sleep(1)


async def _follow_key(session: ClientSession, down: asyncio.Event, binding: str) -> None:
    while True:
        result = await session.call_tool("wait_events", {"timeout_ms": 30000})
        batch = json.loads(result.content[0].text)
        for event in batch.get("events", []):
            if event.get("binding") != binding:
                continue
            if event.get("state") == "pressed":
                down.set()
            elif event.get("state") == "released":
                down.clear()
        if batch.get("ended"):
            log(f"push-to-talk ended: {batch['ended']}")
            return


async def settle(mic: Mic, ptt: asyncio.Event) -> None:
    """Wait for SETTLE_MS below RMS_SPEECH, or for the push-to-talk key.

    The frames that came while the agent was busy go first, and the quiet
    keeps the loud start of the mic and the end of a reply out.
    """
    async with SPEAKING:  # a spoken delegate result is no turn
        mic.flush()
    log("listening")
    quiet, waited = 0, 0
    while ms(quiet) < SETTLE_MS and not ptt.is_set():
        level = rms(await mic.frame())
        quiet = 0 if level >= RMS_SPEECH else quiet + 1
        waited += 1
        if ms(waited) == NO_QUIET_MS:
            log(f"no quiet below RMS_SPEECH {RMS_SPEECH}, level {level:.0f}")
    log("speak now")


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
    frames, quiet, pushed = list(preroll), 0, False
    while True:
        frame = await mic.frame()
        if SPEAKING.locked():  # a delegate result: wait for its end and for quiet
            return await record(mic, ptt, [])
        loud, down = rms(frame) >= RMS_SPEECH, ptt.is_set()
        pushed = pushed or down
        if frames or loud or down:
            frames.append(frame)
        if down:
            continue
        if pushed:  # the key went up
            return b"".join(frames)
        quiet = 0 if loud else quiet + 1
        if frames and ms(quiet) >= SILENCE_MS:
            return b"".join(frames)


async def talk(
    http: aiohttp.ClientSession, mic: Mic, words: AsyncIterator[str], ptt: asyncio.Event
) -> list[bytes]:
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
    list[bytes]
        The last PREROLL_MS of frames when the user talked over the reply,
        which stopped it; else an empty list, also after a stop by key.
    """
    reply = asyncio.create_task(speak(http, sentences(words)))
    recent: collections.deque[bytes] = collections.deque(maxlen=PREROLL_MS // FRAME_MS)
    loud, peak = 0, 0.0
    mic.flush()
    while not reply.done():  # the mic gives a frame every 20 ms
        recent.append(await mic.frame())
        level = rms(recent[-1])
        peak = max(peak, level)
        loud = loud + 1 if level >= RMS_BARGE_IN else 0
        by_key = ptt.is_set()
        if ms(loud) >= BARGE_MS or by_key:
            log("barge-in by key" if by_key else f"barge-in at level {level:.0f}")
            reply.cancel()
            with contextlib.suppress(asyncio.CancelledError):
                await reply
            # By key, the recent frames hold the reply, not the user.
            return [] if by_key else list(recent)
    log(f"mic peak during the reply: {peak:.0f}, barge-in at {RMS_BARGE_IN}")
    await reply  # its errors belong to the turn
    return []


# %% commands
async def chora(stack: contextlib.AsyncExitStack) -> ClientSession | None:
    """Return the MCP session of `choractl mcp`, or None without chora."""
    # The mcp package passes only a few variables by default; choractl
    # needs CHORA_SOCKET and the rest of the session.
    args = ["--name", CHORA_NAME, "mcp"]
    params = StdioServerParameters(command="choractl", args=args, env=dict(os.environ))
    try:
        read, write = await stack.enter_async_context(stdio_client(params))
        session = await stack.enter_async_context(ClientSession(read, write))
        await session.initialize()
    except Exception as err:  # no choractl or no core: talk without tools
        log(f"no chora tools: {err}")
        return None
    return session


def trim(history: list[dict]) -> None:
    """Keep the last turns, from a user message on, and at least the last turn.

    This keeps each tool result with the call that it answers.
    """
    users = [i for i, m in enumerate(history) if m["role"] == "user"]
    start = next((i for i in users if i >= len(history) - HISTORY), users[-1] if users else 1)
    del history[1:start]


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
    if session and PTT_KEY:
        task = asyncio.create_task(push_to_talk(session, ptt), name="push-to-talk")
        task.add_done_callback(report)
    key = PTT_KEY if session else "off"
    log(f"RMS_SPEECH {RMS_SPEECH}, RMS_BARGE_IN {RMS_BARGE_IN}, PTT_KEY {key}")
    history = [{"role": "system", "content": SYSTEM}]
    preroll: list[bytes] = []
    async with Mic() as mic:
        while True:
            # A failed server or tool costs one turn, not the conversation.
            try:
                pcm = await record(mic, ptt, preroll)
                preroll = []
                said = await transcribe(http, pcm)
                log(f"you: {said}")
                if said:
                    if history[-1]["role"] == "user":  # a cut turn got no answer
                        history[-1]["content"] += " " + said
                    else:
                        history.append({"role": "user", "content": said})
                    preroll = await talk(http, mic, answer(http, tools, history), ptt)
            except TURN_ERRORS as err:
                # The history may be what the LLM refuses, such as a context
                # that grew too long, so the next turn starts anew.
                log(f"turn failed, the conversation starts anew: {err!r}")
                del history[1:]
            trim(history)


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
    peak = 0.0
    async with Mic() as mic:
        while True:
            level = rms(await mic.frame())
            peak = max(peak, level)
            bar = "#" * min(40, int(level / 500))
            print(f"\rlevel {level:7.0f} peak {peak:7.0f} {bar:40}", end="", flush=True)


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
    sub = parser.add_subparsers(dest="command")
    commands = {
        "roundtrip": run_roundtrip,
        "tts": run_tts,
        "asr": run_asr,
        "levels": run_levels,
        "tools": run_tools,
    }
    for name, func in commands.items():
        doc = func.__doc__
        command = sub.add_parser(name, help=doc.splitlines()[0], description=doc)
        command.set_defaults(func=func)
        if name in ("roundtrip", "tts"):
            command.add_argument("text", nargs="*")
        elif name == "asr":
            command.add_argument("file")
    args = parser.parse_args()
    func = getattr(args, "func", run_loop)
    async with aiohttp.ClientSession() as http, contextlib.AsyncExitStack() as stack:
        await func(http, stack, args)


if __name__ == "__main__":
    with contextlib.suppress(KeyboardInterrupt):
        asyncio.run(main())
