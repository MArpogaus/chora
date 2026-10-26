#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["aiohttp>=3.9,<4", "mcp>=2.3,<3"]
# ///
"""Voice agent: microphone -> ASR -> LLM with tools -> TTS -> speaker.

agent.py                 the conversation loop (default)
agent.py roundtrip [T]   TTS speaks T into ASR, the LLM answers, TTS plays
agent.py tts [TEXT]      text -> TTS -> speaker
agent.py asr FILE        PCM or WAV file -> ASR -> transcript
agent.py levels          live mic level, to tune RMS_SPEECH
agent.py tools           the tools the LLM gets from chora and this agent
agent.py ptt             print the push-to-talk key as chora reports it

A turn starts when the mic level passes RMS_SPEECH and ends after
SILENCE_MS of quiet. While PTT_KEY is held in chora, the agent records
whatever the level, and the turn ends when the key goes up. The LLM gets
the MCP tools of `choractl mcp` and a tool `delegate` that hands bigger
tasks to CODER_CMD. Every setting is an environment variable.
"""

import argparse
import array
import asyncio
import base64
import binascii
import json
import math
import os
import shlex
import sys
from contextlib import AsyncExitStack

import aiohttp
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

ENV = os.environ.get
ASR_URL = ENV("ASR_URL", "ws://127.0.0.1:8881/v1/realtime?intent=transcription")
TTS_URL = ENV("TTS_URL", "http://127.0.0.1:8882")
LLM_URL = ENV("LLM_URL", "http://127.0.0.1:8888/v1")
MODEL = ENV("MODEL", "gufo")  # the served name of all three models
RMS_SPEECH = int(ENV("RMS_SPEECH", "5000"))
SILENCE_MS = int(ENV("SILENCE_MS", "700"))
PTT_KEY = ENV("PTT_KEY", "Super+v")  # chora reports its release with Super
CHORA_NAME = ENV("CHORA_NAME", "voice")  # the client name of the agent
CODER_CMD = ENV("CODER_CMD", "")  # for example "pi -p" or "claude -p"
SYSTEM = ENV("SYSTEM", (
    "You are a voice assistant on a Linux desktop named chora. Answer in one "
    "or two short spoken sentences, without markdown. Use the tools to look "
    "at and change the desktop. Hand a task that needs code or many steps to "
    "delegate, then say that it runs."))

RATE = 24000
FRAME_MS = 20
FRAME_BYTES = RATE * FRAME_MS // 1000 * 2
HISTORY = 12  # messages kept besides the system prompt
TOOL_ROUNDS = 5  # LLM calls per turn while it asks for tools
# Tools the agent uses itself; the LLM does not get them.
OWN_TOOLS = {"subscribe_events", "wait_events"}


def log(*args):
    print(*args, file=sys.stderr, flush=True)


def rms(frame):
    samples = array.array("h", frame)
    return math.sqrt(sum(s * s for s in samples) / len(samples))


# --- audio -------------------------------------------------------------

async def mic():
    # pw-record and pw-play sniff headers and choke on raw pipes; the alsa
    # tools take an explicit raw format.
    return await asyncio.create_subprocess_exec(
        "arecord", "-q", "-r", str(RATE), "-c", "1", "-f", "S16_LE", "-t", "raw", "-",
        stdout=asyncio.subprocess.PIPE)


async def play(pcm_chunks):
    player = await asyncio.create_subprocess_exec(
        "aplay", "-q", "-r", str(RATE), "-c", "1", "-f", "S16_LE", "-t", "raw", "-",
        stdin=asyncio.subprocess.PIPE)
    try:
        async for chunk in pcm_chunks:
            player.stdin.write(chunk)
            await player.stdin.drain()
    finally:
        player.stdin.close()
        await player.wait()


# --- speech servers ----------------------------------------------------

async def tts(http, text):
    """Yields the PCM of TEXT from the TTS websocket."""
    async with http.get(f"{TTS_URL}/v1/audio/voices") as r:
        voice = (await r.json())["data"][0]["id"]
    url = TTS_URL.replace("http", "ws", 1) + "/v1/audio/speech/stream"
    async with http.ws_connect(url) as ws:
        # The server refuses stream_audio without response_format pcm.
        await ws.send_json({"type": "session.config", "model": MODEL, "voice": voice,
                            "response_format": "pcm", "stream_audio": True})
        await ws.send_json({"type": "input.text", "text": text})
        await ws.send_json({"type": "input.done"})
        while True:
            msg = await asyncio.wait_for(ws.receive(), 30)
            if msg.type == aiohttp.WSMsgType.BINARY:
                yield msg.data
            elif msg.type == aiohttp.WSMsgType.TEXT:
                event = json.loads(msg.data)
                kind = event.get("type", "")
                if "error" in kind:
                    log("tts:", msg.data[:200])
                    return
                if kind.endswith("done"):
                    return
                # Some servers send the PCM as base64 in a text event.
                if isinstance(event.get("audio"), str):
                    try:
                        yield base64.b64decode(event["audio"])
                    except binascii.Error:
                        log("tts: no base64 in", msg.data[:200])
            else:
                return


async def speak(http, text):
    log(f"agent: {text}")
    await play(tts(http, text))


async def transcribe(http, pcm):
    """Sends raw PCM to the ASR websocket and returns the transcript."""
    async with http.ws_connect(ASR_URL) as ws:
        await ws.send_json({"type": "session.update", "session": {
            "type": "transcription",
            "audio": {"input": {"format": {"type": "audio/pcm", "rate": RATE},
                                "transcription": {"model": MODEL},
                                "turn_detection": None}}}})
        for i in range(0, len(pcm), FRAME_BYTES):
            chunk = base64.b64encode(pcm[i:i + FRAME_BYTES]).decode()
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


# --- tools -------------------------------------------------------------

class Tools:
    """The tools of the LLM: those of `choractl mcp` and `delegate`."""

    def __init__(self, session, http):
        self.session = session
        self.http = http
        self.specs = []
        self.names = {}  # OpenAI function name -> MCP tool name
        self.jobs = set()

    async def load(self):
        if self.session:
            for tool in (await self.session.list_tools()).tools:
                if tool.name in OWN_TOOLS:
                    continue
                # OpenAI function names take no dots, plugin methods have one.
                name = tool.name.replace(".", "__")
                self.names[name] = tool.name
                self.specs.append(function(name, tool.description, tool.input_schema))
        if CODER_CMD:
            self.specs.append(function(
                "delegate",
                "Hand a bigger task, such as code changes, to a coding agent. "
                "It runs in the background and the result is spoken when it ends.",
                {"type": "object", "required": ["task"], "properties": {
                    "task": {"type": "string", "description": "the task in full"},
                    "dir": {"type": "string", "description": "the working directory"}}}))

    async def call(self, name, arguments):
        if name == "delegate" and CODER_CMD:
            return self.delegate(**arguments)
        if name not in self.names:
            return f"no tool {name}"
        result = await self.session.call_tool(self.names[name], arguments)
        # The LLM reads text only; images of captures stay out.
        text = "\n".join(c.text for c in result.content if c.type == "text")
        return ("error: " if result.is_error else "") + (text or "done")

    def delegate(self, task, dir="~"):
        job = asyncio.create_task(self.run_coder(task, os.path.expanduser(dir)))
        self.jobs.add(job)
        job.add_done_callback(self.jobs.discard)
        return "the coding agent started"

    async def run_coder(self, task, cwd):
        log(f"delegate in {cwd}: {task}")
        proc = await asyncio.create_subprocess_exec(
            *shlex.split(CODER_CMD), task, cwd=cwd,
            stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.STDOUT)
        out, _ = await proc.communicate()
        lines = [l for l in out.decode(errors="replace").splitlines() if l.strip()]
        last = lines[-1][:200] if lines else "no output"
        state = "is done" if proc.returncode == 0 else "failed"
        try:
            await speak(self.http, f"The coding task {state}. {last}")
        except (aiohttp.ClientError, asyncio.TimeoutError) as err:
            log(f"cannot speak the result: {err}")


def function(name, description, parameters):
    return {"type": "function", "function": {
        "name": name, "description": description or name,
        "parameters": parameters or {"type": "object", "properties": {}}}}


async def answer(http, tools, history):
    """Asks the LLM until it answers in words, running the tools it calls."""
    for _ in range(TOOL_ROUNDS):
        body = {"model": MODEL, "messages": history}
        if tools.specs:
            body["tools"] = tools.specs
        async with http.post(f"{LLM_URL}/chat/completions", json=body) as r:
            message = (await r.json())["choices"][0]["message"]
        history.append(message)
        calls = message.get("tool_calls") or []
        if not calls:
            return message.get("content") or ""
        for call in calls:
            fn = call["function"]
            arguments = json.loads(fn.get("arguments") or "{}")
            log(f"tool {fn['name']} {arguments}")
            result = await tools.call(fn["name"], arguments)
            history.append({"role": "tool", "tool_call_id": call["id"], "content": result})
    return "I stopped after too many tool calls."


# --- listening ---------------------------------------------------------

class PushToTalk:
    """Follows PTT_KEY through chora: held between pressed and released."""

    def __init__(self, session):
        self.session = session
        self.held = asyncio.Event()
        self.up = asyncio.Event()

    async def watch(self):
        args = {"events": ["input.key"], "keys": [PTT_KEY]}
        result = await self.session.call_tool("subscribe_events", args)
        if result.is_error:
            raise SystemExit(f"push-to-talk: {result.content[0].text}")
        # chora names the binding in its own spelling.
        binding = json.loads(result.content[0].text)["keys"][0]
        while True:
            result = await self.session.call_tool("wait_events", {"timeout_ms": 30000})
            for c in result.content:
                if c.type != "text":
                    continue
                for event in json.loads(c.text).get("events", []):
                    if event.get("binding") != binding:
                        continue
                    if event.get("state") == "pressed" and not self.held.is_set():
                        self.up.clear()
                        self.held.set()
                    elif event.get("state") == "released":
                        self.held.clear()
                        self.up.set()


async def record(ptt):
    """Records one turn: by level, or while the push-to-talk key is held."""
    frames, quiet, speaking = [], 0, False
    proc = await mic()
    try:
        while True:
            frame = await proc.stdout.readexactly(FRAME_BYTES)
            held = ptt is not None and ptt.held.is_set()
            loud = rms(frame) >= RMS_SPEECH
            speaking = speaking or loud or held
            if speaking:
                frames.append(frame)
            if held:
                quiet = 0
            elif ptt is not None and ptt.up.is_set() and frames:
                ptt.up.clear()
                return b"".join(frames)
            else:
                quiet = 0 if loud else quiet + 1
                if speaking and quiet * FRAME_MS >= SILENCE_MS:
                    return b"".join(frames)
    finally:
        proc.kill()


# --- commands ----------------------------------------------------------

async def chora(stack):
    """The MCP session of `choractl mcp`, or None when chora is not there."""
    # The mcp package passes only a few variables by default; choractl
    # needs CHORA_SOCKET and the rest of the session.
    params = StdioServerParameters(command="choractl", args=["--name", CHORA_NAME, "mcp"],
                                   env=dict(os.environ))
    try:
        read, write = await stack.enter_async_context(stdio_client(params))
        session = await stack.enter_async_context(ClientSession(read, write))
        await session.initialize()
        return session
    except Exception as err:  # no choractl, no core: talk without tools
        log(f"no chora tools: {err}")
        return None


async def run_loop(http, stack, args):
    session = await chora(stack)
    tools = Tools(session, http)
    await tools.load()
    ptt = None
    if session and PTT_KEY:
        ptt = PushToTalk(session)
        asyncio.create_task(ptt.watch())
    history = [{"role": "system", "content": SYSTEM}]
    log(f"listening; level {RMS_SPEECH}, push-to-talk {PTT_KEY if ptt else 'off'}")
    while True:
        said = await transcribe(http, await record(ptt))
        log(f"you: {said}")
        if not said:
            continue
        history.append({"role": "user", "content": said})
        reply = await answer(http, tools, history)
        del history[1:-HISTORY]
        if reply:
            await speak(http, reply)


async def run_roundtrip(http, stack, args):
    text = " ".join(args.text) or "What time is it in Tokyo when it is noon in Berlin?"
    pcm = b"".join([chunk async for chunk in tts(http, text)])
    log(f"tts gave {len(pcm) / (RATE * 2):.2f} s for: {text}")
    said = await transcribe(http, pcm)
    log(f"asr heard: {said}")
    tools = Tools(None, http)
    history = [{"role": "system", "content": SYSTEM}, {"role": "user", "content": said}]
    await speak(http, await answer(http, tools, history))


async def run_tts(http, stack, args):
    await speak(http, " ".join(args.text) or "Hallo, die Sprachausgabe läuft.")


async def run_asr(http, stack, args):
    pcm = open(args.file, "rb").read()
    if pcm[:4] == b"RIFF":
        pcm = pcm[44:]
    print(await transcribe(http, pcm))


async def run_levels(http, stack, args):
    proc = await mic()
    peak = 0.0
    try:
        while True:
            level = rms(await proc.stdout.readexactly(FRAME_BYTES))
            peak = max(peak, level)
            print(f"\rlevel {level:7.0f} peak {peak:7.0f} " + "#" * min(40, int(level / 500)),
                  end="", flush=True)
    finally:
        proc.kill()


async def run_tools(http, stack, args):
    tools = Tools(await chora(stack), http)
    await tools.load()
    for spec in tools.specs:
        print(spec["function"]["name"])


async def run_ptt(http, stack, args):
    session = await chora(stack)
    if not session:
        sys.exit("push-to-talk needs chora")
    ptt = PushToTalk(session)
    watch = asyncio.create_task(ptt.watch())
    print(f"hold {PTT_KEY}", flush=True)
    while not watch.done():
        await ptt.held.wait()
        print("held", flush=True)
        await ptt.up.wait()
        print("up", flush=True)
    await watch


async def main():
    p = argparse.ArgumentParser(description=__doc__,
                                formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest="command")
    for name, func in [("roundtrip", run_roundtrip), ("tts", run_tts)]:
        cmd = sub.add_parser(name)
        cmd.add_argument("text", nargs="*")
        cmd.set_defaults(func=func)
    sub.add_parser("asr").add_argument("file")
    for name, func in [("levels", run_levels), ("tools", run_tools), ("ptt", run_ptt)]:
        sub.add_parser(name).set_defaults(func=func)
    sub.choices["asr"].set_defaults(func=run_asr)
    args = p.parse_args()
    func = getattr(args, "func", run_loop)
    async with aiohttp.ClientSession() as http, AsyncExitStack() as stack:
        await func(http, stack, args)


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        pass
