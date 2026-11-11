#!/usr/bin/env python3
"""Starts the sessions of the tour: a headless chora with a look, its
companions, a stand-in wpctl, the fake voice agent. Every pid is kept, and
`stop` ends them by pid."""

import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time

REPO = "/home/user/chora"
BIN = f"{REPO}/target/debug"
SCRATCH = os.environ.get("TOUR_SCRATCH", "/tmp/claude-0/tour")
CONTENT = f"{REPO}/demo/content.py"

ENTRIES = [
    {"seq": 1, "role": "user", "text": "Please tidy the notes for Friday."},
    {"seq": 2, "role": "agent", "text": "I will go through your notes and group them by day."},
    {"seq": 3, "role": "tool", "text": "Read notes/friday.md"},
    {"seq": 4, "role": "agent", "text": "Friday has three items now. I would like to write the summary."},
]


def wpctl_stub(directory):
    path = os.path.join(directory, "wpctl")
    state = os.path.join(directory, "wpctl.state")
    with open(state, "w") as f:
        f.write("0.40 0\n")
    with open(path, "w") as f:
        f.write(
            """#!/bin/bash
state=%s
read -r vol mute <"$state"
case "$1" in
get-volume)
    if [ "$mute" = 1 ]; then echo "Volume: $vol [MUTED]"; else echo "Volume: $vol"; fi ;;
set-volume) vol=$(printf '%%s' "$3" | sed 's/%%$//'); case "$3" in *%%) vol=$(awk "BEGIN{printf \\"%%.2f\\", $vol/100}") ;; *) vol=$3 ;; esac; echo "$vol $mute" >"$state" ;;
set-mute) case "$3" in 1) mute=1 ;; 0) mute=0 ;; toggle) mute=$((1-mute)) ;; esac; echo "$vol $mute" >"$state" ;;
esac
"""
            % state
        )
    os.chmod(path, 0o755)


def programs_yaml(stub):
    items = [
        ("Notes", "notes", "notes"),
        ("wobbly.rs", "wobbly", f"code {REPO}/crates/chora-kit/src/wobbly.rs"),
        ("Chart", "chart", "palette"),
        ("Page", "page", "page"),
    ]
    out = "programs:\n"
    for title, script, args in items:
        path = os.path.join(stub, f"chora-{script}")
        with open(path, "w") as f:
            f.write(
                "#!/bin/sh\nexec foot --app-id=%s --title=%s -o pad=14x12 -o colors.background=1a1b26 "
                "-o colors.foreground=c0caf5 -o 'font=DejaVu Sans Mono:size=9' -o cursor.blink=no "
                "-e sh -c 'python3 -I %s %s; exec sleep infinity'\n" % (title, title, CONTENT, args)
            )
        os.chmod(path, 0o755)
        out += f"  - title: {title}\n    command: chora-{script}\n    icon: utilities-terminal\n"
    return out


class Launcher:
    def __init__(self, name, size="1280x720"):
        os.makedirs(SCRATCH, exist_ok=True)
        self.dir = tempfile.mkdtemp(prefix=f"{name}.", dir=SCRATCH)
        self.name = name
        self.size = size
        self.pids = []
        self.env = None
        self.core_env = {}

    def base_env(self):
        env = dict(os.environ)
        env["PATH"] = f"{self.dir}/stub:{BIN}:{env['PATH']}"
        env["CHORA_TEST_ACCEPT_INJECTED_KEYS"] = "1"
        env["CHORA_FRAME_BUDGET_US"] = "500000"
        env.pop("DBUS_SESSION_BUS_ADDRESS", None)
        env["XDG_CONFIG_HOME"] = f"{self.dir}/config"
        env["XDG_DATA_HOME"] = f"{self.dir}/data"
        env["XDG_STATE_HOME"] = f"{self.dir}/state"
        env["GALLIUM_DRIVER"] = "llvmpipe"
        env["LIBGL_ALWAYS_SOFTWARE"] = "1"
        egl = os.path.expanduser("~/.local/opt/mesa-egl")
        if os.path.isdir(egl):
            env["LD_LIBRARY_PATH"] = f"{egl}/usr/lib64:{env.get('LD_LIBRARY_PATH', '')}"
            env["__EGL_VENDOR_LIBRARY_FILENAMES"] = f"{egl}/usr/share/glvnd/egl_vendor.d/50_mesa.json"
        return env

    def spawn(self, argv, env, log, **kw):
        out = open(os.path.join(self.dir, log), "w")
        p = subprocess.Popen(argv, env=env, stdout=out, stderr=subprocess.STDOUT, start_new_session=True, cwd="/home/user/notes", **kw)
        self.pids.append(p.pid)
        return p

    def start(self, startup, flavor=None, test_flags="--test-injected-press-is-real", voice=True, companions=(), extra_config=None, xp=False, outputs=None):
        os.makedirs(f"{self.dir}/stub", exist_ok=True)
        os.makedirs(f"{self.dir}/config/chora", exist_ok=True)
        wpctl_stub(f"{self.dir}/stub")
        for name, text in (extra_config or {}).items():
            with open(f"{self.dir}/config/chora/{name}", "w") as f:
                f.write(text)
        env = self.base_env()
        if test_flags:
            env["CHORA_TEST_FLAGS"] = test_flags
        env["CHORA_LOG"] = f"{self.dir}/chora.log"
        bus_env = {}
        if xp:
            addr = f"unix:path={self.dir}/bus"
            self.spawn(["dbus-daemon", "--session", "--nofork", f"--address={addr}"], env, "dbus.log")
            env["DBUS_SESSION_BUS_ADDRESS"] = addr
            time.sleep(0.5)
        head = self.spawn([f"{REPO}/tests/headless-session.sh", outputs or self.size] + (["--", *startup] if startup else []), env, "headless.out")
        self.session_pid = head.pid
        out = f"{self.dir}/headless.out"
        for _ in range(240):
            if "CHORA_SOCKET=" in open(out).read():
                break
            time.sleep(0.25)
        else:
            raise RuntimeError("no session: " + open(out).read()[-500:])
        kv = dict(line.split("=", 1) for line in open(out).read().splitlines() if "=" in line)
        self.core_env = {
            "CHORA_SOCKET": kv["CHORA_SOCKET"],
            "XDG_RUNTIME_DIR": kv["XDG_RUNTIME_DIR"],
            "WAYLAND_DISPLAY": kv["WAYLAND_DISPLAY"],
        }
        env.update(self.core_env)
        self.env = env
        subprocess.run([f"{BIN}/choractl", "grant", "choractl", "input.inject", "methods.call", "methods.privileged", "screen.capture"], env=env, check=True, capture_output=True)
        if voice:
            self.start_voice(env)
        for argv in companions:
            subprocess.run([f"{BIN}/choractl", "run", "setsid", *argv], env=dict(env, CHORA_FLAVOR_DIR=f"{REPO}/flavors/{flavor}") if flavor else env, check=False, capture_output=True)
        return env

    def start_voice(self, env):
        entries = f"{self.dir}/entries.json"
        with open(entries, "w") as f:
            json.dump(ENTRIES, f)
        fifo = f"{self.dir}/voice.in"
        os.mkfifo(fifo)
        venv = dict(env)
        venv["FAKE_VOICE_ENTRIES"] = entries
        venv["FAKE_VOICE_KEYS"] = "1"
        log = open(f"{self.dir}/voice.log", "w")
        self.voice = subprocess.Popen(
            ["python3", "-I", f"{REPO}/tests/fake-voice.py", f"{self.dir}/claims"],
            env=venv,
            stdin=os.open(fifo, os.O_RDWR),
            stdout=log,
            stderr=subprocess.STDOUT,
            start_new_session=True,
        )
        self.pids.append(self.voice.pid)
        self.fifo = os.open(fifo, os.O_RDWR)

    def emit(self, event, data):
        os.write(self.fifo, f"{event} {json.dumps(data)}\n".encode())

    def stop(self):
        for pid in reversed(self.pids):
            try:
                os.kill(pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
        time.sleep(1.5)
        for pid in self.pids:
            try:
                os.kill(pid, signal.SIGKILL)
            except ProcessLookupError:
                pass


class SceneScript:
    """Runs one of demo/scene-*.sh and takes its session as the one of the tour."""

    def __init__(self, name, script, theme="light"):
        os.makedirs(SCRATCH, exist_ok=True)
        self.dir = tempfile.mkdtemp(prefix=f"{name}.", dir=SCRATCH)
        os.makedirs(f"{self.dir}/stub", exist_ok=True)
        wpctl_stub(f"{self.dir}/stub")
        env = dict(os.environ, CHORA_BIN=BIN, PATH=f"{self.dir}/stub:{os.environ['PATH']}")
        self.log = f"{self.dir}/scene.out"
        out = open(self.log, "w")
        self.proc = subprocess.Popen([f"{REPO}/demo/{script}", theme], env=env, stdout=out, stderr=subprocess.STDOUT, start_new_session=True)
        self.pids = [self.proc.pid]
        for _ in range(480):
            if "DEMO_READY=1" in open(self.log).read():
                break
            time.sleep(0.5)
        else:
            raise RuntimeError("scene did not stand: " + open(self.log).read()[-600:])
        kv = dict(line.split("=", 1) for line in open(self.log).read().splitlines() if "=" in line)
        self.env = dict(os.environ, PATH=f"{BIN}:{os.environ['PATH']}", CHORA_SOCKET=kv["CHORA_SOCKET"], XDG_RUNTIME_DIR=kv["XDG_RUNTIME_DIR"], WAYLAND_DISPLAY=kv["WAYLAND_DISPLAY"])

    def stop(self):
        try:
            os.kill(self.proc.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        time.sleep(2)
        try:
            os.kill(self.proc.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
