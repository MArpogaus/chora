"""Make clean backgrounds for the HUD mockups from the 11-11 showcase.

Removes today's HUD (Chat pill, helper row, Listening chip, pane label,
OPEN CUBE and the cube chips) from real construct captures, by copying the
same rows from a clean spot beside each element (the sky and the floor
change with the height, hardly with x). Builds the dark cube by placing the
real stood-back face of the light capture into the real dark void.

Run: python3 design/mockups/clean.py  (from the log worktree)
"""
from pathlib import Path
from PIL import Image, ImageChops, ImageFilter

SHOW = Path("media/2026-11-11/showcase")
OUT = Path("media/2026-11-11/construct-hud/bg")
OUT.mkdir(parents=True, exist_ok=True)


def lerp_rows(im, box):
    """Fill box with a blend of the columns just left and right of it."""
    x0, y0, x1, y1 = box
    px = im.load()
    for y in range(y0, y1):
        a, b = px[x0 - 1, y], px[x1, y]
        for x in range(x0, x1):
            t = (x - x0 + 1) / (x1 - x0 + 1)
            px[x, y] = tuple(round(a[i] * (1 - t) + b[i] * t) for i in range(3))


def patch(im, box, dx):
    """Fill box (x0, y0, x1, y1) with the pixels dx to the side."""
    x0, y0, x1, y1 = box
    src = im.crop((x0 + dx, y0, x1 + dx, y1))
    im.paste(src, (x0, y0))


# Boxes in 2x pixels, with a margin of about 12 px around each old element.
WALK_BOXES = [
    ((52, 52, 562, 216), 520),      # Chat ... and the helper row
    ((1100, 46, 1326, 140), -420),  # Listening and the ring
    ((2262, 1346, 2548, 1436), -560),  # OPEN CUBE
]

for name, out in [
    ("10-construct-walk-fly-list-light-panes-2x.png", "walk-light.png"),
    ("12-construct-glass-chat-question-dark-panes-2x.png", "walk-dark.png"),
]:
    im = Image.open(SHOW / name).convert("RGB")
    for box, dx in WALK_BOXES:
        patch(im, box, dx)
    lerp_rows(im, (1176, 864, 1386, 946))  # SUNSET label
    im.save(OUT / out, optimize=True)

# The light cube: remove TURN LEFT, TURN RIGHT and STEP IN at the bottom right.
cube = Image.open(SHOW / "14-cube-step-back-turn-step-in-stepped-back-2x.png").convert("RGB")
lerp_rows(cube, (1700, 1284, 2556, 1382))
cube.save(OUT / "cube-light.png", optimize=True)

# The dark cube: a void made of one clean column of the dark capture,
# the face of the light capture, and its faint mirror image on the floor.
dark = Image.open(SHOW / "12-construct-glass-chat-question-dark-panes-2x.png").convert("RGB")
W, H = dark.size
column = dark.crop((828, 0, 832, H)).resize((1, H))
void = column.resize((W, H))
face = cube.crop((476, 266, 2100, 1174))
# The face is the rect 480..2082 x 270..1170 (with its accent edge), and the
# chora window overhangs it at the right, 2082..2096 x 537..1156.
d = Image.new("L", face.size, 0)
d.paste(255, (480 - 476, 270 - 266, 2082 - 476, 1170 - 266))
d.paste(255, (2082 - 476, 537 - 266, 2096 - 476, 1156 - 266))
void.paste(face, (476, 266), d)
mirror = face.transpose(Image.FLIP_TOP_BOTTOM).crop((0, 0, face.width, 260))
mirror = mirror.filter(ImageFilter.GaussianBlur(3))
mask = Image.linear_gradient("L").resize((face.width, 260))
mask = mask.point(lambda v: int((255 - v) * 0.16))
mask = ImageChops.darker(mask, d.transpose(Image.FLIP_TOP_BOTTOM).crop((0, 0, face.width, 260)))
void.paste(mirror, (476, 1174 + 8), mask)
void.save(OUT / "cube-dark.png", optimize=True)

# Thumbnails of the three panes, for the window list.
walk = Image.open(OUT / "walk-light.png")
for i, box in enumerate([(890, 352, 1676, 848), (100, 330, 760, 870), (1800, 330, 2490, 870)]):
    walk.crop(box).resize((160, 100), Image.LANCZOS).save(OUT / f"thumb-{i}.png")
print("ok")
