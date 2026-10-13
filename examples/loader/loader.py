"""The loader: a white construct behind the views. Alt+Escape leaves the
current view into the construct; a click on a screen, or Alt+1..3, enters
a view. Run with CHORA_SOCKET set; the nodes go when it stops.
"""
import io, json, os, queue, threading, time
from PIL import Image, ImageDraw, ImageFont
from hud import Chora, FONT, HERE, place

GAP = 1.35  # distance of the screens, in screen widths
VIEWS = [("01  TILING", None), ("02  RAIN", "rain.frag"), ("03  VOID", "void")]
FLY = 1100


def label_png(text):
    font = ImageFont.truetype(FONT, 96)
    w = int(ImageDraw.Draw(Image.new("L", (1, 1))).textlength(text, font=font)) + 8
    img = Image.new("RGBA", (w, 130))
    ImageDraw.Draw(img).text((4, 4), text, font=font, fill=(40, 46, 54, 255))
    buf = io.BytesIO()
    img.save(buf, "PNG")
    return buf.getvalue(), w / 130


class Loader:
    def __init__(self):
        self.c = Chora("loader")
        self.hud = self.c  # the HUD follows the camera
        out = self.c.call("output.list", {})[0]
        self.output, (self.ow, self.oh) = out["name"], out["size"]
        self.w, self.h = self.ow, self.oh
        self.out = False
        self.labels = [label_png(n)[1] for n, _ in VIEWS]
        self.build()

    def x(self, i):
        return i * self.w * GAP

    def build(self):
        W, H = self.w, self.h
        put = lambda name: self.c.put(name, open(os.path.join(HERE, name), "rb").read())
        floor = put("floor.frag")
        # A restart replaces the nodes of the last run.
        old = self.c.call("scene.query", {"selector": "owner:loader"})
        cmds = [{"remove": {"id": n["id"]}} for n in old if n["parent"] == "root"]
        cmds += [
            {"create": {"id": "construct", "kind": "group", "parent": "root", "props": {"input": "pass"}}},
            {"move": {"id": "construct", "parent": "root", "index": 0}},
            {"create": {"id": "void", "kind": "rect", "parent": "construct", "props": {
                "size": [W * 40, H * 40], "color": [0.97, 0.975, 0.98, 1],
                "transform": {"translate": [-W * 20, -H * 20]}}}},
            {"create": {"id": "floor", "kind": "shader", "parent": "construct", "props": {
                "size": [1200, 300], "shader": floor,
                "transform": {"translate": [-W * 3, H * 0.6], "scale": [W * 8 / 1200, H * 2.5 / 300]}}}},
        ]
        for i, (name, kind) in enumerate(VIEWS):
            v, x = f"view{i}", self.x(i)
            png, ratio = label_png(name)
            lh = H * 0.09
            cmds += [
                {"create": {"id": v, "kind": "group", "parent": "construct", "props": {
                    "transform": {"translate": [x, 0]}}}},
                {"create": {"id": f"{v}-shadow", "kind": "rect", "parent": v, "props": {
                    "size": [W * 0.9, H * 0.08], "radius": H * 0.04, "color": [0, 0, 0, 0.10],
                    "transform": {"translate": [W * 0.05, H * 1.12]}}}},
                {"create": {"id": f"{v}-label", "kind": "image", "parent": v, "props": {
                    "image": self.c.put(f"label{i}.png", png), "size": [lh * ratio, lh],
                    "transform": {"translate": [0, -lh * 1.4]}}}},
            ]
            if kind == "void":
                cmds.append({"create": {"id": f"{v}-screen", "kind": "rect", "parent": v, "props": {
                    "size": [W, H], "color": [1, 1, 1, 1], "radius": 6,
                    "border": {"width": 2, "color": [0.8, 0.83, 0.86, 1]}}}})
            elif kind:
                cmds.append({"create": {"id": f"{v}-screen", "kind": "shader", "parent": v, "props": {
                    "size": [W, H], "shader": put(kind)}}})
            else:
                cmds.append({"create": {"id": f"{v}-screen", "kind": "rect", "parent": v, "props": {
                    "size": [W, H], "color": [0, 0, 0, 1]}}})
            # The hit area for clicks, only there in the construct.
            cmds.append({"create": {"id": f"{v}-hit", "kind": "rect", "parent": "root", "props": {
                "size": [W, H * 1.25], "color": [0, 0, 0, 0], "visible": False,
                "transform": {"translate": [x, -H * 0.15]}}}})
        self.c.call("scene.apply", {"commands": cmds})

    def camera(self, scale, cx, cy):
        """Flies the camera in steps of about a frame. Each step moves the
        camera and the HUD in one transaction, so the HUD stays still."""
        m = self.c.call("output.list", {})[0]["camera"]["matrix"]
        a = (m[0], m[12], m[13])
        b = (scale, self.ow / 2 - scale * cx, self.oh / 2 - scale * cy)
        start = time.monotonic()
        while True:
            k = min(1.0, (time.monotonic() - start) * 1000 / FLY)
            e = k * k * (3 - 2 * k)  # ease in and out
            # The scale moves geometrically, so a zoom feels even.
            s = a[0] * (b[0] / a[0]) ** e
            t = [a[1] + (b[1] - a[1]) * e, a[2] + (b[2] - a[2]) * e]
            self.step(s, t)
            if k == 1.0:
                return
            time.sleep(1 / 60)

    def step(self, s, t):
        out = {"size": [self.ow, self.oh], "camera": {"matrix": [s, 0, 0, 0, 0, s, 0, 0, 0, 0, 1, 0, t[0], t[1], 0, 1]}}
        cmds = [{"camera": {"output": self.output, "transform": {"scale": s, "translate": t}}}]
        if self.hud.call("scene.query", {"selector": "id:hud"}):
            cmds.append({"update": {"id": "hud", "props": {"transform": place(out)}}})
        # The HUD belongs to the client hud: its connection sends the transaction.
        self.hud.call("scene.apply", {"commands": cmds})

    def hits(self, on):
        self.c.call("scene.apply", {"commands": [
            {"update": {"id": f"view{i}-hit", "props": {"visible": on}}} for i in range(len(VIEWS))]})

    def leave(self):
        self.out = True
        span = self.x(len(VIEWS) - 1) + self.w
        scale = self.ow / (span * 1.25)
        self.camera(scale, span / 2, self.h * 0.55)
        self.hits(True)

    def enter(self, i):
        self.out = False
        self.hits(False)
        self.camera(1.0, self.x(i) + self.w / 2, self.h / 2)

    def activate(self, i, surface=None):
        """Leaves the walk into view i at once: the walk ends where the screen
        fills the view, so the 2D camera takes over without a jump."""
        self.walk.leave()
        self.out = False
        self.hits(False)
        self.step(1.0, [-self.x(i), 0])
        if surface:
            self.c.call("scene.apply", {"commands": [{"focus": {"surface": surface}}]})

    def run(self):
        from walk import Walk
        walk, events = Walk(self), queue.Queue()
        self.walk = walk

        def listen(conn, source):
            for line in conn.file:
                p = json.loads(line).get("params") or {}
                if "event" in p:
                    events.put((source, p))

        ev = Chora("loader")
        keys = {ev.call("input.grab", {"binding": b}): b for b in
                ["Alt+0"] + [f"Alt+{i + 1}" for i in range(len(VIEWS))]}
        ev.call("events.subscribe", {"events": ["input.key", "pointer.pressed"]})
        threading.Thread(target=listen, args=(ev, "main"), daemon=True).start()
        while True:
            source, p = events.get()
            try:
                self.handle(walk, keys, source, p, listen)
            except Exception as err:  # one bad event must not end the loader
                print("loader:", err, flush=True)
                walk.leave()

    def handle(self, walk, keys, source, p, listen):
        # Each connection gets each event: the walk takes its own copies.
        if source == "walk":
            if not walk.on:
                return
            if p["event"] == "input.key":
                walk.key(p["binding"])
            else:
                walk.pointer(p)
        elif walk.on and p["event"].startswith("pointer."):
            return
        elif p["event"] == "input.key":
            b = keys.get(p["binding"], p["binding"])
            if b == "Alt+0":
                if not walk.on:
                    conn = walk.enter()
                    conn.call("events.subscribe", {"events": [
                        "input.key", "pointer.pressed", "pointer.moved", "pointer.released"]})
                    threading.Thread(target=listen, args=(conn, "walk"), daemon=True).start()
            else:
                if walk.on:
                    walk.leave()
                self.enter(int(b[-1]) - 1)
        elif p["node"].endswith("-hit") and self.out and not walk.on:
            self.enter(int(p["node"][4:-4]))


Loader().run()
