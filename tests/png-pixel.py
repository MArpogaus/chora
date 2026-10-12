#!/usr/bin/python3
"""Print the color of a pixel of an 8-bit RGBA PNG file as "r g b a".

Usage: tests/png-pixel.py FILE X Y
"""

import struct
import sys
import zlib


def rows(data):
    """Decode the rows of an 8-bit RGBA PNG file."""
    assert data[:8] == b"\x89PNG\r\n\x1a\n", "no PNG file"
    pos, idat, width, height = 8, b"", 0, 0
    while pos < len(data):
        size, kind = struct.unpack(">I4s", data[pos:pos + 8])
        chunk = data[pos + 8:pos + 8 + size]
        if kind == b"IHDR":
            width, height, depth, color = struct.unpack(">IIBB", chunk[:10])
            assert (depth, color) == (8, 6), "not 8-bit RGBA"
        elif kind == b"IDAT":
            idat += chunk
        pos += 12 + size
    raw, stride = zlib.decompress(idat), width * 4
    prev = bytearray(stride)
    for r in range(height):
        start = r * (stride + 1)
        kind, line = raw[start], bytearray(raw[start + 1:start + 1 + stride])
        for i in range(stride):
            a = line[i - 4] if i >= 4 else 0
            b = prev[i]
            c = prev[i - 4] if i >= 4 else 0
            if kind == 1:
                line[i] = (line[i] + a) & 255
            elif kind == 2:
                line[i] = (line[i] + b) & 255
            elif kind == 3:
                line[i] = (line[i] + (a + b) // 2) & 255
            elif kind == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                best = a if pa <= pb and pa <= pc else b if pb <= pc else c
                line[i] = (line[i] + best) & 255
        yield line
        prev = line


def main():
    path, x, y = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
    with open(path, "rb") as f:
        data = f.read()
    for r, line in enumerate(rows(data)):
        if r == y:
            print(*line[x * 4:x * 4 + 4])
            return


main()
