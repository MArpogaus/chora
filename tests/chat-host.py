#!/usr/bin/env python3
"""A host of the chat field for tests/chat.sh.

It does what chora-assistant does in a session. It starts
`chora-chat --input NAME` with a pipe and without CHORA_SOCKET, writes the
lines that it reads from a fifo to the field, logs the lines of the field,
calls `voice agent` for each `say`, and tells the field about the
questions of its helper.

Usage: chat-host.py CHORA_CHAT CHORACTL HELPER LOG FIFO PIDFILE
"""

import json
import os
import subprocess
import sys
import threading


def call_say(choractl, helper, text):
    """Send a text to the helper and say whether voice took it."""
    params = {"tool": "agent_say", "name": helper, "text": text}
    done = subprocess.run(
        [choractl, "--name", "chat-host", "call", "voice", "agent", json.dumps(params)],
        capture_output=True,
        text=True,
        check=False,
    )
    return done.returncode == 0 and not done.stdout.strip().startswith('"error')


def send(field, lock, line):
    """Write one line to the field."""
    with lock:
        field.stdin.write(json.dumps(line) + "\n")
        field.stdin.flush()


def pump_field(field, lock, choractl, helper, log):
    """Log the lines of the field and relay its says."""
    for line in field.stdout:
        log.write(line)
        log.flush()
        message = json.loads(line)
        if "say" in message and not call_say(choractl, helper, message["say"]):
            send(field, lock, {"say_failed": message["say"]})


def pump_fifo(field, lock, fifo):
    """Pass the lines of the fifo to the field."""
    while True:
        with open(fifo, encoding="utf-8") as lines:
            for line in lines:
                with lock:
                    field.stdin.write(line)
                    field.stdin.flush()


def pump_voice(field, lock, choractl, helper):
    """Tell the field about the questions of its helper."""
    events = subprocess.Popen(
        [choractl, "--name", "chat-host", "events", "voice.*"],
        stdout=subprocess.PIPE,
        text=True,
    )
    for line in events.stdout:
        event = json.loads(line)
        data = event.get("data") or {}
        name = event.get("event")
        if name == "voice.state" and str(data.get("name", "")).lower() == helper:
            send(field, lock, {"question": data.get("state") == "question"})
        elif name == "voice.confirm" and not data.get("done") and data.get("keys"):
            send(field, lock, {"keys": data["keys"]})


def main():
    """Start the field and the pumps."""
    chat, choractl, helper, log_path, fifo, pid_path = sys.argv[1:7]
    env = {k: v for k, v in os.environ.items() if k != "CHORA_SOCKET"}
    field = subprocess.Popen(
        [chat, "--input", helper],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
        env=env,
    )
    with open(pid_path, "w", encoding="utf-8") as pid_file:
        pid_file.write(str(field.pid))
    lock = threading.Lock()
    log = open(log_path, "a", encoding="utf-8")  # noqa: SIM115
    args = (field, lock)
    threads = [
        threading.Thread(target=pump_field, args=(*args, choractl, helper, log)),
        threading.Thread(target=pump_fifo, args=(*args, fifo), daemon=True),
        threading.Thread(target=pump_voice, args=(*args, choractl, helper), daemon=True),
    ]
    for thread in threads:
        thread.start()
    field.wait()


main()
