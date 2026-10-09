#!/usr/bin/env python3
"""Render our original Ember SVG to committed PNG/ICO/ICNS assets. Needs Pillow.

No app build or network access. Only the deliberately small M/C/Z SVG vocabulary
used by ember.svg is supported, so raster & vector icons share one source.
"""

from pathlib import Path
import re
import xml.etree.ElementTree as ET

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "assets/app-icon"
APP_ID = "com.orthiclabs.ember"
NS = "{http://www.w3.org/2000/svg}"


def contour(data):
    tokens = re.findall(r"[MCZ]|-?\d+(?:\.\d+)?", data)
    points = []
    cursor = 0
    while cursor < len(tokens):
        command = tokens[cursor]
        cursor += 1
        if command == "Z":
            break
        count = 2 if command == "M" else 6 if command == "C" else 0
        if not count or cursor + count > len(tokens):
            raise ValueError("unsupported or incomplete icon path")
        values = [float(value) for value in tokens[cursor:cursor + count]]
        cursor += count
        if command == "M":
            points.append(tuple(values))
        else:
            start = points[-1]
            for step in range(1, 121):
                t = step / 120
                points.append(tuple((1-t)**3 * start[axis] + 3*(1-t)**2*t*values[axis]
                                    + 3*(1-t)*t*t*values[axis+2] + t**3*values[axis+4]
                                    for axis in range(2)))
    return points


def render(size, macos=False):
    scale = size * 4 / (636 if macos else 512)
    margin = 62 if macos else 0
    image = Image.new("RGBA", (size * 4, size * 4))
    draw = ImageDraw.Draw(image)
    artwork = ET.parse(ASSETS / "ember.svg").getroot()
    for shape in artwork:
        if shape.tag == NS + "rect":
            box = (margin*scale, margin*scale, (512+margin)*scale, (512+margin)*scale)
            draw.rounded_rectangle(box, radius=float(shape.attrib["rx"])*scale, fill=shape.attrib["fill"])
        elif shape.tag == NS + "path":
            points = [((x+margin)*scale, (y+margin)*scale) for x, y in contour(shape.attrib["d"])]
            draw.polygon(points, fill=shape.attrib["fill"])
    return image.resize((size, size), Image.Resampling.LANCZOS)


def main():
    render(1024).save(ASSETS / "ember-1024.png")
    render(512, macos=True).save(ASSETS / "ember-macos-512.png")
    for size in (16, 24, 32, 48, 64, 128, 256, 512):
        target = ASSETS / f"hicolor/{size}x{size}/apps/{APP_ID}.png"
        target.parent.mkdir(parents=True, exist_ok=True)
        render(size).save(target)
    target = ASSETS / f"hicolor/scalable/apps/{APP_ID}.svg"
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes((ASSETS / "ember.svg").read_bytes())
    sizes = (16, 20, 24, 32, 40, 48, 64, 128, 256)
    images = [render(size) for size in sizes]
    images[-1].save(ASSETS / "ember.ico", sizes=[(size, size) for size in sizes], append_images=images[:-1])
    # Pillow's ICNS encoder is portable; use exact macOS-margin renders for each size.
    render(1024, macos=True).save(ASSETS / "ember.icns", append_images=[render(size, macos=True) for size in (128, 256, 512)])
    print("Ember icon assets rendered from original SVG.")


if __name__ == "__main__":
    main()
