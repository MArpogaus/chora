#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["aiohttp>=3.9"]
# ///
"""Minimal pipeline to find where the live audio streaming stops.

agent2.py [--asr-url U] [--tts-url U] [--llm-url U] [--model NAME] ...

agent2.py tts [TEXT]   text -> TTS websocket -> speaker, logs every event
agent2.py asr [FILE]   PCM/WAV file -> ASR websocket -> prints every event
agent2.py turn [--rms N]  mic -> one turn -> speaker, exits (default)

Run each stage alone first. The stage that stays silent or errors is the
one to look at; the logs show what the server actually sends.
"""

import argparse
import array
import asyncio
import base64
import json
import math
import os
import sys

import aiohttp

# Defaults, overridable per call. gufo.sh serves all three models under
# the name "gufo", not "gufo-tts".
ASR_URL = os.environ.get("ASR_URL", "ws://localhost:8881/v1/realtime?intent=transcription")
TTS_URL = os.environ.get("TTS_URL", "http://localhost:8882")
LLM_URL = os.environ.get("LLM_URL", "http://localhost:8888/v1")
MODEL = os.environ.get("MODEL", "gufo")
RATE = 24000
FRAME_BYTES = RATE * 20 // 1000 * 2


def log(*args):
    print(*args, file=sys.stderr, flush=True)


def rms(frame):
    samples = array.array("h", frame)
    return math.sqrt(sum(s * s for s in samples) / len(samples))


async def tts_speak(http, text):
    """Streams text through the TTS websocket to the speakers."""
    async with http.get(f"{TTS_URL}/v1/audio/voices") as r:
        voice = (await r.json())["data"][0]["id"]
    # pw-play sniffs headers via libsndfile and chokes on headerless
    # pipes; aplay has an explicit raw type. Same for arecord below.
    player = await asyncio.create_subprocess_exec(
        "aplay", "-q", "-r", str(RATE), "-c", "1", "-f", "S16_LE", "-t",
        "raw", "-", stdin=asyncio.subprocess.PIPE,
    )
    url = TTS_URL.replace("http", "ws", 1) + "/v1/audio/speech/stream"
    n_bytes = 0
    try:
        async with http.ws_connect(url) as ws:
            # The server rejects stream_audio without response_format pcm.
            await ws.send_json({"type": "session.config", "model": MODEL,
                                "voice": voice, "response_format": "pcm",
                                "stream_audio": True})
            await ws.send_json({"type": "input.text", "text": text})
            await ws.send_json({"type": "input.done"})
            while True:
                try:
                    msg = await asyncio.wait_for(ws.receive(), 15)
                except asyncio.TimeoutError:
                    log("no message for 15 s, giving up")
                    break
                chunk = None
                if msg.type == aiohttp.WSMsgType.BINARY:
                    chunk = msg.data
                elif msg.type == aiohttp.WSMsgType.TEXT:
                    event = json.loads(msg.data)
                    log("tts event:", msg.data[:200])
                    # Some servers put the pcm in a text event as base64.
                    b64 = next((event[k] for k in ("audio", "delta", "data")
                                if isinstance(event.get(k), str)), None)
                    if b64:
                        chunk = base64.b64decode(b64)
                if chunk:
                    n_bytes += len(chunk)
                    player.stdin.write(chunk)
                    await player.stdin.drain()
                elif msg.type not in (aiohttp.WSMsgType.BINARY,
                                      aiohttp.WSMsgType.TEXT):
                    log("tts closed:", msg.type)
                    break
    finally:
        player.stdin.close()
        await player.wait()
    log(f"played {n_bytes / 48000:.2f} s of audio")


async def asr_transcribe(http, pcm, dump_events):
    """Sends raw PCM to the ASR websocket, returns the transcript."""
    async with http.ws_connect(ASR_URL) as ws:
        await ws.send_json({
            "type": "session.update",
            "session": {
                "type": "transcription",
                "audio": {"input": {
                    "format": {"type": "audio/pcm", "rate": RATE},
                    "transcription": {"model": MODEL},
                    "turn_detection": None,
                }},
            },
        })
        for i in range(0, len(pcm), FRAME_BYTES):
            await ws.send_json({"type": "input_audio_buffer.append",
                                "audio": base64.b64encode(pcm[i:i + FRAME_BYTES]).decode()})
        await ws.send_json({"type": "input_audio_buffer.commit"})
        transcript = ""
        while True:
            try:
                msg = await asyncio.wait_for(ws.receive(), 25)
            except asyncio.TimeoutError:
                log("no message for 10 s, giving up")
                break
            if msg.type != aiohttp.WSMsgType.TEXT:
                log("asr non-text:", msg.type)
                break
            event = json.loads(msg.data)
            if dump_events:
                log("asr event:", msg.data[:200])
            kind = event.get("type", "")
            if kind.endswith("transcription.completed"):
                transcript = event.get("transcript", "").strip()
                break
            elif kind.endswith("error") or "error" in kind:
                break
        return transcript


async def mic():
    return await asyncio.create_subprocess_exec(
        "arecord", "-q", "-r", str(RATE), "-c", "1", "-f", "S16_LE", "-t",
        "raw", "-", stdout=asyncio.subprocess.PIPE,
    )


async def run_levels(http, args):
    """Live mic level meter: pick an --rms the speech beats and silence
    does not. Ctrl-C to stop."""
    mic_proc = await mic()
    peak = 0.0
    try:
        while True:
            frame = await mic_proc.stdout.readexactly(FRAME_BYTES)
            level = rms(frame)
            peak = max(peak, level)
            print(f"\rlevel {level:7.0f} peak {peak:7.0f} "
                  + "#" * min(40, int(level / 500)), end="", flush=True)
    except (KeyboardInterrupt, asyncio.IncompleteReadError):
        print()
    finally:
        mic_proc.kill()


async def one_turn(http, rms_speech):
    """Records one utterance from the mic and answers it."""
    pcm = []
    quiet = 0
    speaking = False
    log(f"speak (above {rms_speech}, ends after 0.7 s of silence)...")
    mic_proc = await mic()
    try:
        n = 0
        while True:
            frame = await mic_proc.stdout.readexactly(FRAME_BYTES)
            level = rms(frame)
            if n == 0:
                if level > rms_speech:
                    print("waiting for silence {level:.0f}")
                    os.sleep(10) # BUG: fix loudes
                    continue
            n += 1
            if n % 50 == 1:
                log(f"level {level:.0f}")
            loud = level >= rms_speech
            quiet = 0 if loud else quiet + 1
            speaking = speaking or loud
            if speaking:
                pcm.append(frame)
            if speaking and quiet * 20 >= 700:
                break
    finally:
        mic_proc.kill()
    said = await asr_transcribe(http, b"".join(pcm), dump_events=False)
    log(f"you said: {said}")
    if not said:
        return
    body = {"model": MODEL, "stream": False,
            "messages": [{"role": "system",
                          "content": "Answer in one short sentence."},
                         {"role": "user", "content": said}]}
    async with http.post(f"{LLM_URL}/chat/completions", json=body) as r:
        answer = (await r.json())["choices"][0]["message"]["content"]
    log(f"answer: {answer}")
    await tts_speak(http, answer)


async def run_tts(http, args):
    await tts_speak(http, " ".join(args.text) or "Hallo, der Sprachausgabe Test laeuft.")


async def run_asr(http, args):
    pcm = open(args.file, "rb").read()
    if pcm[:4] == b"RIFF":
        pcm = pcm[44:]
    log(f"transcript: {await asr_transcribe(http, pcm, dump_events=True)}")


async def run_turn(http, args):
    await one_turn(http, args.rms)


async def main():
    global ASR_URL, TTS_URL, LLM_URL, MODEL
    p = argparse.ArgumentParser(description=__doc__,
                                formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--asr-url", default=ASR_URL, help="ASR realtime websocket")
    p.add_argument("--tts-url", default=TTS_URL, help="TTS server, http turns into ws")
    p.add_argument("--llm-url", default=LLM_URL, help="OpenAI compatible LLM server")
    p.add_argument("--model", default=MODEL, help="served name of all three models")
    sub = p.add_subparsers(dest="command")
    tts = sub.add_parser("tts", help="text -> TTS websocket -> speaker")
    tts.add_argument("text", nargs="*", help="what to say")
    tts.set_defaults(func=run_tts)
    asr = sub.add_parser("asr", help="PCM/WAV file -> ASR -> transcript")
    asr.add_argument("file", nargs="?", default="speech.pcm")
    asr.set_defaults(func=run_asr)
    turn = sub.add_parser("turn", help="mic -> one turn -> speaker (default)")
    turn.add_argument("--rms", type=int, default=int(os.environ.get("RMS_SPEECH", 5000)),
                      help="level that counts as speech, tune to the mic")
    turn.set_defaults(func=run_turn)
    sub.add_parser("levels", help="live mic level meter, Ctrl-C to stop").set_defaults(
        func=run_levels)
    args = p.parse_args()
    if args.command is None:
        args.func = run_turn
        args.rms = int(os.environ.get("RMS_SPEECH", 5000))
    ASR_URL, TTS_URL, LLM_URL, MODEL = (args.asr_url, args.tts_url,
                                        args.llm_url, args.model)
    async with aiohttp.ClientSession() as http:
        await args.func(http, args)


if __name__ == "__main__":
    asyncio.run(main())
