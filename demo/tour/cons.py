"""The construct session of the v2 tour: the demo panes of
demo/scene-construct.sh, with the voice HUD of the host (chora-assistant)
and the fake voice agent, so that the HUD, the chat and the questions show."""
import json, os, shlex, time
from launch import *
from tour import Session


def pane(name, at, yaw, width, height, *args):
    pad = "0x0" if args[0] == "picture" else "14x12"
    cmd = " ".join(shlex.quote(a) for a in (CONTENT, *args))
    return f"""  - kind: program
    command: >-
      exec foot --app-id={name} -o pad={pad} -o colors.background=1a1b26 -o colors.foreground=c0caf5
      -o 'font=DejaVu Sans Mono:size=9' -o cursor.blink=no
      -e sh -c 'sleep 1.5; python3 -I "$0" "$@"; exec sleep infinity'
      {cmd}
    at: [{at}]
    yaw: {yaw}
    size: [{width}, {height}]
"""


def start_construct(theme="light", name="construct"):
    L = Launcher(name)
    os.makedirs(f"{L.dir}/config/chora/constructs", exist_ok=True)
    body = ["camera: {x: 0, z: -100, yaw: 0, pitch: -0.07}", "radius: 6000", "panes:"]
    body.append(pane("sunset", "0, 1500", 0, 760, 480, "picture"))
    body.append(pane("wobbly.rs", "-770, 1440", -0.42, 560, 470, "code", f"{REPO}/crates/chora-kit/src/wobbly.rs"))
    body.append(pane("chora", "770, 1440", 0.42, 560, 470, "page"))
    body.append(pane("load", "-1550, 700", -1.1, 560, 400, "palette"))
    body.append(pane("wobble.rs", "1550, 700", 1.1, 600, 420, "code", f"{REPO}/plugins/chora-construct/src/wobble.rs"))
    open(f"{L.dir}/config/chora/constructs/demo.yaml", "w").write("\n".join(body) + "\n")
    env = L.start([f"{BIN}/chora-construct"], flavor="construct", companions=[[f"{BIN}/chora-assistant"]],
                  extra_config={"chora-construct.yaml": f"theme: {theme}\neffects: [wobbly]\ntest_accept_injected_keys: true\n"})
    S = Session(env, L.pids, BIN)
    S.ctl("wait", "id:construct-look", "--timeout", "30")
    for _ in range(60):
        try:
            S.call("construct", "construct.load", {"name": "demo"})
            break
        except Exception:
            time.sleep(0.5)
    for _ in range(80):
        n = S.call("construct", "window.list")
        panes = [w for w in n.get("windows", []) if w.get("pane") in ("shown", "aside", "behind", "far")]
        if len(panes) >= 5:
            break
        time.sleep(0.5)
    for yaw in (-1.2, 1.2, 0):
        S.call("construct", "camera.glide", {"x": 0, "z": -100, "yaw": yaw, "pitch": -0.07})
        time.sleep(1.5)
    return L, S
