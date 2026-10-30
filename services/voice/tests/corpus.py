#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["aiohttp>=3.9,<4", "numpy>=2,<3"]
#
# [tool.uv]
# exclude-newer = "2026-10-29T00:00:00Z"
# ///
"""Write the replay corpus of CI (docs/voice.md, WP3) with the live TTS.

Usage: corpus.py TTS_URL [DIR]

DIR defaults to tests/data/corpus beside this script. Each recording is
a directory in the format of `record_dir`: mic.wav (24 kHz), events.jsonl
with the played pieces, and labels.txt. The user's voices are ryan and
vivian, the agent's voice is aiden; the agent's sound goes into mic.wav
as echo at a fixed level, as a residual of the echo canceller. The noise
comes from a fixed seed, so only the TTS changes from run to run.
played.wav stays out to keep the corpus small; the harness does not
read it.
"""

import asyncio
import json
import sys
import wave
from pathlib import Path

import aiohttp
import numpy as np

RATE = 24000
rng = np.random.default_rng(7)


async def tts(http: aiohttp.ClientSession, url: str, voice: str, text: str) -> np.ndarray:
    """Return the PCM of TEXT in VOICE."""
    pcm = b""
    async with http.ws_connect(url) as ws:
        config = {"model": "gufo", "voice": voice, "response_format": "pcm"}
        await ws.send_json(
            {"type": "session.config", **config, "language": "auto", "stream_audio": True}
        )
        await ws.send_json({"type": "input.text", "text": text})
        await ws.send_json({"type": "input.done"})
        while True:
            msg = await asyncio.wait_for(ws.receive(), 60)
            if msg.type == aiohttp.WSMsgType.BINARY:
                pcm += msg.data
                continue
            kind = json.loads(msg.data).get("type", "") if msg.type == aiohttp.WSMsgType.TEXT else ""
            if kind == "session.done" or "error" in kind or not kind:
                break
    return np.frombuffer(pcm, np.int16).astype(np.float64)


def bounds(speech: np.ndarray) -> tuple[float, float]:
    """Return the first and last loud 10 ms of SPEECH, in s."""
    frames = speech[: len(speech) // 240 * 240].reshape(-1, 240)
    level = np.sqrt((frames**2).mean(axis=1))
    loud = np.flatnonzero(level > 0.05 * level.max())
    return loud[0] / 100, (loud[-1] + 1) / 100


def scaled(speech: np.ndarray, rms: float) -> np.ndarray:
    """Return SPEECH at the level RMS over its loud part."""
    start, end = bounds(speech)
    part = speech[int(start * RATE) : int(end * RATE)]
    return speech * rms / np.sqrt((part**2).mean())


def room(seconds: float) -> np.ndarray:
    """Quiet room (RMS 30) and a fan: low noise of RMS 120."""
    n = int(seconds * RATE)
    fan = np.convolve(rng.normal(0, 1, n), np.ones(48) / 48, "same")
    return rng.normal(0, 30, n) + fan * 120 / fan.std()


def clicks(mic: np.ndarray, at: float, count: int) -> None:
    """Add keys of a keyboard: short loud clicks, 120 ms apart."""
    decay = np.exp(-np.arange(240) / 30)
    for k in range(count):
        i = int((at + 0.12 * k) * RATE)
        mic[i : i + 240] += rng.normal(0, 6000, 240) * decay


class Session:
    """A recording that grows: mic samples, played pieces and labels."""

    def __init__(self, seconds: float) -> None:
        self.mic = room(seconds)
        self.events: list[dict] = []
        self.labels: list[str] = []
        self.played = 0

    def say(self, at: float, speech: np.ndarray) -> tuple[float, float]:
        """The user speaks from AT; returns the labelled turn."""
        i = int(at * RATE)
        self.mic[i : i + len(speech)] += speech
        start, end = bounds(speech)
        self.labels.append(f"turn {at + start:.3f} {at + end:.3f}")
        return at + start, at + end

    def play(self, at: float, sound: np.ndarray, echo: float, barge: float | None = None) -> None:
        """The agent plays SOUND from AT; its echo has the level ECHO. A
        barge-in at BARGE stops it 0.4 s later, as the agent would."""
        n = len(sound) if barge is None else int((barge + 0.4 - at) * RATE)
        i = int(at * RATE)
        self.mic[i : i + n] += scaled(sound, echo)[:n]
        self.events.append({"at": i, "event": "play", "from": self.played, "len": n})
        self.played += n
        if barge is not None:
            self.labels.append(f"barge {barge:.3f}")

    def write(self, root: Path, name: str, note: str) -> None:
        out = root / name
        out.mkdir(parents=True, exist_ok=True)
        with wave.open(str(out / "mic.wav"), "wb") as wav:
            wav.setnchannels(1)
            wav.setsampwidth(2)
            wav.setframerate(RATE)
            wav.writeframes(np.clip(self.mic, -32768, 32767).astype(np.int16).tobytes())
        lines = "".join(json.dumps(e, separators=(",", ":")) + "\n" for e in self.events)
        (out / "events.jsonl").write_text(lines)
        header = f"# {note}\n# Written by tests/corpus.py; times in s of mic.wav.\n"
        (out / "labels.txt").write_text(header + "\n".join(self.labels) + "\n")
        print(name, f"{len(self.mic) / RATE:.2f} s", *self.labels, sep="\n  ")


async def main() -> None:
    """Write the corpus."""
    url = sys.argv[1]
    root = Path(sys.argv[2]) if sys.argv[2:] else Path(__file__).parent / "data/corpus"
    async with aiohttp.ClientSession() as http:
        agenda = scaled(await tts(http, url, "vivian", "Zeig mir bitte die Agenda."), 2500)
        stop = scaled(await tts(http, url, "ryan", "Stopp."), 2000)
        wait = scaled(await tts(http, url, "ryan", "Stopp, warte mal."), 2500)
        reply = await tts(
            http, url, "aiden", "Heute hast du drei Termine. Um zehn Uhr triffst du das Team."
        )
        more = await tts(http, url, "aiden", "Danach ist der Nachmittag frei.")

    # Two turns in a room with a fan and a keyboard.
    s = Session(1.0 + len(agenda) / RATE + 1.2 + len(stop) / RATE + 1.2)
    clicks(s.mic, 0.3, 4)
    _, end = s.say(1.0, agenda)
    clicks(s.mic, end + 0.3, 3)
    s.say(end + 1.2, stop)
    s.write(root, "turns-noise", "Two turns, a fan and keyboard clicks; no playback.")

    # The agent speaks, the user stops it; then a reply with louder echo
    # and no barge-in.
    at = 2.0
    assert 0.5 + len(reply) / RATE > at + 1, "the user speaks over the reply"
    s = Session(at + len(wait) / RATE + 1.0 + len(more) / RATE + 1.0)
    s.play(0.5, reply, 150, barge=at + bounds(wait)[0])
    _, end = s.say(at, wait)
    s.play(end + 0.8, more, 600)
    s.write(root, "echo-barge", "Echo of RMS 150, a barge-in, then echo of RMS 600 alone.")


asyncio.run(main())
