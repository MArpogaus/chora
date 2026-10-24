#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["aiohttp>=3.9"]
# ///
"""The voice agent of milestone M6.

Microphone -> Qwen3-ASR -> LLM -> Qwen3-TTS -> speaker, in a loop that
stays running. aplay and arecord do the audio (they take an explicit raw
format; pw-play chokes on headerless pipes) at 24 kHz mono PCM16, the
format of both Gufo audio servers. All three models are served under the
name "gufo" by services/voice/systemd/*.container (or ad hoc by
services/voice/gufo.sh). A level threshold finds the turns; speech while
the agent talks (barge-in) stops the answer and starts the next turn.

Usage:
  agent.py                 the conversation loop
  agent.py levels          live mic level meter, pick RMS_SPEECH
  agent.py tts TEXT        text -> TTS -> speaker
  agent.py asr [FILE]      PCM/WAV file -> ASR -> transcript
  agent.py test            run the unit checks

Environment: ASR_URL, TTS_URL, LLM_URL, MODEL, VOICE, LANGUAGE,
RMS_SPEECH, RMS_BARGE_IN, SILENCE_MS, see CONFIG.
"""

import argparse
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
    "LLM_URL": "http://localhost:8888/v1",
    "MODEL": "gufo",  # one served name for all three models
    "VOICE": "",  # empty: the first voice of /v1/audio/voices
    "LANGUAGE": "",  # empty: the ASR detects it
    "RMS_SPEECH": "5000",  # level of speech, tune with: agent.py levels
    "RMS_BARGE_IN": "8000",  # level for barge-in while the agent talks
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


def log(*args):
    print(*args, file=sys.stderr, flush=True)


def rms(frame):
    samples = array.array("h", frame)
    return math.sqrt(sum(s * s for s in samples) / len(samples))


class Turns:
    """Cuts a stream of levels into turns.

    feed() returns "start" on the first loud frame and "end" after
    SILENCE_MS of quiet frames, else None.
    """

    # ponytail: level threshold, no VAD model; Silero or the webrtc VAD
    # once noise or music start turns by themselves.
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


async def mic():
    return await asyncio.create_subprocess_exec(
        "arecord", "-q", "-r", str(RATE), "-c", "1", "-f", "S16_LE", "-t",
        "raw", "-", stdout=asyncio.subprocess.PIPE,
    )


async def player():
    return await asyncio.create_subprocess_exec(
        "aplay", "-q", "-r", str(RATE), "-c", "1", "-f", "S16_LE", "-t",
        "raw", "-", stdin=asyncio.subprocess.PIPE,
    )


async def asr_transcribe(http, pcm):
    """Sends raw PCM to the ASR websocket, returns the transcript."""
    async with http.ws_connect(CFG["ASR_URL"]) as ws:
        transcription = {"model": CFG["MODEL"]}
        if CFG["LANGUAGE"]:
            transcription["language"] = CFG["LANGUAGE"]
        await ws.send_json({
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
        for i in range(0, len(pcm), FRAME_BYTES):
            await ws.send_json({
                "type": "input_audio_buffer.append",
                "audio": base64.b64encode(pcm[i:i + FRAME_BYTES]).decode(),
            })
        await ws.send_json({"type": "input_audio_buffer.commit"})
        transcript = ""
        while True:
            try:
                msg = await asyncio.wait_for(ws.receive(), 25)
            except asyncio.TimeoutError:
                log("asr: no message for 25 s, giving up")
                break
            if msg.type != aiohttp.WSMsgType.TEXT:
                log("asr: non-text message", msg.type)
                break
            event = json.loads(msg.data)
            kind = event.get("type", "")
            if kind.endswith("transcription.completed"):
                transcript = event.get("transcript", "").strip()
                break
            elif "error" in kind:
                log("asr:", msg.data[:200])
                break
        return transcript


async def tts_speak(http, text, play_from=None):
    """Streams text through the TTS websocket to the speakers.

    play_from is an optional asyncio.Queue of further text deltas, for a
    reply that is still being generated; it ends with None.
    """
    async with http.get(f"{CFG['TTS_URL']}/v1/audio/voices") as r:
        voice = CFG["VOICE"] or (await r.json())["data"][0]["id"]
    play = await player()
    url = CFG["TTS_URL"].replace("http", "ws", 1) + "/v1/audio/speech/stream"
    spoken = 0
    try:
        async with http.ws_connect(url) as ws:
            # The server rejects stream_audio without response_format pcm.
            await ws.send_json({
                "type": "session.config", "model": CFG["MODEL"],
                "voice": voice, "response_format": "pcm",
                "stream_audio": True, "split_granularity": "sentence",
            })

            async def feed():
                if play_from is None:
                    await ws.send_json({"type": "input.text", "text": text})
                else:
                    while True:
                        delta = await play_from.get()
                        if delta is None:
                            break
                        await ws.send_json({"type": "input.text",
                                            "text": delta})
                await ws.send_json({"type": "input.done"})

            feeder = asyncio.create_task(feed())
            try:
                while True:
                    timeout = 1.5 if feeder.done() else None
                    msg = await asyncio.wait_for(ws.receive(), timeout)
                    if msg.type == aiohttp.WSMsgType.BINARY:
                        play.stdin.write(msg.data)
                        await play.stdin.drain()
                        spoken += len(msg.data)
                    elif msg.type == aiohttp.WSMsgType.TEXT:
                        event = json.loads(msg.data)
                        # Some servers put the pcm in a text event as base64.
                        b64 = next((event[k] for k in
                                    ("audio", "delta", "data")
                                    if isinstance(event.get(k), str)), None)
                        if b64:
                            chunk = base64.b64decode(b64)
                            play.stdin.write(chunk)
                            await play.stdin.drain()
                            spoken += len(chunk)
                    else:
                        break
            except asyncio.TimeoutError:
                pass
            finally:
                feeder.cancel()
        play.stdin.close()
        await play.wait()
    finally:
        if play.returncode is None:
            play.kill()
    log(f"played {spoken / 2 / RATE:.2f} s of audio")


async def llm_stream(http, history):
    """Yields the text deltas of a streamed chat completion."""
    body = {"model": CFG["MODEL"], "messages": history, "stream": True}
    async with http.post(f"{CFG['LLM_URL']}/chat/completions", json=body) as r:
        r.raise_for_status()
        async for line in r.content:
            line = line.decode().strip()
            if not line.startswith("data: ") or line == "data: [DONE]":
                continue
            delta = json.loads(line[6:])["choices"][0]["delta"]
            if delta.get("content"):
                yield delta["content"]


async def reply(http, history):
    """Speaks the answer of the LLM to the last turn of history.

    A cancel (barge-in, Ctrl-C) kills the player and keeps the part of
    the answer that was generated, so that the history stays true.
    """
    text = ""
    deltas = asyncio.Queue()

    async def generate():
        nonlocal text
        async for delta in llm_stream(http, history):
            text += delta
            await deltas.put(delta)
        await deltas.put(None)

    generator = asyncio.create_task(generate())
    try:
        await tts_speak(http, "", play_from=deltas)
    finally:
        generator.cancel()
        if text:
            history.append({"role": "assistant", "content": text})
    log(f"agent: {text}")


async def run_loop(http, args):
    """Records turn after turn and answers each one, until Ctrl-C."""
    turns = Turns(int(CFG["SILENCE_MS"]))
    history = [{"role": "system", "content": SYSTEM}]
    talking = None
    # 300 ms before the first loud frame, for the onset of the word
    preroll_frames = 3000 // FRAME_MS
    preroll = []
    print("listening", file=sys.stderr)
    while True:
        pcm = []
        level = 0
        m = await mic()
        try:
            while True:
                frame = await m.stdout.readexactly(FRAME_BYTES)
                busy = talking is not None and not talking.done()
                threshold = int(CFG["RMS_BARGE_IN" if busy else "RMS_SPEECH"])
                level = rms(frame)
                event = turns.feed(level, threshold)
                if event == "start":
                    if busy:
                        talking.cancel()
                        log("barge-in")
                    pcm = preroll[:]
                    preroll.clear()
                if turns.speaking or event == "end":
                    pcm.append(frame)
                else:
                    preroll = [*preroll[-preroll_frames + 1:], frame]
                if event == "end":
                    break
        finally:
            m.kill()
        said = await asr_transcribe(http, b"".join(pcm))
        log(f"you said: {said}")
        if not said:
            continue
        history.append({"role": "user", "content": said})
        history[:] = history[:1] + history[-13:]  # keep the last 6 turns
        talking = asyncio.create_task(reply(http, history))


async def run_tts(http, args):
    await tts_speak(http, " ".join(args.text) or "Hallo, der Test laeuft.")


async def run_asr(http, args):
    pcm = open(args.file, "rb").read()
    if pcm[:4] == b"RIFF":
        pcm = pcm[44:]
    log(f"transcript: {await asr_transcribe(http, pcm)}")


async def run_levels(http, args):
    """Live mic meter: pick an RMS_SPEECH that speech beats and
    silence does not. Ctrl-C to stop."""
    m = await mic()
    peak = 0.0
    try:
        while True:
            frame = await m.stdout.readexactly(FRAME_BYTES)
            level = rms(frame)
            peak = max(peak, level)
            print(f"\rlevel {level:7.0f} peak {peak:7.0f} "
                  + "#" * min(40, int(level / 500)), end="", flush=True)
    except (KeyboardInterrupt, asyncio.IncompleteReadError):
        print()
    finally:
        m.kill()


def test():
    turns = Turns(silence_ms=100)
    levels = [0, 900, 900, 0, 0, 0, 0, 0, 900, 0]
    events = [turns.feed(level, 600) for level in levels]
    assert events == [None, "start", None, None, None, None, None, "end",
                      "start", None], events
    loud = array.array("h", [1000, -1000] * 4).tobytes()
    assert rms(loud) == 1000
    print("ok")


async def main():
    p = argparse.ArgumentParser(description=__doc__,
                                formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest="command")
    tts = sub.add_parser("tts", help="text -> TTS -> speaker")
    tts.add_argument("text", nargs="*")
    tts.set_defaults(func=run_tts)
    asr = sub.add_parser("asr", help="PCM/WAV file -> ASR -> transcript")
    asr.add_argument("file", nargs="?", default="speech.pcm")
    asr.set_defaults(func=run_asr)
    sub.add_parser("levels", help="live mic level meter").set_defaults(
        func=run_levels)
    sub.add_parser("run", help="the conversation loop (default)").set_defaults(
        func=run_loop)
    args = p.parse_args()
    if args.command is None:
        args.func = run_loop
    async with aiohttp.ClientSession() as http:
        await args.func(http, args)


if __name__ == "__main__":
    if sys.argv[1:] == ["test"]:
        test()
    else:
        asyncio.run(main())
