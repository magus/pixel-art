# Sampling comparison

[compare.py](compare.py) renders every supported image in `images/` with all four CLI sampling modes.
Each render uses a 32-pixel longest edge, a 16-color limit, three RGB partitions, and no palette bar.
The script calls the Rust implementation for every output. It checks that modes share the same
palette and dimensions, use only palette colors, and preserve the source profile.

- `mode` chooses the most frequent palette match. This remains the CLI default.
- `mean` averages source RGB values in each block, then finds the nearest palette color.
- `center` takes the center source pixel of each block, then finds the nearest palette color.
- `ink` takes the 15th percentile of palette matches ordered from dark to light. Dark strokes can
  survive even when most of the block is light. This deliberately favors dark features.

The average uses source RGB values, without converting them to linear light. It weights by alpha
so invisible colors cannot affect the average. The mean and ink rules choose transparency when
average alpha coverage is less than half. Center and mode keep their own transparency rules.
Brightness order uses `299R + 587G + 114B`. Equal brightness uses palette order.

The CLI uses fixed integer block widths and heights. These can leave gaps when the source dimensions
do not divide evenly by the output dimensions. All four modes preserve those boundaries.
Complete area coverage is a separate possible improvement.

## Reproduce

Run from the repository root after building the CLI. Use the existing Python environment from the
[RGB visualization setup](../2026-09-20-rgb-color-space/README.md), which supplies Pillow:

```sh
cargo build --release --locked
output/.venv-rgb/bin/python logs/2026-09-20-sampling/compare.py
```

The comparison uses the macOS Arial font. Generated files stay under `output/sampling/`:

- Each source gets a folder containing four native PNGs, `comparison.png`, and `render.log`.
- `index.html` shows every comparison and links to the native PNGs.
- `preview-1.png`, `preview-2.png`, and subsequent pages group up to four comparisons per page.

Native PNGs preserve the source profile. Comparison images convert to sRGB for consistent display.
The source column trims transparent borders to match the CLI crop. The CLI reads the original source
file directly, including JPEG decoding.

## Initial experiment

The first comparison used a line drawing, a horse photo, and a sprite. Averaging preserved the
drawing's glasses, nose, and mouth using gray pixels. The ink rule also preserved facial features,
but thickened them and darkened the photo. Center sampling kept some fine marks but broke others.
These examples supported offering a choice of sampling rules. They did not establish one rule as
best for every image.

After implementation, all 11 current source images produced identical pixels with the old renderer,
the new default, and explicit `--sampling mode`. All 44 mode outputs passed the palette, size, and
profile checks. The Rust test suite passed all 22 tests.
