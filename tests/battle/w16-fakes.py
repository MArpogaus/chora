#!/usr/bin/env python3
"""Fakes for W16, so that the real chora-voice runs without a GPU server.

The fakes of services/voice/src/fake.rs are `#[cfg(test)]` code of the
binary, so this file does the same in Python (standard library only):

  w16-fakes.py servers DIR   ASR and TTS websockets and an LLM with SSE on
                             127.0.0.1; DIR/ports gets one JSON line.
  w16-fakes.py mic DIR       a capture command: silence in real time, and
                             the speech of DIR/speech.pcm for each line
                             that DIR/mic.cmd gets.
  w16-fakes.py play DIR      a playback command: reads PCM in real time and
                             logs the chunks to DIR/play.log.

The LLM and the ASR follow a script that the test pushes by HTTP to the
port of the LLM:
  POST /_llm {"text": "..."} or {"tool": "name", "args": {...}}
  POST /_asr {"text": "..."}
  GET  /_state  -> pending replies
The requests of the LLM go to DIR/llm.log (one JSON body per line), the
commits of the ASR to DIR/asr.log, the texts of the TTS to DIR/tts.log.
"""

import asyncio
import base64
import hashlib
import json
import os
import struct
import sys
import time

RATE = 24000
MAGIC = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11"


def append(path, line):
    with open(path, "a", encoding="utf-8") as f:
        f.write(line + "\n")


# --- a websocket by hand: handshake, text and binary frames ----------------
async def handshake(reader, writer):
    head = (await reader.readuntil(b"\r\n\r\n")).decode("latin-1")
    key = next(
        line.split(":", 1)[1].strip()
        for line in head.split("\r\n")
        if line.lower().startswith("sec-websocket-key")
    )
    accept = base64.b64encode(hashlib.sha1(key.encode() + MAGIC).digest()).decode()
    writer.write(
        (
            "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\n"
            f"Connection: Upgrade\r\nSec-WebSocket-Accept: {accept}\r\n\r\n"
        ).encode()
    )
    await writer.drain()


async def recv(reader):
    """The next text message, None at the end of the socket."""
    data, kind = b"", None
    while True:
        try:
            b1, b2 = await reader.readexactly(2)
            size = b2 & 0x7F
            if size == 126:
                (size,) = struct.unpack(">H", await reader.readexactly(2))
            elif size == 127:
                (size,) = struct.unpack(">Q", await reader.readexactly(8))
            mask = await reader.readexactly(4) if b2 & 0x80 else b"\0" * 4
            body = bytearray(await reader.readexactly(size))
        except (asyncio.IncompleteReadError, ConnectionError):
            return None
        for i in range(size):
            body[i] ^= mask[i % 4]
        op = b1 & 0x0F
        if op == 8:
            return None
        if op in (1, 2):
            kind = op
        if op in (0, 1, 2):
            data += bytes(body)
            if b1 & 0x80:
                done, data = data, b""
                if kind == 1:
                    return json.loads(done)


def frame(op, payload):
    n = len(payload)
    head = bytes([0x80 | op])
    if n < 126:
        head += bytes([n])
    elif n < 65536:
        head += bytes([126]) + struct.pack(">H", n)
    else:
        head += bytes([127]) + struct.pack(">Q", n)
    return head + payload


async def send(writer, event):
    writer.write(frame(1, json.dumps(event).encode()))
    await writer.drain()


async def serve_ws(handler):
    async def accept(reader, writer):
        try:
            await handshake(reader, writer)
            await handler(reader, writer)
        except (ConnectionError, asyncio.IncompleteReadError):
            pass
        finally:
            writer.close()

    return await asyncio.start_server(accept, "127.0.0.1", 0)


# --- the fakes ---------------------------------------------------------------
class Fakes:
    def __init__(self, work):
        self.work = work
        self.llm, self.asr = [], []

    async def asr_socket(self, reader, writer):
        while (event := await recv(reader)) is not None:
            if event.get("type") != "input_audio_buffer.commit":
                continue
            text = self.asr.pop(0) if self.asr else "okay"
            append(f"{self.work}/asr.log", text)
            delta = "conversation.item.input_audio_transcription.delta"
            for part in (text[: len(text) // 2], text[len(text) // 2 :]):
                await send(writer, {"type": delta, "delta": part})
            done = "conversation.item.input_audio_transcription.completed"
            await send(writer, {"type": done, "transcript": text})

    async def tts_socket(self, reader, writer):
        text = ""
        while (event := await recv(reader)) is not None:
            if event.get("type") == "input.text":
                text += event["text"]
            elif event.get("type") == "input.done":
                break
        append(f"{self.work}/tts.log", json.dumps({"t": time.time(), "text": text}))
        sentences, last = [], 0
        for i, c in enumerate(text):
            if c in ".?":
                sentences.append(text[last : i + 1].strip())
                last = i + 1
        if text[last:].strip():
            sentences.append(text[last:].strip())
        for index, sentence in enumerate(sentences):
            start = {
                "type": "audio.start",
                "sentence_index": index,
                "sentence_text": sentence,
                "format": "pcm",
                "sample_rate": RATE,
            }
            await send(writer, start)
            # 60 ms of silence per character
            pcm = bytes(len(sentence) * 1440 * 2)
            for i in range(0, len(pcm), 4800):
                writer.write(frame(2, pcm[i : i + 4800]))
            await writer.drain()
            await send(writer, {"type": "audio.done", "sentence_index": index})
        await send(writer, {"type": "session.done"})

    def sse(self, reply):
        def chunk(delta, finish=None):
            return {"choices": [{"index": 0, "delta": delta, "finish_reason": finish}]}

        events = [chunk({"role": "assistant"})]
        if "tool" in reply:
            args = json.dumps(reply.get("args", {}))
            call = {
                "index": 0,
                "id": "call-1",
                "function": {"name": reply["tool"], "arguments": args},
            }
            events.append(chunk({"tool_calls": [call]}))
            finish = "tool_calls"
        else:
            text = reply.get("text", "")
            for i in range(0, len(text), 4):
                events.append(chunk({"content": text[i : i + 4]}))
            finish = "stop"
        events.append(chunk({}, finish))
        events.append({"choices": [], "usage": {"prompt_tokens": 100}})
        return "".join(f"data: {json.dumps(e)}\n\n" for e in events) + "data: [DONE]\n\n"

    async def http(self, reader, writer):
        try:
            head = (await reader.readuntil(b"\r\n\r\n")).decode("latin-1").lower()
            length = next(
                (int(h.split(":")[1]) for h in head.split("\r\n") if h.startswith("content-length")),
                0,
            )
            body = await reader.readexactly(length) if length else b""
            line = head.split("\r\n")[0]
            kind, out = "application/json", {}
            if line.startswith("get") and "/models" in line:
                out = {
                    "data": [
                        {
                            "id": "gufo",
                            "context_length": 8000,
                            "architecture": {"input_modalities": ["text"]},
                        }
                    ]
                }
            elif "/_llm" in line:
                self.llm.append(json.loads(body))
            elif "/_asr" in line:
                self.asr.append(json.loads(body)["text"])
            elif "/_state" in line:
                out = {"llm_pending": len(self.llm), "asr_pending": len(self.asr)}
            elif "/chat/completions" in line:
                request = json.loads(body)
                if request.get("max_tokens") == 1:
                    # the warm-up takes no script entry
                    out = {"choices": [{"message": {"role": "assistant", "content": "."}}]}
                else:
                    append(f"{self.work}/llm.log", json.dumps(request))
                    reply = self.llm.pop(0) if self.llm else {"text": ""}
                    out, kind = self.sse(reply), "text/event-stream"
            payload = (out if isinstance(out, str) else json.dumps(out)).encode()
            head = f"HTTP/1.1 200 X\r\ncontent-type: {kind}\r\ncontent-length: {len(payload)}\r\nconnection: close\r\n\r\n"
            writer.write(head.encode() + payload)
            await writer.drain()
        except (ConnectionError, asyncio.IncompleteReadError):
            pass
        finally:
            writer.close()


async def servers(work):
    fakes = Fakes(work)
    asr = await serve_ws(fakes.asr_socket)
    tts = await serve_ws(fakes.tts_socket)
    llm = await asyncio.start_server(fakes.http, "127.0.0.1", 0)
    ports = {"asr": asr, "tts": tts, "llm": llm}
    ports = {k: s.sockets[0].getsockname()[1] for k, s in ports.items()}
    with open(f"{work}/ports", "w", encoding="utf-8") as f:
        f.write(json.dumps(ports) + "\n")
    await asyncio.Event().wait()


# --- the capture and the playback command --------------------------------
def mic(work):
    """32 ms frames in real time; the speech of speech.pcm on each line of mic.cmd."""
    frame_bytes = 512 * 3 // 2 * 2
    with open(f"{work}/speech.pcm", "rb") as f:
        speech = f.read()
    seen, queue, out = 0, b"", sys.stdout.buffer
    nxt = time.monotonic()
    while True:
        try:
            with open(f"{work}/mic.cmd", encoding="utf-8") as f:
                lines = f.read().splitlines()
        except FileNotFoundError:
            lines = []
        for _ in lines[seen:]:
            queue += speech
        seen = len(lines)
        chunk, queue = queue[:frame_bytes], queue[frame_bytes:]
        out.write(chunk.ljust(frame_bytes, b"\0"))
        out.flush()
        nxt += 0.032
        time.sleep(max(0, nxt - time.monotonic()))


def play(work):
    """Reads PCM in real time; a kill of this process ends the log."""
    log = f"{work}/play.log"
    pid = os.getpid()
    append(log, json.dumps({"ev": "start", "pid": pid, "t": time.time()}))
    total = 0
    while chunk := sys.stdin.buffer.read(4800):
        total += len(chunk)
        append(log, json.dumps({"ev": "chunk", "pid": pid, "t": time.time(), "total": total}))
        time.sleep(len(chunk) / 2 / RATE)
    append(log, json.dumps({"ev": "eof", "pid": pid, "t": time.time(), "total": total}))


if __name__ == "__main__":
    mode, work = sys.argv[1:3]
    if mode == "servers":
        asyncio.run(servers(work))
    elif mode == "mic":
        mic(work)
    else:
        play(work)
