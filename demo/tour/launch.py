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

REPO = os.environ.get("TOUR_REPO", "/home/user/chora")
BIN = f"{REPO}/target/debug"
SCRATCH = os.environ.get("TOUR_SCRATCH", "/tmp/claude-0/tour")
CONTENT = f"{REPO}/demo/content.py"

ENTRIES = [
    {"seq": 1, "role": "user", "text": "Please tidy the notes for Friday."},
    {"seq": 2, "role": "agent", "text": "I will go through your notes and group them by day."},
    {"seq": 3, "role": "tool", "text": "Read notes/friday.md"},
    {"seq": 4, "role": "agent", "text": "Friday has three items now. I would like to write the summary."},
]


NOTICE_SECONDS = 600


def notify_config(position="bottom_right"):
    """The chora-notify.yaml of every session of the tour, in one place. chora-notify expires a card
    on the wall clock (default 10 s) and `choractl record` steps the scene clock (a frame takes
    seconds of wall time), so a card was gone before its frames were reached (tour r1 F-1, rt-2).
    This keeps it for NOTICE_SECONDS. Notify on the scene clock is the product fix (backlog)."""
    return f"position: {position}\nshortest: {NOTICE_SECONDS}\nlongest: {NOTICE_SECONDS}\n"


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
set-volume) last=${@: -1}
    case "$last" in
    *%%+) vol=$(awk "BEGIN{printf \\"%%.2f\\", $vol+${last%%%%?+}/100}") ;;
    *%%-) vol=$(awk "BEGIN{printf \\"%%.2f\\", $vol-${last%%%%?-}/100}") ;;
    *%%) vol=$(awk "BEGIN{printf \\"%%.2f\\", ${last%%%%?}/100}") ;;
    *) vol=$last ;;
    esac
    echo "$vol $mute" >"$state" ;;
set-mute) case "$3" in 1) mute=1 ;; 0) mute=0 ;; toggle) mute=$((1-mute)) ;; esac; echo "$vol $mute" >"$state" ;;
esac
"""
            % state
        )
    os.chmod(path, 0o755)


def programs_yaml(stub):
    items = [
        ("Editor", "editor", "nano /home/user/notes/friday.md"),
        ("Notes", "notes", "python3 -I %s notes" % CONTENT),
        ("Chart", "chart", "python3 -I %s palette" % CONTENT),
        ("Page", "page", "python3 -I %s page" % CONTENT),
    ]
    out = "programs:\n"
    for title, script, command in items:
        path = os.path.join(stub, f"chora-{script}")
        with open(path, "w") as f:
            f.write(
                "#!/bin/sh\nexec foot --app-id=%s --title=%s -o pad=14x12 -o colors.background=1a1b26 "
                "-o colors.foreground=c0caf5 -o 'font=DejaVu Sans Mono:size=9' -o cursor.blink=no "
                "-e sh -c '%s; exec sleep infinity'\n" % (title, title, command)
            )
        os.chmod(path, 0o755)
        out += f"  - title: {title}\n    command: chora-{script}\n    icon: utilities-terminal\n"
    return out


FRIDAY = "# Friday\n\n- Release notes for 0.9 (Anna, by noon)\n- Review the cube merge request\n- Design review at 15:30\n"


class Launcher:
    def __init__(self, name, size="1280x720"):
        os.makedirs(SCRATCH, exist_ok=True)
        self.dir = tempfile.mkdtemp(prefix=f"{name}.", dir=SCRATCH)
        os.makedirs("/home/user/notes", exist_ok=True)
        with open("/home/user/notes/friday.md", "w") as f:
            f.write(FRIDAY)
        os.makedirs(f"{self.dir}/data/nano", exist_ok=True)
        self.name = name
        self.size = size
        self.pids = []
        self.env = None
        self.core_env = {}

    def base_env(self):
        env = dict(os.environ)
        env["PATH"] = f"{self.dir}/stub:{BIN}:{self.path_without_writer(env['PATH'])}"
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

    def path_without_writer(self, path):
        """PATH without the writers: every PATH entry that resolves to /usr/bin (/bin is a link
        to it) is replaced by one link farm that lacks lowriter and mousepad. This container has
        LibreOffice, which opens no window in the headless session, so Notepad would try it first
        and say nothing (tour v7 F-1: /bin was left in PATH and leaked it). Without a GUI editor
        Notepad shows its honest notice. Asserts the leak is gone."""
        farm = f"{self.dir}/usrbin"
        if not os.path.isdir(farm):
            os.makedirs(farm)
            for name in os.listdir("/usr/bin"):
                if name not in ("lowriter", "mousepad", "soffice", "libreoffice"):
                    os.symlink(f"/usr/bin/{name}", f"{farm}/{name}")
        real = os.path.realpath("/usr/bin")
        out = []
        for p in path.split(":"):
            if p and os.path.isdir(p) and os.path.realpath(p) == real:
                if farm not in out:
                    out.append(farm)
            else:
                out.append(p)
        joined = ":".join(out)
        if shutil.which("lowriter", path=joined):
            raise RuntimeError("harness: lowriter still on PATH: " + joined)
        return joined

    def spawn(self, argv, env, log, **kw):
        out = open(os.path.join(self.dir, log), "w")
        p = subprocess.Popen(argv, env=env, stdout=out, stderr=subprocess.STDOUT, start_new_session=True, cwd="/home/user/notes", **kw)
        self.pids.append(p.pid)
        return p

    def start(self, startup, flavor=None, test_flags="--test-injected-press-is-real", voice=True, companions=(), extra_config=None, xp=False, outputs=None):
        os.makedirs(f"{self.dir}/stub", exist_ok=True)
        os.makedirs(f"{self.dir}/config/chora", exist_ok=True)
        wpctl_stub(f"{self.dir}/stub")
        for name, text in {"chora-notify.yaml": notify_config(), **(extra_config or {})}.items():
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
            env["CHORA_TEST_KEEP_BUS"] = "1"
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
