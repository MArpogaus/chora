#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["aiohttp>=3.9,<4", "numpy>=2,<3"]
#
# [tool.uv]
# exclude-newer = "2026-10-29T00:00:00Z"
# ///
"""Live check of the ASR and TTS servers, and the test clip of the VAD.

Usage: live.py TTS_URL ASR_URL [CLIP.wav]

It speaks a sentence with the TTS, prints the events of the TTS, sends
the audio to the ASR at 24 kHz and at 16 kHz, and prints the
transcripts. With CLIP.wav, it writes the 24 kHz test clip of
`vad::tests`: 1 s of quiet room, the sentence, 1.5 s of quiet room.
"""

import asyncio
import base64
import json
import sys
import time
import wave

import aiohttp
import numpy as np

TEXT = "Hallo Chora. Zeig mir bitte die Agenda für heute."
RATE = 24000


async def tts(http: aiohttp.ClientSession, url: str) -> bytes:
    """Return the PCM of TEXT and print the events of the TTS."""
    pcm = b""
    async with http.ws_connect(url) as ws:
        config = {"model": "gufo", "voice": "aiden", "response_format": "pcm"}
        await ws.send_json(
            {"type": "session.config", **config, "language": "auto", "stream_audio": True}
        )
        await ws.send_json({"type": "input.text", "text": TEXT})
        await ws.send_json({"type": "input.done"})
        while True:
            msg = await asyncio.wait_for(ws.receive(), 60)
            if msg.type == aiohttp.WSMsgType.BINARY:
                pcm += msg.data
                continue
            if msg.type != aiohttp.WSMsgType.TEXT:
                return pcm
            event = json.loads(msg.data)
            print("tts", {k: v for k, v in event.items() if k != "audio"})
            if event.get("type") == "session.done" or "error" in event.get("type", ""):
                return pcm


async def asr(http: aiohttp.ClientSession, url: str, pcm: bytes, rate: int) -> str:
    """Return the transcript of PCM at RATE, or the error of the ASR."""
    async with http.ws_connect(url) as ws:
        audio = {
            "format": {"type": "audio/pcm", "rate": rate},
            "transcription": {"model": "gufo", "language": "de"},
            "turn_detection": None,
        }
        await ws.send_json(
            {
                "type": "session.update",
                "session": {"type": "transcription", "audio": {"input": audio}},
            }
        )
        for i in range(0, len(pcm), rate * 2):
            chunk = base64.b64encode(pcm[i : i + rate * 2]).decode()
            await ws.send_json({"type": "input_audio_buffer.append", "audio": chunk})
        start = time.monotonic()
        await ws.send_json({"type": "input_audio_buffer.commit"})
        while True:
            event = json.loads((await asyncio.wait_for(ws.receive(), 30)).data)
            kind = event.get("type", "")
            if kind.endswith("completed"):
                return f"{time.monotonic() - start:.2f} s: {event.get('transcript', '')!r}"
            if "error" in kind or kind.endswith("failed"):
                return f"error: {event}"


async def main() -> None:
    """Run the checks."""
    tts_url, asr_url = sys.argv[1], sys.argv[2]
    async with aiohttp.ClientSession() as http:
        speech = np.frombuffer(await tts(http, tts_url), np.int16)
        print("asr 24 kHz:", await asr(http, asr_url, speech.tobytes(), RATE))
        # Two of each three samples: a rough 16 kHz copy, enough to see
        # whether the ASR takes the rate.
        rough = speech[: len(speech) // 3 * 3].reshape(-1, 3)[:, :2].ravel()
        print("asr 16 kHz:", await asr(http, asr_url, rough.tobytes(), 16000))
    if clip_path := sys.argv[3:]:
        rng = np.random.default_rng(1)

        def room(n: int) -> np.ndarray:
            return rng.normal(0, 30, n).astype(np.int16)  # RMS 30

        clip = np.concatenate([room(RATE), speech + room(len(speech)), room(RATE * 3 // 2)])
        with wave.open(clip_path[0], "wb") as out:
            out.setnchannels(1)
            out.setsampwidth(2)
            out.setframerate(RATE)
            out.writeframes(clip.tobytes())
        print(f"speech from 1.00 s to {1 + len(speech) / RATE:.2f} s of {len(clip) / RATE:.2f} s")


asyncio.run(main())
