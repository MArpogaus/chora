"""pong.py — a 60 fps pong game on the chora scene API.

Two paddles, a ball with an effect shader (ball.frag, pulsing gradient) and
a score drawn with PIL into an image node. The loop updates the transforms
of the rects about 60 times a second; the left paddle chases the ball, the
right one moves on a scripted sine, so rallies end and the score moves.
Run for RUN seconds, print the achieved update rate, save a shot, clean up.

Run:  CHORA_SOCKET=... python pong.py   (needs `./grant-pong.sh` for its screenshots)
"""
import io, math, os, time
from PIL import Image, ImageDraw, ImageFont
from choraclient import Chora

BOLD = "/usr/share/fonts/liberation-mono-fonts/LiberationMono-Bold.ttf"
W, H = 1280, 800
PW, PH = 20, 120
BR = 24
OUT = os.environ.get("SLIDES_OUTPUT", "HEADLESS-1")
RUN = float(os.environ.get("PONG_RUN", "12"))
SHOTS = os.path.join(os.environ.get("XDG_RUNTIME_DIR", os.getcwd()), "chora-showcase-shots")
HERE = os.path.dirname(os.path.abspath(__file__))


def score_png(a, b):
    img = Image.new("RGBA", (300, 70))
    d = ImageDraw.Draw(img)
    t = f"{a:02d}  {b:02d}"
    f = ImageFont.truetype(BOLD, 52)
    d.text(((300 - d.textlength(t, font=f)) / 2, 4), t, font=f, fill=(235, 235, 235, 220))
    buf = io.BytesIO()
    img.save(buf, "PNG")
    return buf.getvalue()


def main():
    os.makedirs(SHOTS, exist_ok=True)
    c = Chora("pong")
    for n in [x["id"] for x in c.call("scene.query", {"selector": "owner:pong"})]:
        c.apply([{"remove": {"id": n}}])
    score = [0, 0]
    c.apply([
        {"create": {"id": "pong", "kind": "group", "parent": "root", "props": {"input": "pass"}}},
        {"create": {"id": "pong-bg", "kind": "rect", "parent": "pong",
                    "props": {"size": [W, H], "color": [0.05, 0.06, 0.09, 1]}}},
        {"create": {"id": "pong-net", "kind": "rect", "parent": "pong", "props": {
            "size": [4, H], "color": [0.3, 0.33, 0.4, 0.8], "transform": {"translate": [W // 2, 0]}}}},
        {"create": {"id": "pong-padL", "kind": "rect", "parent": "pong", "props": {
            "size": [PW, PH], "radius": 8, "color": [0.35, 0.8, 1, 1]}}},
        {"create": {"id": "pong-padR", "kind": "rect", "parent": "pong", "props": {
            "size": [PW, PH], "radius": 8, "color": [1, 0.55, 0.35, 1],
            "transform": {"translate": [W - PW, 0]}}}},
        {"create": {"id": "pong-ball", "kind": "rect", "parent": "pong", "props": {
            "size": [BR, BR], "radius": BR // 2, "color": [1, 1, 1, 1],
            "effects": [{"shader": os.path.join(HERE, "ball.frag")}]}}},
        {"create": {"id": "pong-score", "kind": "image", "parent": "pong", "props": {
            "image": c.put("score0.png", score_png(0, 0)), "size": [300, 70],
            "transform": {"translate": [(W - 300) / 2, 30]}}}},
    ])

    x, y = W / 2, H / 2
    vx, vy = 480.0, 240.0
    yl, yr = H / 2 - PH / 2, H / 2 - PH / 2
    t0 = time.monotonic()
    step = 1 / 60
    frames = late = 0
    mid = None
    while True:
        now = time.monotonic()
        if now - t0 >= RUN:
            break
        x += vx * step
        y += vy * step
        if y < 0 or y > H - BR:
            vy = -vy
            y = min(max(y, 0), H - BR)
        # left paddle chases, right paddle lives on a sine
        yl += max(-9, min(9, (y - PH / 4) - yl))
        yr = H / 2 - PH / 2 + 240 * math.sin((now - t0) * 1.3)
        if x <= PW and yl < y + BR < yl + PH + 10 and vx < 0:
            vx = abs(vx) * 1.05
            vy += (y - (yl + PH / 2)) * 3
        if x >= W - PW - BR and yr < y + BR < yr + PH + 10 and vx > 0:
            vx = -abs(vx) * 1.05
            vy += (y - (yr + PH / 2)) * 3
        if x < -BR:
            score[1] += 1
            x, y, vx, vy = W / 2, H / 2, 480.0, 240.0
            c.apply([{"update": {"id": "pong-score", "props": {
                "image": c.put(f"score{score[0]}-{score[1]}.png", score_png(*score))}}}])
        if x > W:
            score[0] += 1
            x, y, vx, vy = W / 2, H / 2, -480.0, 240.0
            c.apply([{"update": {"id": "pong-score", "props": {
                "image": c.put(f"score{score[0]}-{score[1]}.png", score_png(*score))}}}])
        vy = max(-700, min(700, vy))
        c.apply([
            {"update": {"id": "pong-ball", "props": {"transform": {"translate": [x, y]}}}},
            {"update": {"id": "pong-padL", "props": {"transform": {"translate": [0, yl]}}}},
            {"update": {"id": "pong-padR", "props": {"transform": {"translate": [W - PW, yr]}}}},
        ])
        frames += 1
        if mid is None and now - t0 >= RUN / 2:
            mid = c.capture(os.path.join(SHOTS, "pong-mid.png"), output=OUT)
        if time.monotonic() - now > 1 / 30:
            late += 1
        sleep = t0 + frames * step - time.monotonic()  # pace to 60 steps/s
        if sleep > 0:
            time.sleep(sleep)
    el = time.monotonic() - t0
    c.capture(os.path.join(SHOTS, "pong-final.png"), output=OUT)
    print(f"pong: {frames} frames in {el:.1f}s = {frames / el:.1f} updates/s "
          f"({late} slow), score {score[0]}:{score[1]}", flush=True)
    c.apply([{"remove": {"id": "pong"}}])


if __name__ == "__main__":
    main()
