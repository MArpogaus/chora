"""Chapter slides of the tour: a title card of 1280x720, as PNG."""
from PIL import Image, ImageDraw, ImageFont

BOLD = "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"
PLAIN = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"
MONO = "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf"


def slide(path, number, title, lines, accent=(122, 162, 247)):
    image = Image.new("RGB", (1280, 720), (26, 27, 38))
    d = ImageDraw.Draw(image)
    d.rectangle([80, 250, 86, 470], fill=accent)
    d.text((120, 240), f"CHAPTER {number}", font=ImageFont.truetype(MONO, 28), fill=accent)
    d.text((120, 290), title, font=ImageFont.truetype(BOLD, 64), fill=(192, 202, 245))
    y = 390
    for line in lines:
        d.text((120, y), line, font=ImageFont.truetype(PLAIN, 28), fill=(150, 160, 200))
        y += 42
    d.text((120, 650), "chora tour, 2026-11-12", font=ImageFont.truetype(MONO, 20), fill=(86, 95, 137))
    image.save(path)
