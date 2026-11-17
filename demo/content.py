#!/usr/bin/env python3
"""The content of the panes of the demo scene, for a terminal with sixel.

Usage: demo/content.py KIND [FILE]
  picture [SEED]  a generated landscape that fills the terminal
  mosaic [SEED]   the landscape from half-block characters, two pixels a
                  cell, for a terminal with a small font and no sixel
  code FILE     a Rust file with syntax colours
  page          a page with headings, text and two generated pictures
  notes         a page of text only: headings, a list and small charts
  palette       a colour chart with bars
  bars          a bar chart from half-block characters

Everything is generated here: no asset has a licence of a third party.
The pictures need Pillow. The terminal size comes from the terminal;
the cell size comes from its pixel size, or is 8x17 if it gives none.
"""

import fcntl
import math
import os
import random
import re
import shutil
import struct
import sys
import termios

from PIL import Image, ImageDraw, ImageFilter

ESC = "\x1b"


def rgb(color, text, bold=False):
    weight = "1;" if bold else ""
    return f"{ESC}[{weight}38;2;{color[0]};{color[1]};{color[2]}m{text}{ESC}[0m"


def sixel(image, colors=128):
    """An image as a sixel string with an adaptive palette."""
    quantized = image.convert("RGB").quantize(colors, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    palette = quantized.getpalette()[: colors * 3]
    width, height = quantized.size
    pixels = quantized.load()
    out = [f"{ESC}Pq", f'"1;1;{width};{height}']
    for index in range(len(palette) // 3):
        r, g, b = (round(v * 100 / 255) for v in palette[index * 3 : index * 3 + 3])
        out.append(f"#{index};2;{r};{g};{b}")
    for top in range(0, height, 6):
        rows = range(top, min(top + 6, height))
        used = sorted({pixels[x, y] for y in rows for x in range(width)})
        for color in used:
            line = []
            for x in range(width):
                bits = sum(1 << (y - top) for y in rows if pixels[x, y] == color)
                line.append(chr(63 + bits))
            out.append(f"#{color}" + compress("".join(line)) + "$")
        out.append("-")
    out.append(f"{ESC}\\")
    return "".join(out)


def compress(line):
    return re.sub(r"(.)\1{3,}", lambda m: f"!{len(m.group(0))}{m.group(1)}", line)


def landscape(size, seed=7):
    """A sunset over layered hills."""
    width, height = size
    rng = random.Random(seed)
    image = Image.new("RGB", size)
    draw = ImageDraw.Draw(image)
    sky = [(24, 18, 62), (96, 40, 120), (232, 96, 96), (255, 190, 110)]
    horizon = int(height * 0.62)
    for y in range(horizon):
        t = y / horizon * (len(sky) - 1)
        a, b = sky[int(t)], sky[min(int(t) + 1, len(sky) - 1)]
        k = t - int(t)
        draw.line([(0, y), (width, y)], fill=tuple(round(a[i] + (b[i] - a[i]) * k) for i in range(3)))
    sun = (int(width * 0.64), int(horizon * 0.86))
    glow = Image.new("RGB", size)
    ImageDraw.Draw(glow).ellipse(
        [sun[0] - height // 4, sun[1] - height // 4, sun[0] + height // 4, sun[1] + height // 4],
        fill=(255, 150, 80),
    )
    image = Image.blend(image, Image.composite(glow, image, glow.convert("L").filter(ImageFilter.GaussianBlur(height // 9))), 0.55)
    draw = ImageDraw.Draw(image)
    radius = height // 11
    draw.ellipse([sun[0] - radius, sun[1] - radius, sun[0] + radius, sun[1] + radius], fill=(255, 236, 190))
    for _ in range(60):
        x, y = rng.randrange(width), rng.randrange(int(horizon * 0.45))
        draw.point((x, y), fill=(230, 220, 255))
    hills = [(88, 44, 110), (62, 34, 96), (40, 26, 76), (22, 18, 52)]
    for layer, color in enumerate(hills):
        base = horizon - height * 0.12 + layer * height * 0.13
        phase = [rng.uniform(0, math.tau) for _ in range(3)]
        points = [(0, height)]
        for x in range(0, width + 4, 4):
            y = base
            for octave in range(3):
                y -= math.sin(x / width * math.tau * (1.3 + octave * 1.7) + phase[octave]) * height * 0.07 / (octave + 1)
            points.append((x, y))
        points.append((width, height))
        draw.polygon(points, fill=color)
    water = int(height * 0.9)
    for y in range(water, height):
        k = (y - water) / max(height - water, 1)
        draw.line([(0, y), (width, y)], fill=(round(30 + 90 * k), round(22 + 40 * k), round(64 + 30 * k)))
    return image


def chart(size, seed=3):
    """A bar chart on a dark card."""
    width, height = size
    rng = random.Random(seed)
    image = Image.new("RGB", size, (26, 27, 38))
    draw = ImageDraw.Draw(image)
    colors = [(247, 118, 142), (255, 158, 100), (224, 175, 104), (158, 206, 106), (125, 207, 255), (187, 154, 247)]
    count = 12
    gap = width / (count * 1.5 + 0.5)
    for i in range(count):
        value = 0.25 + 0.7 * abs(math.sin(i * 0.7 + rng.random()))
        left = gap * (0.5 + i * 1.5)
        draw.rounded_rectangle(
            [left, height * (1 - value) + 6, left + gap, height - 6], radius=4, fill=colors[i % len(colors)]
        )
    return image


def cells():
    size = shutil.get_terminal_size((80, 24))
    return size.columns, size.lines


def cell():
    """The width and height of a cell in pixels."""
    columns, lines = cells()
    try:
        packed = fcntl.ioctl(sys.stdout.fileno(), termios.TIOCGWINSZ, b"\0" * 8)
        _, _, width, height = struct.unpack("HHHH", packed)
    except OSError:
        width = height = 0
    if width and height:
        return width / columns, height / lines
    return 8.0, 17.0


TOKENS = re.compile(
    r"(?P<comment>//[^\n]*)"
    r'|(?P<string>"(?:\\.|[^"\\])*")'
    r"|(?P<attribute>#!?\[[^\]\n]*\])"
    r"|(?P<number>\b\d[\d_]*(?:\.\d+)?(?:f64|u32|usize|u64|i32)?\b)"
    r"|(?P<keyword>\b(?:fn|let|mut|pub|use|impl|struct|enum|const|static|for|in|if|else|match|while|loop|return|self|Self|crate|super|mod|where|as|move|ref|trait|type|unsafe|break|continue)\b)"
    r"|(?P<type>\b[A-Z][A-Za-z0-9]*\b)"
    r"|(?P<call>\b[a-z_][a-z0-9_]*(?=\())"
    r"|(?P<macro>\b[a-z_]+!)"
)
PALETTE = {
    "comment": (86, 95, 137),
    "string": (158, 206, 106),
    "attribute": (224, 175, 104),
    "number": (255, 158, 100),
    "keyword": (187, 154, 247),
    "type": (42, 195, 222),
    "call": (122, 162, 247),
    "macro": (125, 207, 255),
}
TEXT = (192, 202, 245)


def highlight(line):
    out, last = [], 0
    for match in TOKENS.finditer(line):
        out.append(rgb(TEXT, line[last : match.start()]))
        out.append(rgb(PALETTE[match.lastgroup], match.group(0)))
        last = match.end()
    out.append(rgb(TEXT, line[last:]))
    return "".join(out)


def bar(color, text, columns):
    """A full line with a coloured background."""
    line = f" {text} ".ljust(columns)[:columns]
    return f"{ESC}[1;48;2;{color[0]};{color[1]};{color[2]};38;2;26;27;38m{line}{ESC}[0m"


def code(path):
    columns, lines = cells()
    with open(path, encoding="utf-8") as file:
        source = file.read().splitlines()
    start = next((i for i, line in enumerate(source) if line.startswith("impl ")), 0)
    marks = {0: (158, 206, 106), 1: (158, 206, 106), 2: (158, 206, 106), 9: (224, 175, 104), 15: (125, 207, 255)}
    print(bar((122, 162, 247), f"{os.path.basename(path)}  ·  rust", columns))
    shown = source[start : start + lines - 3]
    for row, line in enumerate(shown):
        mark = marks.get(row % 20)
        gutter = rgb(mark, "▎") if mark else " "
        print(gutter + rgb((86, 95, 137), f"{start + row + 1:4} ") + highlight(line[: columns - 7]))
    sys.stdout.write(bar((187, 154, 247), f"NORMAL  {start + 1}:1  utf-8", columns))


def picture(seed):
    columns, lines = cells()
    width, height = cell()
    sys.stdout.write(sixel(landscape((int(columns * width), int(lines * height) - 1), seed), 200))


def mosaic(image):
    """An image of the size of the terminal in half blocks: the upper
    pixel of a cell is its foreground, the lower one its background."""
    columns, lines = cells()
    pixels = image.convert("RGB").resize((columns, lines * 2), Image.Resampling.LANCZOS).load()
    rows = []
    for line in range(lines):
        row = []
        for x in range(columns):
            top, bottom = pixels[x, line * 2], pixels[x, line * 2 + 1]
            row.append(f"{ESC}[38;2;{top[0]};{top[1]};{top[2]};48;2;{bottom[0]};{bottom[1]};{bottom[2]}m▀")
        rows.append("".join(row) + f"{ESC}[0m")
    sys.stdout.write("\r\n".join(rows))


def scene(draw, seed):
    """A picture of four times the pixels of the mosaic, so that the
    mosaic is smooth."""
    columns, lines = cells()
    return draw((columns * 4, lines * 8), seed)


def page():
    columns, _ = cells()
    width = min(columns, 72)
    cell_w, cell_h = cell()
    heading, accent, muted = (192, 202, 245), (122, 162, 247), (130, 139, 184)
    print()
    print("  " + rgb(heading, "chora", bold=True) + rgb(muted, "  ·  a compositor that gives forms a place"))
    print("  " + rgb(accent, "─" * (width - 4)))
    print()
    sys.stdout.write("  " + sixel(landscape((int((width - 4) * cell_w), int(8 * cell_h)), seed=21), 160) + "\n")
    print()
    print("  " + rgb(heading, "Windows stand in a space", bold=True))
    for line in (
        "Every window is a pane. Walk between them, turn, fly into one",
        "and work in it at 1:1. Grab a pane and it bends like a sail.",
    ):
        print("  " + rgb(TEXT, line[: width - 4]))
    print()
    sys.stdout.write("  " + sixel(chart((int((width - 4) * cell_w), int(6 * cell_h))), 32) + "\n")
    print()
    print("  " + rgb((158, 206, 106), "●") + rgb(muted, " scene  ") + rgb((224, 175, 104), "●") + rgb(muted, " plugins  ") + rgb((247, 118, 142), "●") + rgb(muted, " effects"))


def notes():
    columns, _ = cells()
    width = min(columns, 72) - 4
    heading, accent, muted = (192, 202, 245), (122, 162, 247), (130, 139, 184)
    colors = [(247, 118, 142), (255, 158, 100), (224, 175, 104), (158, 206, 106), (125, 207, 255), (187, 154, 247)]
    rng = random.Random(9)
    print()
    print("  " + rgb(heading, "chora", bold=True) + rgb(muted, "  ·  a compositor that gives forms a place"))
    print("  " + rgb(accent, "─" * width))
    print()
    print("  " + rgb(heading, "A desktop on every face", bold=True))
    for line in (
        "Each face of the cube is a screen of its own, with a",
        "taskbar and windows. Step back and the windows float.",
    ):
        print("  " + rgb(TEXT, line[:width]))
    print()
    items = [("step back", "see the whole cube"), ("turn", "go to the next face"), ("pull", "take a window into the space")]
    for color, (name, text) in zip(colors[3:], items):
        print("  " + rgb(color, "●") + " " + rgb(heading, f"{name:<10}", bold=True) + rgb(muted, text[: width - 13]))
    print()
    print("  " + rgb(heading, "frames", bold=True) + rgb(muted, "  ms, last minute"))
    for row, color in enumerate(colors[:4]):
        levels = [abs(math.sin(i / 6 + row * 1.3)) * (0.55 + 0.45 * rng.random()) for i in range(width)]
        print("  " + rgb(color, "".join("▁▂▃▄▅▆▇█"[min(int(v * 8), 7)] for v in levels)))
    print()
    sys.stdout.write("  " + rgb(colors[3], "●") + rgb(muted, " scene  ") + rgb(colors[2], "●") + rgb(muted, " plugins  ") + rgb(colors[0], "●") + rgb(muted, " effects"))


def palette():
    columns, lines = cells()
    names = ["red", "orange", "yellow", "green", "cyan", "blue", "violet"]
    colors = [(247, 118, 142), (255, 158, 100), (224, 175, 104), (158, 206, 106), (125, 207, 255), (122, 162, 247), (187, 154, 247)]
    rng = random.Random(5)
    print()
    print("  " + rgb((192, 202, 245), "load", bold=True) + rgb((86, 95, 137), "  per core, last minute"))
    print()
    for row in range(min(lines - 5, 14)):
        color = colors[row % len(colors)]
        value = int((columns - 16) * (0.2 + 0.8 * rng.random()))
        print("  " + rgb((130, 139, 184), f"{names[row % len(names)]:>7} ") + rgb(color, "█" * value) + rgb((59, 66, 97), "░" * (columns - 16 - value)))


def main():
    kind = sys.argv[1] if len(sys.argv) > 1 else ""
    sys.stdout.write(f"{ESC}[?25l{ESC}[2J{ESC}[H")
    if kind == "picture":
        picture(int(sys.argv[2]) if len(sys.argv) > 2 else 7)
    elif kind == "mosaic":
        mosaic(scene(landscape, int(sys.argv[2]) if len(sys.argv) > 2 else 7))
    elif kind == "bars":
        mosaic(scene(chart, 11))
    elif kind == "code" and len(sys.argv) > 2:
        code(sys.argv[2])
    elif kind == "page":
        page()
    elif kind == "notes":
        notes()
    elif kind == "palette":
        palette()
    else:
        sys.exit(__doc__)
    sys.stdout.flush()


if __name__ == "__main__":
    main()
