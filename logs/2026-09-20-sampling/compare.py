"""Render every source image with each CLI sampling mode and build a comparison."""

import html
import io
import re
import subprocess
from pathlib import Path

from PIL import Image, ImageCms, ImageDraw, ImageFont


ROOT = Path(__file__).resolve().parents[2]
OUTPUT = ROOT / "output/sampling"
MODES = ["mode", "mean", "center", "ink"]
LABELS = ["Source", "Mode · most common", "Mean · average", "Center · one pixel", "Ink · 15% dark"]
EXTENSIONS = {".png", ".jpg", ".jpeg", ".webp", ".tif", ".tiff", ".bmp"}
BACKGROUND = "#181818"
TEXT = "#eeeeea"
SRGB = ImageCms.ImageCmsProfile(ImageCms.createProfile("sRGB"))
FONT = "/System/Library/Fonts/Supplemental/Arial.ttf"


def display_image(image):
    profile = image.info.get("icc_profile")
    if not profile:
        return image.convert("RGBA")

    return ImageCms.profileToProfile(
        image.convert("RGBA"),
        ImageCms.ImageCmsProfile(io.BytesIO(profile)),
        SRGB,
        outputMode="RGBA",
    )


def render(source_path):
    directory = OUTPUT / source_path.stem
    directory.mkdir(parents=True, exist_ok=True)
    variants = []
    logs = []
    palette = None

    with Image.open(source_path) as source:
        profile = source.info.get("icc_profile")
        original = display_image(source)
        original = original.crop(original.getchannel("A").getbbox())

    for mode in MODES:
        destination = directory / f"{mode}.png"
        command = [
            str(ROOT / "target/release/pixel-art"),
            "--input", str(source_path),
            "--output", str(destination),
            "--size", "32",
            "--colors", "16",
            "--partitions", "3",
            "--sampling", mode,
        ]
        result = subprocess.run(command, capture_output=True, text=True, check=True)
        logs.append(f"sampling: {mode}\n{result.stdout}{result.stderr}")
        colors = re.findall(r"Rgba\(\[(\d+), (\d+), (\d+), 255\]\)", result.stdout)
        colors = [tuple(map(int, color)) + (255,) for color in colors]
        if palette is None:
            palette = colors
        assert colors == palette, f"Palette changed for {source_path.name}: {mode}"

        allowed = set(colors) | {(0, 0, 0, 0)}
        with Image.open(destination) as image:
            assert image.info.get("icc_profile") == profile
            assert max(image.size) == 32
            assert all(color in allowed for _, color in image.convert("RGBA").getcolors())
            if variants:
                assert image.size == variants[0].size
            variants.append(display_image(image))

    (directory / "render.log").write_text("\n".join(logs))
    print(f"{source_path.name}: all four modes; palette, dimensions, and profile verified")
    return original, variants


def comparison(source_path, original, variants):
    preview = Image.new("RGB", (1300, 340), BACKGROUND)
    draw = ImageDraw.Draw(preview)
    title = ImageFont.truetype(FONT, 21)
    label = ImageFont.truetype(FONT, 17)
    width, height = variants[0].size
    draw.text((24, 12), f"{source_path.name} · {width} × {height}", font=title, fill=TEXT)

    for column, (name, image) in enumerate(zip(LABELS, [original] + variants)):
        left = 24 + column * 256
        draw.text((left, 48), name, font=label, fill=TEXT)
        if column == 0:
            scale = min(224 / image.width, 240 / image.height)
            size = (round(image.width * scale), round(image.height * scale))
            resampling = Image.Resampling.NEAREST if max(image.size) <= 128 else Image.Resampling.LANCZOS
        else:
            size = (width * 7, height * 7)
            resampling = Image.Resampling.NEAREST

        image = image.resize(size, resampling)
        x = left + (224 - image.width) // 2
        y = 82 + (240 - image.height) // 2
        preview.paste(image, (x, y), image)

    preview.save(OUTPUT / source_path.stem / "comparison.png", icc_profile=SRGB.tobytes())
    return preview


def gallery(sources):
    sections = []
    for source in sources:
        name = html.escape(source.name)
        directory = html.escape(source.stem, quote=True)
        links = " · ".join(
            f'<a href="{directory}/{mode}.png">{mode}</a>' for mode in MODES
        )
        sections.append(
            f'<section><img src="{directory}/comparison.png" alt="{name}: source and four sampling modes">'
            f'<p>Native PNGs: {links}</p></section>'
        )

    document = '''<!doctype html>
<html lang="en">
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Sampling comparisons</title>
<style>
body { margin: 24px auto; padding: 0 20px; max-width: 1300px;
       background: #181818; color: #eeeeea; font: 16px system-ui, sans-serif; }
h1 { font-size: 26px; }
section { margin: 32px 0; overflow-x: auto; }
img { display: block; width: 100%; min-width: 900px; }
a { color: #82c7ff; }
p { color: #bbbbbb; }
</style>
<h1>Sampling comparisons</h1>
<p>32-pixel longest edge · Up to 16 colors · 3 RGB partitions · Same palette in every mode</p>
'''
    (OUTPUT / "index.html").write_text(document + "\n".join(sections) + "\n</html>\n")


def main():
    OUTPUT.mkdir(parents=True, exist_ok=True)
    sources = sorted(
        (path for path in (ROOT / "images").iterdir() if path.suffix.lower() in EXTENSIONS),
        key=lambda path: path.name.lower(),
    )
    rows = []
    for source_path in sources:
        original, variants = render(source_path)
        rows.append(comparison(source_path, original, variants))

    for start in range(0, len(rows), 4):
        page = rows[start:start + 4]
        preview = Image.new("RGB", (1300, 50 + 340 * len(page)), BACKGROUND)
        draw = ImageDraw.Draw(preview)
        heading = ImageFont.truetype(FONT, 21)
        draw.text((24, 12), "32-pixel longest edge · 16-color limit · 3 RGB partitions", font=heading, fill=TEXT)
        for row, image in enumerate(page):
            preview.paste(image, (0, 50 + row * 340))
        preview.save(OUTPUT / f"preview-{start // 4 + 1}.png", icc_profile=SRGB.tobytes())

    gallery(sources)
    print(f"{len(sources) * len(MODES)} renders saved under {OUTPUT}")


if __name__ == "__main__":
    main()
