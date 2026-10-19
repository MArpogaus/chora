"""news.py — a text-only reader on the chora scene API.

A left index of 6 headlines and a reading pane. All text is drawn with PIL
into PNGs and shown as image nodes. No window: the keyboard goes to the
group node "news" via focus_node; Page_Down/Page_Up (injected keys) move
through the headlines.

Run:  CHORA_SOCKET=... python news.py
Keys: Page_Down/Page_Up next/prev, q quits. Inject with e.g.
  echo '[{"key":{"key":"Page_Down","state":"pressed"}}]' | choractl --name demo-keys inject
"""
import io, os, sys
from PIL import Image, ImageDraw, ImageFont
from choraclient import Chora

FONT = "/usr/share/fonts/liberation-mono-fonts/LiberationMono-Regular.ttf"
BOLD = "/usr/share/fonts/liberation-mono-fonts/LiberationMono-Bold.ttf"
W, H = 1280, 800
IDX_W, PAD = 360, 30
FG, SEL, DIM, BG = (235, 235, 230), (120, 220, 255), (120, 125, 135), (16, 18, 24)

NEWS = [
    ("Chora ships headless backend",
     "The compositor now runs without a display, a GPU or a seat. Tests start "
     "it with two virtual outputs and drive the whole scene over the IPC "
     "socket. The renderer falls back to llvmpipe, which is slow but exact."),
    ("Scene API reaches 1.0 freeze",
     "The node property table is the one list of what a node can do. Unknown "
     "fields in a transaction are errors, so a typo never passes silently. "
     "Plugins, choractl and agents speak one language of commands and events."),
    ("Camera moves get easing curves",
     "A camera command with a duration moves the output over the curve. It "
     "starts where the camera is now, also in the middle of another move. "
     "The end sends animation.done with the output name."),
    ("Nodes can hold the keyboard",
     "focus_node gives the keys to a node that is no surface. The core sends "
     "each key as node.key to the connections of the owner. A held key "
     "repeats after 200 ms, 25 times a second, like a Wayland client."),
    ("Shader effects with budgets",
     "Up to eight effects per node, applied in order. When frames stay slow "
     "for two seconds, a watchdog removes the effects of the most expensive "
     "client and, if that does not help, disconnects it. effects.dropped says so."),
    ("Captures render the scene again",
     "screen.capture answers with a PNG in base64. A capture of an output "
     "shows it as on screen without the cursor; a capture of a node shows it "
     "without its transform, on a transparent background."),
]


def font(path, size):
    return ImageFont.truetype(path, size)


def wrap(text, f, room):
    d = ImageDraw.Draw(Image.new("L", (1, 1)))
    lines, line = [], ""
    for word in text.split():
        if d.textlength(line + " " + word, font=f) > room and line:
            lines.append(line)
            line = word
        else:
            line = f"{line} {word}".strip()
    lines.append(line)
    return lines


def index_png(sel):
    img = Image.new("RGBA", (IDX_W, H), BG)
    d = ImageDraw.Draw(img)
    f, fb = font(BOLD, 15), font(FONT, 13)
    d.text((PAD, 26), "CHORA DAILY", font=fb, fill=SEL)
    y = 78
    for i, (title, _) in enumerate(NEWS):
        c = SEL if i == sel else DIM
        for ln in wrap(title, f if i == sel else fb, IDX_W - 2 * PAD - 20):
            d.text((PAD + (0 if i == sel else 6), y), ln, font=f if i == sel else fb, fill=c)
            y += 22
        y += 14
    d.line([(IDX_W - 1, 0), (IDX_W - 1, H)], fill=(50, 55, 65), width=1)
    return img


def read_png(i):
    img = Image.new("RGBA", (W - IDX_W, H), BG)
    d = ImageDraw.Draw(img)
    title, body = NEWS[i]
    ft, fb, fn = font(BOLD, 34), font(BOLD, 15), font(FONT, 17)
    x, y = PAD, 40
    for ln in wrap(title, ft, img.width - 2 * PAD):
        d.text((x, y), ln, font=ft, fill=FG)
        y += 42
    d.line([(x, y + 6), (img.width - x, y + 6)], fill=(60, 66, 80))
    y += 30
    for para in wrap(body, fn, img.width - 2 * PAD):
        d.text((x, y), para, font=fn, fill=(200, 202, 208))
        y += 27
    d.text((x, H - 50), f"article {i + 1} of {len(NEWS)}  ·  Page_Down next", font=fb, fill=DIM)
    return img


def png_bytes(img):
    buf = io.BytesIO()
    img.save(buf, "PNG")
    return buf.getvalue()


def main():
    c = Chora("news")
    old = [n["id"] for n in c.call("scene.query", {"selector": "owner:news"})]
    c.apply([{"remove": {"id": n}} for n in old])
    sel = 0
    c.put("index.png", png_bytes(index_png(sel)))
    c.put("read.png", png_bytes(read_png(sel)))
    c.apply([
        {"create": {"id": "news", "kind": "group", "parent": "root"}},
        {"create": {"id": "news-bg", "kind": "rect", "parent": "news",
                    "props": {"size": [W, H], "color": [c0 / 255 for c0 in (*BG, 1)]}}},
        {"create": {"id": "news-index", "kind": "image", "parent": "news",
                    "props": {"image": c.put("index.png", png_bytes(index_png(sel)))}}},
        {"create": {"id": "news-read", "kind": "image", "parent": "news",
                    "props": {"image": c.put("read.png", png_bytes(read_png(sel))),
                              "transform": {"translate": [IDX_W, 0]}}}},
    ])
    c.apply([{"focus_node": {"node": "news"}}])
    c.call("events.subscribe", {"events": ["node.key"]})
    print("news: ready, node 'news' has the keyboard", flush=True)
    while True:
        p = c.events.get()
        if p["event"] != "node.key" or p["state"] != "pressed":
            continue
        k = p["key"]
        if k in ("q", "Q"):
            break
        if k in ("Next", "Page_Down", "Down"):
            sel = (sel + 1) % len(NEWS)
        elif k in ("Prior", "Page_Up", "Up"):
            sel = (sel - 1) % len(NEWS)
        else:
            continue
        c.apply([
            {"update": {"id": "news-index", "props": {"image": c.put("index.png", png_bytes(index_png(sel)))}}},
            {"update": {"id": "news-read", "props": {"image": c.put("read.png", png_bytes(read_png(sel)))}}},
        ])
        print(f"news: show {sel}", flush=True)
    c.apply([{"focus_node": {}}, {"remove": {"id": "news"}}])


if __name__ == "__main__":
    main()
