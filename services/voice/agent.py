#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["aiohttp>=3.9"]
# ///
"""A prototype of the voice agent of milestone M6, without Pipecat.

Microphone -> Qwen3-ASR -> LLM -> Qwen3-TTS -> speaker. PipeWire does the
audio (pw-record, pw-play) at 24 kHz mono PCM16, the format of both Gufo
audio servers. A level threshold finds the turns. Speech while the agent
talks (barge-in) stops the audio and cancels the LLM request.

The servers come from services/voice/gufo.sh (asr 8881, tts 8882) and a
llama-server or 'gufo serve llm' for the LLM. The environment sets the
URLs and the knobs, see CONFIG.

Usage: services/voice/agent.py [test]
"""

import array
import asyncio
import base64
import json
import math
import os
import sys
import time

import aiohttp

CONFIG = {
    "ASR_URL": "ws://localhost:8881/v1/realtime?intent=transcription",
    "TTS_URL": "http://localhost:8882",
    "LLM_URL": "http://localhost:8080/v1",
    "LLM_MODEL": "gufo",
    "VOICE": "",  # empty: the first voice of /v1/audio/voices
    "LANGUAGE": "",  # empty: the ASR detects it
    "RMS_SPEECH": "600",  # level of speech, tune to the microphone
    "RMS_BARGE_IN": "2000",  # level for barge-in while the agent talks
    "SILENCE_MS": "500",  # end of turn after this much silence
}
CFG = {k: os.environ.get(k, v) for k, v in CONFIG.items()}

RATE = 24000
FRAME_MS = 20
FRAME_BYTES = RATE * FRAME_MS // 1000 * 2
SYSTEM = (
    "You are the voice of a desktop. Answer in one to three short "
    "sentences, in the language of the user, without markdown."
)


def rms(frame):
    samples = array.array("h", frame)
    return math.sqrt(sum(s * s for s in samples) / len(samples))


class Turns:
    """Cuts a stream of frames into turns by level.

    feed() returns "start" on the first loud frame, "end" after
    SILENCE_MS of quiet frames, or None.
    """

    # ponytail: level threshold, no VAD model; Silero or the webrtc VAD
    # if noise or music start turns.
    def __init__(self, silence_ms):
        self.silence_frames = silence_ms // FRAME_MS
        self.speaking = False
        self.quiet = 0

    def feed(self, level, threshold):
        if level >= threshold:
            self.quiet = 0
            if not self.speaking:
                self.speaking = True
                return "start"
        elif self.speaking:
            self.quiet += 1
            if self.quiet >= self.silence_frames:
                self.speaking = False
                return "end"
        return None


def log(t0, what):
    print(f"[{(time.monotonic() - t0) * 1000:6.0f} ms] {what}", file=sys.stderr)


async def llm_stream(http, history):
    """Yields the text deltas of a streamed chat completion."""
    body = {"model": CFG["LLM_MODEL"], "messages": history, "stream": True}
    async with http.post(f"{CFG['LLM_URL']}/chat/completions", json=body) as r:
        r.raise_for_status()
        async for line in r.content:
            line = line.decode().strip()
            if not line.startswith("data: ") or line == "data: [DONE]":
                continue
            delta = json.loads(line[6:])["choices"][0]["delta"]
            if delta.get("content"):
                yield delta["content"]


async def reply(http, voice, history, t0):
    """Speaks the answer of the LLM to the last turn of history."""
    player = await asyncio.create_subprocess_exec(
        "pw-play", "--rate", str(RATE), "--channels", "1",
        "--format", "s16", "-", stdin=asyncio.subprocess.PIPE,
    )
    url = f"{CFG['TTS_URL']}/v1/audio/speech/stream".replace("http", "ws", 1)
    text = ""
    try:
        async with http.ws_connect(url) as tts:
            await tts.send_json({
                "type": "session.config", "model": "gufo-tts", "voice": voice,
                "response_format": "pcm", "stream_audio": True,
                "split_granularity": "sentence",
            })

            async def feed():
                nonlocal text
                async for delta in llm_stream(http, history):
                    if not text:
                        log(t0, "first LLM token")
                    text += delta
                    await tts.send_json({"type": "input.text", "text": delta})
                await tts.send_json({"type": "input.done"})
                log(t0, f"LLM: {text}")

            feeder = asyncio.create_task(feed())
            first = True
            try:
                # ponytail: the end of the reply is the end of the input
                # plus a second without a message, since the docs name no
                # final event; use it once the server shows one.
                while True:
                    timeout = 1.0 if feeder.done() else None
                    msg = await asyncio.wait_for(tts.receive(), timeout)
                    if msg.type == aiohttp.WSMsgType.BINARY:
                        if first:
                            log(t0, "first audio")
                            first = False
                        player.stdin.write(msg.data)
                        await player.stdin.drain()
                    elif msg.type == aiohttp.WSMsgType.TEXT:
                        event = json.loads(msg.data)
                        if event.get("type") not in ("audio.start", "audio.done"):
                            print(f"tts: {event}", file=sys.stderr)
                    else:
                        break
            except TimeoutError:
                pass
            finally:
                feeder.cancel()
        player.stdin.close()
        await player.wait()
    finally:
        if player.returncode is None:
            player.kill()
        # A barge-in keeps the part of the answer that was spoken.
        if text:
            history.append({"role": "assistant", "content": text})


async def main():
    async with aiohttp.ClientSession() as http:
        async with http.get(f"{CFG['TTS_URL']}/v1/audio/voices") as r:
            voices = await r.json()
        print(f"voices: {voices}", file=sys.stderr)
        voice = CFG["VOICE"] or voices["voices"][0]["name"]

        asr = await http.ws_connect(CFG["ASR_URL"])
        transcription = {"model": "gufo-asr"}
        if CFG["LANGUAGE"]:
            transcription["language"] = CFG["LANGUAGE"]
        await asr.send_json({
            "type": "session.update",
            "session": {
                "type": "transcription",
                "audio": {"input": {
                    "format": {"type": "audio/pcm", "rate": RATE},
                    "transcription": transcription,
                    "turn_detection": None,
                }},
            },
        })

        mic = await asyncio.create_subprocess_exec(
            "pw-record", "--rate", str(RATE), "--channels", "1",
            "--format", "s16", "-", stdout=asyncio.subprocess.PIPE,
        )
        history = [{"role": "system", "content": SYSTEM}]
        turns = Turns(int(CFG["SILENCE_MS"]))
        # 300 ms before the first loud frame, for the onset of the word
        preroll = []
        talking = None
        t0 = time.monotonic()
        print("listening", file=sys.stderr)

        async def transcripts():
            nonlocal talking
            async for msg in asr:
                event = json.loads(msg.data)
                kind = event.get("type", "")
                if kind.endswith("transcription.completed"):
                    said = event.get("transcript", "").strip()
                    log(t0, f"ASR: {said}")
                    if said:
                        history.append({"role": "user", "content": said})
                        talking = asyncio.create_task(reply(http, voice, history, t0))
                elif "error" in kind:
                    print(f"asr: {event}", file=sys.stderr)

        reader = asyncio.create_task(transcripts())
        while True:
            frame = await mic.stdout.readexactly(FRAME_BYTES)
            busy = talking is not None and not talking.done()
            threshold = int(CFG["RMS_BARGE_IN" if busy else "RMS_SPEECH"])
            event = turns.feed(rms(frame), threshold)
            if event == "start":
                t0 = time.monotonic()
                if busy:
                    talking.cancel()
                    log(t0, "barge-in")
                for f in preroll:
                    await asr.send_json({"type": "input_audio_buffer.append",
                                         "audio": base64.b64encode(f).decode()})
                preroll.clear()
            if turns.speaking or event == "end":
                await asr.send_json({"type": "input_audio_buffer.append",
                                     "audio": base64.b64encode(frame).decode()})
            else:
                preroll = [*preroll[-14:], frame]
            if event == "end":
                t0 = time.monotonic()
                log(t0, "end of turn")
                await asr.send_json({"type": "input_audio_buffer.commit"})
            if reader.done():
                reader.result()


def test():
    turns = Turns(silence_ms=100)
    levels = [0, 900, 900, 0, 0, 0, 0, 0, 900, 0]
    events = [turns.feed(level, 600) for level in levels]
    assert events == [None, "start", None, None, None, None, None, "end",
                      "start", None], events
    loud = array.array("h", [1000, -1000] * 4).tobytes()
    assert rms(loud) == 1000
    print("ok")


if __name__ == "__main__":
    if sys.argv[1:] == ["test"]:
        test()
    else:
        asyncio.run(main())
