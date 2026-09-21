#!/usr/bin/env python3
"""Generate pixel art and its standalone, rotating RGB partition visualization."""

import argparse
import hashlib
import json
import math
import platform
import re
import subprocess
from pathlib import Path

import numpy as np
import PIL
from PIL import Image, ImageOps


SCRIPT_DIR = Path(__file__).resolve().parent
REPO_DIR = SCRIPT_DIR.parents[1]


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def parse_args():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, required=True, help="Source image, before pixelation")
    parser.add_argument("--output-dir", type=Path, required=True, help="New directory for all results")
    parser.add_argument("--size", type=int, default=256, help="Longest artwork edge, default 256")
    parser.add_argument("--colors", type=int, choices=range(1, 257), metavar="1..256", default=16)
    parser.add_argument("--partitions", type=int, choices=range(1, 17), metavar="1..16", default=4)
    parser.add_argument("--seed", type=int, default=409)
    parser.add_argument("--samples", type=int, default=2500)
    parser.add_argument("--binary", type=Path, default=REPO_DIR / "target/release/pixel-art")
    args = parser.parse_args()

    if args.size < 1 or args.samples < 1 or args.seed < 0:
        parser.error("Size and sample count must be positive; seed must be nonnegative.")

    if not args.binary.is_file():
        parser.error("Build the CLI first with cargo build --release --locked, or pass --binary.")

    if args.output_dir.exists():
        parser.error("Choose a new --output-dir to preserve earlier results.")

    return args


def make_data(pixels, palette, partitions, sample_count, seed):
    partition_width = math.ceil(255 / partitions)
    channels = np.minimum(pixels, 254).astype(np.int64) // partition_width
    indices = channels[:, 0] + partitions * channels[:, 1] + partitions**2 * channels[:, 2]
    counts = np.bincount(indices, minlength=partitions**3)
    totals = [
        np.bincount(indices, weights=pixels[:, channel], minlength=partitions**3)
        for channel in range(3)
    ]
    cells = []

    for index, count in enumerate(counts):
        color = [int(np.rint(total[index] / count)) for total in totals] if count else [128] * 3
        cells.append({
            "index": [index % partitions, index // partitions % partitions, index // partitions**2],
            "count": int(count),
            "color": color,
        })

    random = np.random.default_rng(seed)
    sample_indices = random.choice(len(pixels), size=min(sample_count, len(pixels)), replace=False)

    return {
        "partitions": partitions,
        "cells": cells,
        "samples": pixels[sample_indices].tolist(),
        "palette": palette,
    }


def main():
    args = parse_args()
    template_path = SCRIPT_DIR / "visualize.html"

    with Image.open(args.input) as original:
        source = ImageOps.exif_transpose(original).convert("RGBA")

    bounds = source.getchannel("A").getbbox()
    if bounds is None:
        raise ValueError("The source has no visible pixels.")

    source = source.crop(bounds)
    rgba = np.asarray(source).reshape(-1, 4)
    pixels = rgba[rgba[:, 3] != 0, :3]

    args.output_dir.mkdir(parents=True)
    source_path = args.output_dir / "source.png"
    artwork_path = args.output_dir / "pixel-art.png"
    source.save(source_path)

    command = [
        str(args.binary.resolve()),
        "--input", str(source_path.resolve()),
        "--output", str(artwork_path.resolve()),
        "--size", str(args.size),
        "--colors", str(args.colors),
        "--partitions", str(args.partitions),
    ]
    result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    (args.output_dir / "palette.log").write_text(result.stdout)
    if result.returncode:
        raise RuntimeError(f"pixel-art failed; see {args.output_dir / 'palette.log'}")

    matches = re.findall(r"space\[\s*\d+\].*?Rgba\(\[(\d+), (\d+), (\d+), 255\]\)", result.stdout)
    palette = [[int(channel) for channel in color] for color in matches]
    if not palette or len(palette) > args.colors:
        raise ValueError("Could not read the selected palette from the CLI log. Check its log format.")

    data = make_data(pixels, palette, args.partitions, args.samples, args.seed)
    occupied = sum(cell["count"] > 0 for cell in data["cells"])
    data_text = json.dumps(data, separators=(",", ":"))
    (args.output_dir / "data.json").write_text(data_text + "\n")

    summary = (
        f"{args.partitions} × {args.partitions} × {args.partitions} cells · "
        f"{occupied} occupied · {len(palette)} selected colors"
    )
    document = template_path.read_text()
    for placeholder, value in {
        "__RGB_DATA__": data_text,
        "__RGB_SUMMARY__": summary,
        "__CELL_COUNT__": str(args.partitions**3),
        "__PALETTE_COUNT__": str(len(palette)),
    }.items():
        document = document.replace(placeholder, value)

    html_path = args.output_dir / "rgb-color-space.html"
    html_path.write_text(document)

    with Image.open(artwork_path) as artwork:
        artwork_size = list(artwork.size)

    manifest = {
        "input": str(args.input.resolve()),
        "input_sha256": sha256(args.input),
        "normalized_source_sha256": sha256(source_path),
        "source_size": list(source.size),
        "artwork_size": artwork_size,
        "colors_requested": args.colors,
        "colors_selected": len(palette),
        "partitions": args.partitions,
        "occupied_cells": occupied,
        "sample_count": len(data["samples"]),
        "seed": args.seed,
        "command": command,
        "binary_sha256": sha256(args.binary),
        "generator_sha256": sha256(Path(__file__)),
        "template_sha256": sha256(template_path),
        "python": platform.python_version(),
        "numpy": np.__version__,
        "pillow": PIL.__version__,
    }
    (args.output_dir / "reproduction.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"{summary}\n{html_path.resolve()}")


if __name__ == "__main__":
    main()
