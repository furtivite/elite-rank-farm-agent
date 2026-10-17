#!/usr/bin/env python3

import struct
import zlib
from pathlib import Path

SUPERSAMPLE = 4
SIZES = (16, 32, 48, 256)

GRAPHITE = (27, 36, 48, 255)
ORANGE = (255, 133, 27, 255)
ORANGE_DARK = (176, 78, 0, 255)
TRANSPARENT = (0, 0, 0, 0)

class Canvas:
    def __init__(self, size):
        self.size = size
        self.pixels = [TRANSPARENT] * (size * size)

    def fill_polygon(self, points, color):
        ys = [y for _, y in points]
        for y in range(max(0, int(min(ys))), min(self.size, int(max(ys)) + 1)):
            crossings = []
            for index in range(len(points)):
                x1, y1 = points[index]
                x2, y2 = points[(index + 1) % len(points)]
                if (y1 <= y < y2) or (y2 <= y < y1):
                    crossings.append(x1 + (y - y1) * (x2 - x1) / (y2 - y1))
            crossings.sort()
            for start, end in zip(crossings[0::2], crossings[1::2]):
                for x in range(max(0, int(start)), min(self.size, int(end) + 1)):
                    self.pixels[y * self.size + x] = color

    def fill_rounded_rect(self, left, top, right, bottom, radius, color):
        for y in range(max(0, int(top)), min(self.size, int(bottom) + 1)):
            for x in range(max(0, int(left)), min(self.size, int(right) + 1)):
                dx = max(left + radius - x, x - (right - radius), 0)
                dy = max(top + radius - y, y - (bottom - radius), 0)
                if dx * dx + dy * dy <= radius * radius:
                    self.pixels[y * self.size + x] = color

    def downsample(self, factor):
        size = self.size // factor
        result = Canvas(size)
        for y in range(size):
            for x in range(size):
                r = g = b = a = 0
                for sy in range(factor):
                    for sx in range(factor):
                        pr, pg, pb, pa = self.pixels[
                            (y * factor + sy) * self.size + (x * factor + sx)
                        ]
                        r += pr * pa
                        g += pg * pa
                        b += pb * pa
                        a += pa
                count = factor * factor
                if a == 0:
                    result.pixels[y * size + x] = TRANSPARENT
                else:
                    result.pixels[y * size + x] = (
                        round(r / a),
                        round(g / a),
                        round(b / a),
                        round(a / count),
                    )
        return result

    def to_png(self):
        raw = bytearray()
        for y in range(self.size):
            raw.append(0)
            for x in range(self.size):
                raw.extend(self.pixels[y * self.size + x])

        def chunk(tag, data):
            body = tag + data
            return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body))

        header = struct.pack(">IIBBBBB", self.size, self.size, 8, 6, 0, 0, 0)
        return (
            b"\x89PNG\r\n\x1a\n"
            + chunk(b"IHDR", header)
            + chunk(b"IDAT", zlib.compress(bytes(raw), 9))
            + chunk(b"IEND", b"")
        )

def draw(size, badge, glyph):
    canvas = Canvas(size)
    unit = size / 32

    if badge is not None:
        canvas.fill_rounded_rect(0.5 * unit, 0.5 * unit, size - 0.5 * unit,
                                 size - 0.5 * unit, 7 * unit, badge)

    thickness = 6.5 * unit
    for top in (5.0, 17.0):
        y = top * unit
        height = 10 * unit
        left = 8 * unit
        tip = left + 9 * unit

        canvas.fill_polygon(
            [
                (left, y),
                (tip, y + height / 2),
                (left, y + height),
                (left + thickness, y + height),
                (tip + thickness, y + height / 2),
                (left + thickness, y),
            ],
            glyph,
        )

    return canvas

def write_ico(path, badge, glyph):
    images = []
    for size in SIZES:
        big = draw(size * SUPERSAMPLE, badge, glyph)
        images.append((size, big.downsample(SUPERSAMPLE).to_png()))

    header = struct.pack("<HHH", 0, 1, len(images))
    entries = b""
    offset = len(header) + len(images) * 16
    for size, png in images:
        entries += struct.pack(
            "<BBBBHHII",
            0 if size >= 256 else size,
            0 if size >= 256 else size,
            0,
            0,
            1,
            32,
            len(png),
            offset,
        )
        offset += len(png)

    path.write_bytes(header + entries + b"".join(png for _, png in images))
    print(f"{path.name}: {path.stat().st_size} байт")

if __name__ == "__main__":
    here = Path(__file__).parent
    write_ico(here / "app.ico", GRAPHITE, ORANGE)
    write_ico(here / "window-dark.ico", None, ORANGE)
    write_ico(here / "window-light.ico", None, ORANGE_DARK)
