# Reproduce the rotating RGB color space visualization

Use this recipe to make the dark browser visualization for a new image. It preserves the continuous
rotation, draggable view, cell spacing control, source dots, palette rings, and room around the cube.
The generated HTML opens directly in a browser and works offline. It needs no Codex plugin or server.

The files needed to reproduce this view are together in this log entry:

- [visualize.py](visualize.py) generates the artwork, data, and browser page.
- [visualize.html](visualize.html) contains the view and its styles.
- [requirements.txt](requirements.txt) pins the Python dependencies.

Keep generated images, metadata, and Python environments under `output/`.

## Set up once

Run these commands from the repository root with Rust and Python 3.11 or newer installed:

```sh
cargo build --release --locked
python3 -m venv output/.venv-rgb
output/.venv-rgb/bin/python -m pip install \
  -r logs/2026-09-20-rgb-color-space/requirements.txt
```

Rebuild the Rust binary after changing the palette algorithm. The generator uses
`target/release/pixel-art` by default. Pass `--binary /path/to/pixel-art` to use another build.

## Generate a new result

Start with the full-resolution source image and choose its crop first. A 16:9 crop produces 256 × 144
pixel art with these settings:

```sh
output/.venv-rgb/bin/python logs/2026-09-20-rgb-color-space/visualize.py \
  --input /path/to/source-crop.png \
  --output-dir output/my-image-rgb \
  --size 256 \
  --colors 16 \
  --partitions 4
```

Open `output/my-image-rgb/rgb-color-space.html` in a browser. On macOS:

```sh
open -a 'Google Chrome' output/my-image-rgb/rgb-color-space.html
```

The output directory must be new so earlier results stay intact. The generator writes:

| File | Contents |
| --- | --- |
| `rgb-color-space.html` | Self-contained browser visualization, ready to share |
| `pixel-art.png` | Native-resolution artwork without a palette bar |
| `source.png` | Source after EXIF orientation and removal of fully transparent outer edges |
| `palette.log` | Complete output from the Rust CLI, including its selected palette |
| `data.json` | Cell counts and means, sampled source colors, and selected palette |
| `reproduction.json` | Settings, command, dependency versions, and source/code/binary hashes |

Keep this directory with any result you want to reproduce. `output/` is ignored by Git. The generator,
template, dependency list, and this guide live together in this log entry so they can be committed.
For long-term reproduction, also keep the matching source revision and Rust binary. Hashes identify
these files but cannot restore them.

The defaults are 16 colors, 4 partitions per channel, 2,500 sampled pixels, and random seed 409.
Change `--colors`, `--partitions`, `--samples`, or `--seed` as needed. This viewer supports 1 through 16
partitions per channel. Four partitions give the clearest
view of individual cells. More partitions add cubic drawing cost: 8 means 512 cells; 16 means 4,096.
The requested color count is a ceiling. An image can produce fewer selected colors.

Use a PNG or another format Pillow can decode. The script handles EXIF orientation but does not
convert color profiles to sRGB. For comparable RGB values, prepare an sRGB image before running it.
Fully transparent pixels do not contribute to the palette or samples. Other pixels contribute their
RGB values without alpha weighting, matching the Rust palette code.

## Reproduce IMG_0409

The chosen source is the upright photograph cropped to `3024 × 1701`, starting at `x=0, y=600`.
The existing crop is:

```text
output/IMG_0409-resolution-study/source-crop.png
SHA-256: 7536875230a19bc2577acb23734b44f84671814487b1864c54758a4aa4238086
```

Use that saved crop for the closest reproduction:

```sh
output/.venv-rgb/bin/python logs/2026-09-20-rgb-color-space/visualize.py \
  --input output/IMG_0409-resolution-study/source-crop.png \
  --output-dir output/IMG_0409-rgb-reproduced \
  --size 256 --colors 16 --partitions 4 --samples 2500 --seed 409
```

If the crop is missing, install `libheif` and ImageMagick, then recreate it from the original photo:

```sh
mkdir -p output/IMG_0409-resolution-study
heif-convert "$HOME/Downloads/IMG_0409.HEIC" output/IMG_0409-resolution-study/decoded.png
magick output/IMG_0409-resolution-study/decoded.png \
  -crop 3024x1701+0+600 +repage output/IMG_0409-resolution-study/source-crop.png
```

Check that the decoded image is upright and `3024 × 4032` before cropping. The macOS `sips` decoder
produced a black image during this study, so use `heif-convert`. Decoder versions can change pixel
values or file encoding. Keep the saved crop if exact reproduction matters.

Expected results are 256 × 144 artwork pixels, 64 starting cells, 33 occupied cells, and these 16
selected colors in CLI order:

```text
114,155,202   101,95,27     90,99,84      167,160,82
214,225,229   161,145,47    153,179,201   38,47,46
224,208,106   53,66,68      139,160,177   180,199,210
200,181,86    249,235,144   138,116,26    121,132,80
```

The reusable generator was checked against the original result. The artwork pixels, all cell counts
and means, all 2,500 samples, and the selected palette matched exactly.

## How the data and view work

[visualize.py](visualize.py) runs the real Rust CLI and reads its palette from `palette.log`. Do not
extract the palette from the pixelated PNG. Some selected colors might not appear in the final art.
Do not build the distribution from the pixelated PNG either. Use the full-resolution crop.

The generator follows the RGB bucket rules in `src/image/palette.rs`:

```text
partition_width = ceil(255 / partitions)
channel_bucket = min(channel, 254) // partition_width
cell_index = red_bucket + partitions * green_bucket + partitions² * blue_bucket
```

Filled cells show occupied starting buckets. Their tint is their mean RGB color, rounded to the
nearest integer. Small dots are source pixels sampled without replacement using NumPy's
`default_rng(409)`. Sampling pixels rather than unique colors preserves color frequency.
Rings show the final palette produced by greedy bucket merging. The merge picks the pair with the
smallest increase in squared RGB error, weighted by pixel count. Rust truncates final channel means
to integers. Filled cells show the initial partitions, not the irregular groups formed by merging.

[visualize.html](visualize.html) contains the browser drawing code and its styles. Numeric
data and counts replace the `__RGB_*__`, `__CELL_COUNT__`, and `__PALETTE_COUNT__` placeholders.
The styles came from the original browser export and are saved locally with the template.

The initial yaw is -0.72 radians and tilt is 0.43 radians. Cell spacing starts at 18 and ranges from
0 to 48. A full automatic turn takes about 28 seconds. Dragging pauses it. The button resumes it.
Reduced-motion preferences disable automatic rotation on load.

The page stays dark, centers the view at up to 960 pixels wide, and uses available vertical space.
The camera fits a sphere around the entire cube and its axis-label anchors. It reserves additional
space for label text and point rings. The fit stays stable through rotation and updates when the
window or spacing changes. Preserve this calculation when editing the view to prevent clipping.

When changing the template, check a full turn at spacing 0, 18, and 48. Also drag to both tilt limits.
Check desktop and phone widths, pause/resume, the source-color checkbox, and palette-ring selection.
All cells, labels, and rings should remain inside the canvas throughout the turn.
