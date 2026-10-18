"""slides.py — a three slide presentation on the chora scene API.

Title slide, bullet slide and a figure slide (chart drawn with PIL). Slides
are 1280x800 image nodes in a row in the scene; the camera travels between
them with animated moves that zoom out and back in (camera command with
duration). The slide counter is fixed on the screen root screen:HEADLESS-1,
so it does not move with the camera.

Run:  CHORA_SOCKET=... python slides.py   (needs grant: slides screen.capture)
Saves one PNG per slide into the shots directory of the runtime
directory and prints camera timings.
"""
import io, os, time
from PIL import Image, ImageDraw, ImageFont
from choraclient import Chora

FONT = "/usr/share/fonts/liberation-mono-fonts/LiberationMono-Regular.ttf"
BOLD = "/usr/share/fonts/liberation-mono-fonts/LiberationMono-Bold.ttf"
W, H, GAP = 1280, 800, 320
STEP = W + GAP
OUT = os.environ.get("SLIDES_OUTPUT", "HEADLESS-1")
BG = (14, 16, 22)
ACCENT = (110, 210, 255)
SHOTS = os.path.join(os.environ.get("XDG_RUNTIME_DIR", os.getcwd()), "chora-showcase-shots")


def f(path, size):
    return ImageFont.truetype(path, size)


def slide_title():
    img = Image.new("RGBA", (W, H), BG)
    d = ImageDraw.Draw(img)
    d.text((100, 280), "CHORA", font=f(BOLD, 110), fill=(240, 240, 238))
    d.text((104, 430), "a scene graph you can talk to", font=f(FONT, 30), fill=ACCENT)
    d.line([(100, 520), (700, 520)], fill=(70, 76, 90), width=2)
    d.text((104, 545), "dogfood showcase · agent 3", font=f(FONT, 20), fill=(140, 145, 155))
    return img


def slide_bullets():
    img = Image.new("RGBA", (W, H), BG)
    d = ImageDraw.Draw(img)
    d.text((100, 80), "What a node knows", font=f(BOLD, 52), fill=(240, 240, 238))
    items = ["transform, opacity, size — animated by the core",
             "rect, image, shader, group, surface",
             "clip, radius, border make it a layer",
             "effects: up to 8 fragment shaders, in order",
             "focus_node: the keyboard without a window",
             "screen:NAME — nodes that ignore the camera"]
    y = 220
    for it in items:
        d.ellipse((104, y + 12, 116, y + 24), fill=ACCENT)
        d.text((144, y), it, font=f(FONT, 27), fill=(214, 216, 222))
        y += 84
    return img


def slide_figure():
    img = Image.new("RGBA", (W, H), BG)
    d = ImageDraw.Draw(img)
    d.text((100, 70), "Frame times in the headless session", font=f(BOLD, 44), fill=(240, 240, 238))
    d.text((100, 140), "ms per frame, llvmpipe, 1280x800 (illustrative)", font=f(FONT, 20), fill=(150, 155, 165))
    x0, y0, x1, y1 = 150, 240, 1180, 660  # plot area, y0 top
    vals = [("idle", 4.1), ("100 rects", 7.8), ("ball shader", 9.6),
            ("60 fps updates", 12.9), ("500 nodes", 21.4)]
    d.line([(x0, y1), (x1, y1)], fill=(90, 96, 110), width=2)
    d.line([(x0, y0), (x0, y1)], fill=(90, 96, 110), width=2)
    bw = (x1 - x0) / len(vals)
    top = 25.0
    for i, (name, v) in enumerate(vals):
        bh = (y1 - y0) * v / top
        bx = x0 + i * bw + bw * 0.18
        d.rounded_rectangle((bx, y1 - bh, bx + bw * 0.64, y1), 6, fill=(70, 160, 230))
        d.text((bx + bw * 0.12, y1 - bh - 34), f"{v:.1f}", font=f(BOLD, 22), fill=ACCENT)
        d.text((x0 + i * bw + 14, y1 + 14), name, font=f(FONT, 18), fill=(190, 193, 200))
    for t in (5, 10, 15, 20):
        ty = y1 - (y1 - y0) * t / top
        d.line([(x0 - 8, ty), (x0, ty)], fill=(90, 96, 110), width=2)
        d.text((x0 - 60, ty - 11), str(t), font=f(FONT, 17), fill=(150, 155, 165))
    return img


def counter_png(i, n):
    img = Image.new("RGBA", (180, 50))
    d = ImageDraw.Draw(img)
    d.rounded_rectangle((0, 0, 179, 49), 12, fill=(0, 0, 0, 150))
    t = f"{i + 1} / {n}"
    d.text(((180 - d.textlength(t, font=f(BOLD, 26))) / 2, 9), t, font=f(BOLD, 26), fill=ACCENT)
    buf = io.BytesIO()
    img.save(buf, "PNG")
    return buf.getvalue()


def png(img):
    buf = io.BytesIO()
    img.save(buf, "PNG")
    return buf.getvalue()


def main():
    os.makedirs(SHOTS, exist_ok=True)
    c = Chora("slides")
    for n in [x["id"] for x in c.call("scene.query", {"selector": "owner:slides"})]:
        c.apply([{"remove": {"id": n}}])
    slides = [slide_title(), slide_bullets(), slide_figure()]
    cmds = [{"create": {"id": "slides", "kind": "group", "parent": "root"}}]
    for i, img in enumerate(slides):
        cmds.append({"create": {"id": f"slide{i}", "kind": "image", "parent": "slides", "props": {
            "image": c.put(f"slide{i}.png", png(img)), "transform": {"translate": [i * STEP, 0]}}}})
    # the counter lives on the screen root: fixed while the camera moves
    cmds.append({"create": {"id": "counter", "kind": "image", "parent": f"screen:{OUT}", "props": {
        "image": c.put("counter.png", counter_png(0, len(slides))),
        "transform": {"translate": [W - 210, H - 70]}}}})
    c.apply(cmds)
    c.call("events.subscribe", {"events": ["animation.done"]})

    def fly(scale, tx, ty, ms):
        t0 = time.monotonic()
        c.apply([{"camera": {"output": OUT, "transform": {"scale": scale, "translate": [tx, ty]},
                             "duration": ms, "easing": "ease_in_out"}}])
        while True:
            p = c.events.get()
            if p["event"] == "animation.done" and p.get("output") == OUT:
                return (time.monotonic() - t0) * 1000

    fly(1.0, 0, 0, 1)  # park at slide 0 without a visible move
    for i in range(len(slides)):
        if i:
            mid = (i - 0.5) * STEP
            s = 0.6
            ms1 = fly(s, W / 2 - s * mid, H / 2 - s * H / 2, 450)  # zoom out between slides
            ms2 = fly(1.0, -i * STEP, 0, 450)  # zoom into the next slide
            print(f"slide {i}: transition {ms1:.0f} + {ms2:.0f} ms", flush=True)
        # a same-name assets.put does NOT reload the node: update the prop
        c.apply([{"update": {"id": "counter",
                "props": {"image": c.put(f"counter{i}.png", counter_png(i, len(slides)))}}}])
        r = c.capture(os.path.join(SHOTS, f"slide{i + 1}.png"), output=OUT)
        print(f"slide {i + 1}/{len(slides)} captured {r['width']}x{r['height']}", flush=True)
        time.sleep(1.2)
    print("done", flush=True)


if __name__ == "__main__":
    main()
