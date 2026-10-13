"""A live HUD over the scene: a progress bar and a status line.

Usage: hud.py set PERCENT TEXT  |  hud.py hide
"""
import base64, io, json, os, socket, sys
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
FONT = os.environ.get("LOADER_FONT", "/usr/share/fonts/liberation-mono-fonts/LiberationMono-Bold.ttf")
W, PAD, SS = 620, 14, 3  # panel width, padding, supersampling of the text


class Chora:
    def __init__(self, name):
        self.sock = socket.socket(socket.AF_UNIX)
        self.sock.connect(os.environ["CHORA_SOCKET"])
        self.file = self.sock.makefile("rw")
        self.id = 0
        self.call("hello", {"name": name, "keep_nodes": True})

    def call(self, method, params):
        self.id += 1
        self.file.write(json.dumps({"jsonrpc": "2.0", "id": self.id, "method": method, "params": params}) + "\n")
        self.file.flush()
        while True:
            msg = json.loads(self.file.readline())
            if msg.get("id") == self.id:
                if "error" in msg:
                    raise RuntimeError(msg["error"])
                return msg.get("result")

    def put(self, name, data):
        return self.call("assets.put", {"name": name, "data": base64.b64encode(data).decode()})["path"]


def text_png(text, percent):
    w, h = (W - 2 * PAD) * SS, 22 * SS
    img = Image.new("RGBA", (w, h))
    d = ImageDraw.Draw(img)
    font = ImageFont.truetype(FONT, 13 * SS)
    pct = f"{percent:3d}%"
    text = "» " + text
    room = w - d.textlength(pct + "  ", font=font)
    while d.textlength(text, font=font) > room:
        text = text[:-2] + "…"
    d.text((0, 2 * SS), text, font=font, fill=(150, 240, 255, 255))
    d.text((w - d.textlength(pct, font=font), 2 * SS), pct, font=font, fill=(255, 90, 220, 255))
    buf = io.BytesIO()
    img.save(buf, "PNG")
    return buf.getvalue()


TOP = 12  # the place of the HUD on the screen: centered, this far from the top


def place(out):
    """The transform that holds the HUD still on the screen: the inverse of
    the camera, which maps a scene point p to scale * p + translate."""
    m = out["camera"]["matrix"]
    s, tx, ty = m[0], m[12], m[13]
    x = (out["size"][0] - W) / 2
    return {"scale": 1 / s, "translate": [(x - tx) / s, (TOP - ty) / s]}


def main():
    c = Chora("loader")
    old = [n["id"] for n in c.call("scene.query", {"selector": "owner:loader"}) if n["parent"] == "root" and n["id"].startswith("hud")]
    if sys.argv[1] == "hide":
        c.call("scene.apply", {"commands": [{"remove": {"id": i}} for i in old]})
        return
    percent, text = int(sys.argv[2]), " ".join(sys.argv[3:])
    out = c.call("output.list", {})[0]
    frag = c.put("hud-bar.frag", open(os.path.join(HERE, "hud-bar.frag"), "rb").read())
    # A new name per update: the core reloads an image when its path changes.
    png = c.put("hud-text.png", text_png(text, percent))
    progress = {"shader": frag, "uniforms": {"progress": percent / 100}}
    cmds = [{"remove": {"id": i}} for i in old if i != "hud"]
    if "hud" in old:
        cmds += [{"update": {"id": "hud-text", "props": {"image": png}}},
                 {"update": {"id": "hud-bar", "props": progress}},
                 {"move": {"id": "hud", "parent": "root"}}]
    else:
        cmds += [
            {"create": {"id": "hud", "kind": "group", "parent": "root",
                        "props": {"input": "pass", "transform": place(out)}}},
            {"create": {"id": "hud-bg", "kind": "rect", "parent": "hud", "props": {
                "size": [W, 56], "radius": 10, "color": [0.02, 0.04, 0.08, 0.82],
                "border": {"width": 1, "color": [0.2, 0.9, 1.0, 0.7]}}}},
            {"create": {"id": "hud-text", "kind": "image", "parent": "hud", "props": {
                "image": png, "size": [W - 2 * PAD, 22], "transform": {"translate": [PAD, 7]}}}},
            {"create": {"id": "hud-bar", "kind": "shader", "parent": "hud", "props": {
                "size": [W - 2 * PAD, 10], "transform": {"translate": [PAD, 36]}, **progress}}},
        ]
    c.call("scene.apply", {"commands": cmds})


if __name__ == "__main__":
    main()
