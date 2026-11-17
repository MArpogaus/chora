#!/usr/bin/python3
"""Print the color of a pixel of an 8-bit RGBA PNG file as "r g b a".

Usage: tests/png-pixel.py FILE X Y
"""

import struct
import sys
import zlib


def rows(data):
    """Decode the rows of an 8-bit RGBA or RGB PNG file; RGB gets alpha 255."""
    assert data[:8] == b"\x89PNG\r\n\x1a\n", "no PNG file"
    pos, idat, width, height, color = 8, b"", 0, 0, 6
    while pos < len(data):
        size, kind = struct.unpack(">I4s", data[pos:pos + 8])
        chunk = data[pos + 8:pos + 8 + size]
        if kind == b"IHDR":
            width, height, depth, color = struct.unpack(">IIBB", chunk[:10])
            assert depth == 8 and color in (2, 6), "not 8-bit RGB or RGBA"
        elif kind == b"IDAT":
            idat += chunk
        pos += 12 + size
    raw, stride = zlib.decompress(idat), width * (4 if color == 6 else 3)
    prev = bytearray(stride)
    for r in range(height):
        start = r * (stride + 1)
        kind, line = raw[start], bytearray(raw[start + 1:start + 1 + stride])
        for i in range(stride):
            step = 4 if color == 6 else 3
            a = line[i - step] if i >= step else 0
            b = prev[i]
            c = prev[i - step] if i >= step else 0
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
        prev = line
        if color == 2:
            line = bytearray(b"".join(bytes(line[i:i + 3]) + b"\xff" for i in range(0, stride, 3)))
        yield line


def main():
    path, x, y = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
    with open(path, "rb") as f:
        data = f.read()
    for r, line in enumerate(rows(data)):
        if r == y:
            print(*line[x * 4:x * 4 + 4])
            return


if __name__ == "__main__":
    main()
