# Video-to-pixel-art work log

Created: 2026-09-20 01:17 PDT, America/Los_Angeles. This first entry records the earlier work from
the conversation. Exact times were not recorded for those steps.

## Current state

Keep this log focused on decisions, reproducible steps, and results. Store generated metadata
and raw measurements under `output/`, not in `logs/`.

The user selected **four frame workers**, overriding the provisional benchmark recommendations.
`logs/2026-09-20-01-17-video-pixel-art/render-video.py` now defaults to four workers, four Rayon threads per worker, and
uncompressed PPM input between FFmpeg and Rust. Worker count and Rayon threads are independent
options; changing one does not change the other. The pixelation algorithm is unchanged.

The retained script is [render-video.py](render-video.py). It defaults to the whole video from
start to finish; use `--start 4 --duration 3` for the earlier preview. It preserves source frame
timestamps and the final frame duration rather than forcing constant 24 fps. It is still configured
for this portrait 4K source and the selected output dimensions.

The three-second motion test is complete. The user approved four frames: Window light, Down the
aisle, Fare display, and Houses below the mountains. We copied the full-size PNG previews to
Downloads at the user's request. The full 14.712-second video has now been converted and validated. The completed work includes
still-image comparisons and new command-line options. The saved renderer now handles the full video and shorter intervals; selected-frame timestamps
and commands are recorded below.

A palette is the set of colors available to an image. This project builds a separate palette for
each frame. It divides each red, green, and blue channel, or RGB channel, into ranges called
partitions. Combinations of these ranges form color groups called buckets.

| Decision                                                      | Status                                                                                             |
| ------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| Process frames independently, then assemble them into a video | User selected this approach; no shared video palette                                               |
| Pixel resolution                                              | User selected 180 × 320                                                                            |
| Color limit                                                   | User selected 64 RGB colors per frame                                                              |
| Palette partitions                                            | Comparisons use 16 per RGB channel; carry this setting forward to reproduce the selected look      |
| Preserve the existing pixelation implementation               | User requested this; pixel selection and color matching are unchanged                              |
| Run optimized builds                                          | User authorized release builds                                                                     |
| Paid processing                                               | User requires approval before expensive work; none used so far                                     |
| Motion test                                                   | User approved proceeding; rendered source 00:04–00:07 at 24 fps with audio; awaiting visual review |

The motion test uses source 00:04–00:07 and its audio. It includes all 72 original frames at 24
frames per second, or fps. Each pixelated frame is 180 × 320 with 64 colors. The MP4 displays them
at 1080 × 1920. It uses nearest-neighbor enlargement, which copies each pixel into a 6×6 block
without smoothing. The source frames in this interval are spaced at 1/24 second. We did not
duplicate or drop frames. Review the motion before deciding whether to change the frame rate or
colors for the full clip.

## Source and environment

Repository: `/Users/noah/github/pixel-art`. Starting Git HEAD:
`f3812eaaef357cc04789c949d1e78184cb0e1e65`. The code changes described below are uncommitted at the
time of this entry.

Source video, unchanged:

```text
/Users/noah/Downloads/71661760565__22FDC8AB-AC83-415A-8482-3C2CA2959220.MOV
SHA-256: b66300162ee8ca802e3a2e8721c063177c0d3c92a874e398f75712f6f33aaea3
```

FFprobe reported a duration of 14.711667 seconds and a file size of 47,799,307 bytes. The file uses
the HEVC video format and AAC audio format. Stored video dimensions are 3840 × 2160, with rotation
-90 degrees. After rotation, the display dimensions are 2160 × 3840. The average frame rate is
211800/8827, approximately 23.995 fps. This average does not show whether all frames are equally
spaced. The file's color information specifies BT.2020 primaries, which define its color range. It
also specifies HLG transfer, `arib-std-b67`, which defines how stored values represent brightness.
HLG supports high dynamic range, or HDR, video.

Tools used:

- FFmpeg/FFprobe 8.1.2, Homebrew, Apple Silicon.
- Rust `1.98.1 (48a229cea 2026-09-01)`.
- Cargo `1.98.1 (797e8a9bc 2026-08-05)`.
- Python 3.12.14 and Pillow 12.3.0 from the bundled Codex runtime.
- Python executable:
  `/Users/noah/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3`.

Rust was not available through PATH, the shell's list of executable locations. We downloaded the
required Rust tools into `/tmp/pixel-art-toolchain`. We did not change the shell profile. These
temporary tools may disappear. A future machine can use its own matching Rust installation.

## Process and decisions

1. Inspected the video information and a contact sheet, a grid of sample frames. The footage
   includes roadside scenery, mountains, and the bus interior. We used these first frames to choose
   views, not to judge final colors.
2. Proposed one shared palette for the video to reduce color flicker. The user chose separate
   processing for each frame. Keep that choice when converting the video.
3. Inspected the Rust code. Input, output, longest edge, color limit, and RGB partitions were fixed
   in the code. The 32-color setting had only three partitions per channel, or 27 possible buckets.
   It could produce far fewer distinct colors.
4. Added command-line options for input, output, and size. Built in release mode to use compiler
   optimizations. Kept the pixelation algorithm unchanged.
5. Extracted the scenery frame at 00:03 with the HDR conversion below. A temporary 00:01 extraction
   remains, but we did not use it for the comparisons.
6. Rendered 72 × 128, 180 × 320, 432 × 768, and 1080 × 1920 with the original palette defaults. The
   user selected 180 × 320.
7. Added options for color count and RGB partitions. Used 16 partitions per channel for every color
   sample. Only the color limit changed across the six samples. The scenery input filled 414 buckets
   at this setting, enough to test up to 128 palette entries. Increasing the old limit alone would
   not provide enough colors with three partitions.
8. Converted the scenery frame with color limits of 8, 16, 24, 32, 64, and 128. The first five used
   exactly those numbers of visible colors. The 128-entry palette produced 125 visible colors after
   the algorithm chose a color for each output pixel.
9. At the user's request, repeated the same six settings on a frame at 00:07 showing the bus
   interior. All six outputs used exactly their requested numbers of visible colors. This frame
   showed the differences in orange poles, blue seat fabric, and the yellow hat more clearly.
10. The user selected 64 colors. At that point, we had discussed a short motion test but had not
    started it.
11. The user requested this timestamped log and ongoing updates so future readers can understand and
    reproduce the work.

## Code changes

- `src/main.rs`: added `--input`, `--output`, `--size`, `--colors`, and `--partitions`. Added help
  text and checks for invalid arguments. `--size` sets the longest output edge. For this portrait
  source, `--size 320` produces 180 × 320. The output cannot exceed the input dimensions.
- `src/image/palette.rs`: added `palette_with_options(img, output_color_count, partitions)`. The
  existing bucket calculations now accept settings in place of fixed values. Kept `palette(img)` as
  a function that uses the original defaults.
- `src/image/mod.rs`: exported `palette_with_options`.
- Root `README.md`: documented CLI options, defaults, and palette limitations.

Defaults remain the panda input and `output/pixelated.png` output. The default longest edge is 32
pixels. The default color limit is 32, with three partitions per RGB channel. `--colors` accepts
1–256. `--partitions` accepts 1–32.

The algorithm still averages the colors in each RGB bucket. It sorts the buckets by pixel count and
takes up to the requested number. It then finds the closest palette color for each source pixel. For
each output pixel, it chooses the most frequent match in the corresponding source area. Transparency
remains an additional palette entry. Some entries may never be chosen, so the output may use fewer
colors than the limit. We did not add dithering, which uses patterns of pixels to suggest extra
shades. We did not add a shared video palette.

## Reproduce the stills

Commands below run from the repository root. Generated files go under `output/`, which is ignored by
Git. The source MOV is local and is not included in the repository. Re-running these commands
replaces the named generated files.

### Build

```sh
cargo build --release --locked
```

The equivalent build using this session's temporary compiler is:

```sh
CARGO_HOME=/tmp/pixel-art-toolchain/cargo \
RUSTUP_HOME=/tmp/pixel-art-toolchain/rustup \
/tmp/pixel-art-toolchain/cargo/bin/cargo build --release --locked
```

### Extract comparison frames

The installed FFmpeg lacks `zscale`. The first attempt to use it failed before producing a
comparison image. The working filter uses `scale` to convert brightness and color values. It then
applies Hable tone mapping to fit HDR brightness into the output range. Finally, it converts to
sRGB, the color format used for these still images. Both comparisons use exactly the same filter. We
inspected the result visually but did not check it against a calibrated HDR display.

```sh
mkdir -p output/resolution-study output/color-study output/color-study-interior
pixel_source='/Users/noah/Downloads/71661760565__22FDC8AB-AC83-415A-8482-3C2CA2959220.MOV'
pixel_filter='scale=iw:ih:out_transfer=linear:out_primaries=bt709,format=gbrpf32le,tonemap=tonemap=hable:desat=0,scale=iw:ih:in_transfer=linear:out_transfer=iec61966-2-1:out_primaries=bt709,format=rgb24'

ffmpeg -hide_banner -loglevel error -ss 3 -i "$pixel_source" \
  -vf "$pixel_filter" -frames:v 1 -y output/resolution-study/source-03s.png
ffmpeg -hide_banner -loglevel error -ss 7 -i "$pixel_source" \
  -vf "$pixel_filter" -frames:v 1 -y output/color-study-interior/source-07s.png
```

FFmpeg applies the source rotation automatically. These PNGs are 2160 × 3840 RGB.

```text
source-03s.png SHA-256: 54311bece415ec24caed9a565045f4a5c126874c0f4c0ed1ed0695434b073e1c
source-07s.png SHA-256: c6f2f169652f60ea72a8a96785b35d1d711c78bb5f0c4110b4427e01b79c8ccb
```

### Resolution comparison with original palette defaults

```sh
for size in 128 320 768 1920; do
  target/release/pixel-art \
    --input output/resolution-study/source-03s.png \
    --output "output/resolution-study/size-${size}.png" \
    --size "$size" \
    > "output/resolution-study/size-${size}.log" || break
done
```

| CLI size | Actual pixel dimensions | Visible colors |
| -------- | ----------------------- | -------------- |
| 128      | 72 × 128                | 8              |
| 320      | 180 × 320               | 9              |
| 768      | 432 × 768               | 9              |
| 1920     | 1080 × 1920             | 9              |

Each sample completed in under one second locally. This is not a full-video performance estimate.

### Color comparisons

```sh
for colors in 8 16 24 32 64 128; do
  target/release/pixel-art \
    --input output/resolution-study/source-03s.png \
    --output "output/color-study/colors-${colors}.png" \
    --size 320 --colors "$colors" --partitions 16 \
    > "output/color-study/colors-${colors}.log" || break

  target/release/pixel-art \
    --input output/color-study-interior/source-07s.png \
    --output "output/color-study-interior/colors-${colors}.png" \
    --size 320 --colors "$colors" --partitions 16 \
    > "output/color-study-interior/colors-${colors}.log" || break
done
```

Selected settings for future frames:

```sh
target/release/pixel-art --input FRAME.png --output PIXEL_FRAME.png \
  --size 320 --colors 64 --partitions 16
```

### Display previews

Preview enlargement uses Pillow `Image.Resampling.NEAREST`, with no smoothing. The color overviews
show each 180 × 320 sample at 360 × 640. They use two rows: 8/16/24, then 32/64/128. The color
samples below each image are ordered by how many output pixels use each color. Full previews are
1080 × 1920, an exact 6× enlargement. Full previews from the resolution study are 2160 × 3840.

For example, regenerate an individual full preview with:

```python
from PIL import Image

Image.open('output/color-study-interior/colors-64.png').convert('RGB').resize(
    (1080, 1920), Image.Resampling.NEAREST
).save('output/color-study-interior/64-colors-full.png')
```

We assembled the contact sheets with Pillow and Arial from
`/System/Library/Fonts/Supplemental/Arial.ttf`. Their labels and layout are for display only. The
original-size pixelated PNGs are the outputs to compare when repeating the process.

## Artifacts

All links below refer to local files under the ignored `output/` directory; a fresh clone will need
to regenerate them.

- [Resolution overview](../../output/resolution-study/comparison.png).
- [Scenery color overview](../../output/color-study/comparison.png).
- [Interior color overview](../../output/color-study-interior/comparison.png).
- [Selected scenery preview](../../output/color-study/64-colors-full.png).
- [Selected interior preview](../../output/color-study-interior/64-colors-full.png).
- Each study directory also contains a README, native output PNGs, and per-run timing logs.

## Validation and limitations

- Optimized release builds succeeded.
- All comparison output dimensions were checked.
- Color samples were checked for full opacity and visible color counts within their requested caps.
- After adding palette options, we compared the default 180 × 320 output with the earlier resolution
  sample. The decoded pixel values matched byte-for-byte.
- Missing, malformed, zero, and out-of-range CLI inputs were checked and rejected as appropriate.
- `git diff --check` passed after code changes.
- We attempted a formatting check, but the temporary Rust installation lacked `rustfmt`. We did not
  complete an automated formatting check.
- When we created this log, we had not tested video timing, audio alignment, color changes between
  frames, or MP4 encoding. The later motion-test checks are recorded below. The user has not yet
  approved how the motion looks.
- No paid API or cloud rendering was used. Local downloads installed the compiler/dependencies, and
  rendering ran locally.

## Updating this log

Update this log as work proceeds. Keep the current-state section accurate. Append dated entries for
new work. Record user decisions, exact commands and settings, and source time ranges. Record code
changes and their reasons, output paths, and check results. Include runtime, cost, and unresolved
issues. Separate completed work and approved choices from proposals. When a decision changes, keep
the earlier record and state what replaced it.

### 2026-09-20 01:17 PDT: log established

Recorded the conversation history and commands to repeat the still-image processing. At that time,
we saved a copy of the dependency lockfile, which records exact library versions, beside this note.
Work was paused before the motion test while we completed the documentation request.

### 2026-09-20 01:25 PDT: motion test completed

The user approved the motion test. We selected source 00:04–00:07 to show exterior scenery, the
camera turn, and the bus interior. The source contains 353 frames in total. The selected interval
contains 72 frames from timestamp 4.000000 through 6.958333 seconds. FFprobe reports timestamps to
six decimal places. These differ from exact 1/24-second spacing by at most 0.0000003334 seconds.

Added the motion-test renderer, now [render-video.py](render-video.py). Its initial version ran
the existing Rust release executable without changing the pixelation algorithm. That version
performed these steps:

1. Reads frame timestamps and checks that the selected interval matches 1/24-second spacing.
2. Decodes full-resolution RGB frames using the same HDR conversion as the still comparisons, with
   `-fps_mode passthrough`.
3. Writes one temporary full-resolution PNG at a time and runs the Rust pixelator with
   `--size 320 --colors 64 --partitions 16`.
4. Keeps the small pixelated frames. Checks their dimensions, transparency, and color counts.
   Removes the temporary source image after processing.
5. Encodes the frames at 24 fps with nearest-neighbor enlargement to 1080 × 1920. Converts sRGB
   still values to the BT.709 video color format. Uses limited-range YUV420P, which stores less
   color detail than brightness detail. Uses H.264/libx264, preset medium, and CRF 16, the quality
   setting. Adds BT.709 color labels and MP4 faststart, which allows playback before the whole file
   downloads. Video compression can change pixel colors. The 64-color limit applies to the original
   pixelated PNGs.
6. Takes audio from source 00:04–00:07 and re-encodes it as AAC at 192 kb/s. Both streams are
   trimmed to three seconds.
7. Saves source timestamps, frame-processing logs, encoding logs, and output file information. Saves
   a manifest, a run record with settings, commands, timings, and visible color counts.

Command used from the repository root:

```sh
/Users/noah/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3 \
  logs/2026-09-20-01-17-video-pixel-art/render-video.py \
  '/Users/noah/Downloads/71661760565__22FDC8AB-AC83-415A-8482-3C2CA2959220.MOV' --start 4 --duration 3
```

The initial version defaulted to start 4 seconds, duration 3 seconds, and `output/motion-test`.
The current renderer defaults to the whole video; the command above explicitly selects the preview. For another run, pass a new directory such as
`--output output/motion-test-repeat`. The script refuses to reuse an existing `frames` directory so
old frames cannot enter the output. It remains configured for this portrait 4K source; the current version preserves its variable frame timing. It does not yet handle a full video with changing frame intervals. It
requires Pillow, FFmpeg/FFprobe, and an existing `target/release/pixel-art` executable.

Results:

- [Motion-test MP4](../../output/motion-test/motion-test-180x320-64-colors.mp4), approximately 6.2
  MiB.
- [Decoded frame overview](../../output/motion-test/filmstrip.png), sampled every 12 frames. We
  inspected it to confirm that the video moves from the exterior to the interior.
- [Run manifest](../../output/motion-test/manifest.json).
- [Output metadata](../../output/motion-test/output-probe.json).
- [Validation measurements](../../output/motion-test/validation.json).
- Native frames: `output/motion-test/frames/frame-000000.png` through `frame-000071.png`.
- MP4 SHA-256: `f32b3ab39261a26d15c900d02af8bd64612726cfdcb828bd08ccfef65f916166`.
- Reading the input information, extracting frames, and pixelating them took 38.815 seconds. Total
  script runtime was 40.351 seconds, including encoding and output checks.
- No paid processing; all rendering was local.

Validation:

- All 72 native frames are opaque, 180 × 320, and contain exactly 64 visible colors.
- The encoded video is 1080 × 1920, 24 fps, 72 frames, and 3.000000 seconds long.
- Video and audio both start at 0.000000 and have duration 3.000000 seconds.
- Decoded the complete MP4 with FFmpeg to the null output without errors:

```sh
ffmpeg -hide_banner -loglevel error \
  -i output/motion-test/motion-test-180x320-64-colors.mp4 -f null -
```

- Compared the original audio section and exported audio without shifting either in time. Decoded
  both to 44,100 Hz mono float32 PCM, a stream of uncompressed audio samples. Each three-second
  stream contained all 132,300 samples. Pearson correlation, a measure of signal similarity, was
  0.999657. This supports correct audio selection and alignment despite AAC re-encoding. This was a
  numerical check. We did not perform a listening test.
- Python syntax compilation and `git diff --check` passed.

Next, the user reviews the motion and audio. Wait for that review before converting the full video.
If approved, decide how to preserve the full source's timestamps and the duration of its last frame.
Then convert the full clip with the chosen settings. Record the commands and results here.

### 2026-09-20 01:29 PDT: four selected frames for review

The user requested four distinct, interesting frames before converting the full video. We reviewed a
contact sheet with one frame per second from the whole clip. We chose four timestamps with different
views and colors. The contact sheet was only a selection aid. We extracted the final frames at full
source resolution with the same HDR conversion as before.

| Timestamp | Selection                  | Reason                                                                 |
| --------- | -------------------------- | ---------------------------------------------------------------------- |
| 00:00.25  | Window light               | Seat and window geometry, sunlight, and an outside view together       |
| 00:04.00  | Road and mountains         | Broad sky and mountain shapes against detailed road and fence textures |
| 00:09.00  | Fare display               | Close bus details, orange poles, dark screen, and yellow accents       |
| 00:13.50  | Houses below the mountains | Green vegetation, pale buildings, poles, and layered hills             |

All four use the approved `--size 320 --colors 64 --partitions 16` settings. We did not change the
Rust pixelation algorithm or color conversion. The motion test and full video still need user
review.

A temporary selection helper extracted the frames, checked their dimensions and color counts,
and created the previews. That helper was removed during cleanup; the timestamps above and the
HDR filter in this log preserve the steps needed to recreate the images. For example, after
setting `pixel_source` and `pixel_filter` as shown in the extraction section:

```sh
mkdir -p output/selected-frame-rerun
ffmpeg -hide_banner -loglevel error -ss 0.25 -i "$pixel_source" \
  -vf "$pixel_filter" -frames:v 1 -y output/selected-frame-rerun/source.png
target/release/pixel-art --input output/selected-frame-rerun/source.png \
  --output output/selected-frame-rerun/pixelated.png --size 320 --colors 64 --partitions 16
```

Substitute the other recorded timestamps to repeat the other selections. Native images are
180 × 320; previews were enlarged to 1080 × 1920 with nearest-neighbor sampling.

Artifacts:

- [Four-frame overview](../../output/four-frame-study/comparison.png).
- [Window light](../../output/four-frame-study/window-light-full.png).
- [Road and mountains](../../output/four-frame-study/road-and-mountains-full.png).
- [Fare display](../../output/four-frame-study/fare-display-full.png).
- [Houses below the mountains](../../output/four-frame-study/mountain-houses-full.png).
- [Run manifest](../../output/four-frame-study/manifest.json), including source-frame and
  native-output SHA-256 hashes.

Checks confirmed that the four original-size outputs are distinct, opaque, and 180 × 320. Each
contains exactly 64 visible colors. We inspected the overview visually. Script runtime was 7.756
seconds, excluding the first contact sheet. All processing ran locally, with no paid rendering. Wait
for the user's review before the full conversion.

### 2026-09-20 01:38 PDT: replace the road-and-mountains frame

The user approved the houses, window-light, and fare-display selections. They requested a
replacement for Road and mountains. We kept the three approved images byte-for-byte unchanged in a
new output directory. We proposed Down the aisle at 00:07.75. It shows the yellow cap, orange
handrails, and repeating seat backs. We inspected the replacement visually. It uses the same 180 ×
320 resolution, 64 colors, and 16 partitions per RGB channel. We also inspected an exterior frame at
00:11.25. We did not select it because motion blur hid its detail.

The revised selection was written to `output/four-frame-study-v2`, preserving the original
outputs. Reproduce the replacement with the extraction and pixelation commands above using
`-ss 7.75`. The three retained selections use 00:00.25, 00:09.00, and 00:13.50.

- [Replacement frame: Down the aisle](../../output/four-frame-study-v2/bus-aisle-full.png).
- [Revised four-frame overview](../../output/four-frame-study-v2/comparison.png).
- [Revised run manifest](../../output/four-frame-study-v2/manifest.json).

All four revised outputs are distinct, opaque, 180 × 320, and use exactly 64 visible colors. We
compared the three retained PNG files with the previous set. They are identical. We did not change
the pixelation algorithm or use paid processing. At this point, the replacement and full conversion
still needed user review.

### 2026-09-20 01:41 PDT: approved stills copied to Downloads

The user accepted the replacement and requested the four frames in `~/Downloads`. We copied the
revised set's full-size PNG previews. Each is 1080 × 1920, enlarged exactly 6× from the 180 × 320
pixel art with nearest-neighbor sampling:

| Source in `output/four-frame-study-v2/` | Destination                                           |
| --------------------------------------- | ----------------------------------------------------- |
| `window-light-full.png`                 | `/Users/noah/Downloads/pixel-art-window-light.png`    |
| `bus-aisle-full.png`                    | `/Users/noah/Downloads/pixel-art-down-the-aisle.png`  |
| `fare-display-full.png`                 | `/Users/noah/Downloads/pixel-art-fare-display.png`    |
| `mountain-houses-full.png`              | `/Users/noah/Downloads/pixel-art-mountain-houses.png` |

All destination names were available. We created the files without overwriting any existing files.
SHA-256 checksum comparisons confirmed that each copy matched its source. We did not render the
images again or change their pixels. We had not started the full-video conversion.

### 2026-09-20 01:43 PDT: benchmark frame concurrency and uncompressed inputs

In the side conversation, the user authorized tests with one, two, and four frame workers. Each
worker processes a frame separately, so multiple workers can run at once. The user also authorized
replacing PNG as the format passed to the pixelator. The tests measured CPU and memory use and
checked that output pixels stayed the same. We added a separate benchmark script without changing
the main render script or Rust algorithm. Nine trials ran one after another on the same 72 frames.
All 648 output frames exactly matched the original RGBA values, which include red, green, blue, and
transparency.

The PNG baseline processed frames one at a time and took 21.507 seconds in the measured test. PPM is
an uncompressed image format tested as a replacement for PNG. Four PPM workers took 5.746–6.108
seconds. Each used four Rayon threads, which allow work inside the Rust pixelator to run at the same
time. This setup averaged 14.36–14.57 busy cores out of 18 reported logical CPUs. Two PPM workers
took 6.383–6.793 seconds. The recommendation is to use four PPM workers in the next render. These
measurements exclude the initial source inspection and final video/audio encoding. They do not
replace the original complete motion-test timing.

Python's peak RSS, the memory held in RAM, was about 149 MiB with four PPM workers. The largest peak
for any single child process was about 1.84 GiB. The available process-monitoring permissions did
not allow measurement of all processes' combined memory use at one time. No paid processing was
used.

See the
[benchmark report and reproduction commands](../2026-09-20-01-43-frame-worker-benchmark/README.md)
for the measured results.
This side task did not add those changes to the main render script.

### 2026-09-20 01:51 PDT: expand concurrency search beyond four workers

At the user's request, we tested 4, 6, 8, 9, 12, and 16 PPM workers. We ran three new repeats each
for 4/6/8/9. All fourteen additional trials matched the original output pixels. Six workers with
three Rayon threads each had the lowest median, the middle result of the repeated runs. That median
was 5.816 seconds. Eight workers with two threads each had a 6.008-second median. The measured
ranges overlap, and later runs slowed across settings. These results suggest a useful range but do
not prove which setting is fastest. We tested twelve and sixteen workers only once each.

The proposed default is now six PPM workers with three Rayon threads each. This replaces the earlier
four-worker recommendation. The main render script has not changed. See the expanded
[benchmark report](../2026-09-20-01-43-frame-worker-benchmark/README.md) for the search results.
Across both stages, 1,656 frames from 23 trials matched exactly. No paid processing.

### 2026-09-20 01:57 PDT: correct the experiment by holding Rayon fixed

The user instructed us to vary only worker count. Tested 4, 8, 16, and 32 frame workers with exactly four Rayon threads per process in every run, repeating each setting three times in changed order. Median frame-processing times were 6.563, 6.029, 5.822, and 5.748 seconds respectively. The 16-to-32 improvement was only 1.27%, smaller than observed variation, so stopped at 32 rather than continuing to 64.

Provisional operating point is now 16 frame workers with 4 Rayon threads each. This supersedes the earlier six-worker recommendation from the confounded experiment. All 864 output frames from this controlled sweep matched the originals exactly. Full-video probing and final encoding are excluded from these timings; no main-render integration was performed. No paid processing.

See the [controlled benchmark report](../2026-09-20-01-54-fixed-rayon-worker-sweep/README.md) for the measured results and reproduction commands.

### 2026-09-20 02:04 PDT: user selects four workers; integrate into the render script

The user chose four workers despite the small gains at higher counts. Updated `logs/2026-09-20-01-17-video-pixel-art/render-video.py` to use four concurrent frame conversions by default, with `RAYON_NUM_THREADS=4` per Rust process and uncompressed PPM handoff. The CLI exposes independent `--workers` and `--rayon-threads` options, both defaulting to four. No automatic coupling between those settings remains in the renderer. This user choice supersedes all provisional benchmark recommendations.

Each worker gets a unique temporary input and per-frame log. The scheduler bounds in-flight work to the worker limit plus one decoded frame. Output filenames, combined logs, and manifest color counts remain in source order. Temporary inputs are removed after processing, and the Rust pixelation implementation is unchanged. The temporary benchmark helper used explicit thread settings for the controlled tests; it was later removed during cleanup.

Reproduction and validation command, using the new defaults:

```sh
/Users/noah/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3 \
  logs/2026-09-20-01-17-video-pixel-art/render-video.py --start 4 --duration 3 \
  '/Users/noah/Downloads/71661760565__22FDC8AB-AC83-415A-8482-3C2CA2959220.MOV' \
  --output output/motion-test-four-workers
```

Use a fresh output directory for another run. Explicit settings, if desired, are `--workers 4 --rayon-threads 4`.

Verification: all 72 native frames match the original motion-test RGBA pixels exactly and in order. The encoded MP4 has 72 frames at 24 fps, 1080 × 1920 dimensions, and three-second video/audio streams. A complete FFmpeg decode reported no errors. Script syntax and rejection of zero worker/thread counts were checked; `git diff --check` passed.

The manifest now separates timing stages. This run spent 17.264 seconds scanning source timestamps and 5.705 seconds processing frames, with 24.466 seconds total including encoding and output probing. These are observed local timings, not guaranteed future runtime.

- [Updated renderer](render-video.py).
- [Validation run manifest](../../output/motion-test-four-workers/manifest.json).
- [Validation video](../../output/motion-test-four-workers/motion-test-180x320-64-colors.mp4).

The original motion-test artifacts remain unchanged. No full-video conversion or paid processing was performed.

### 2026-09-20 02:06 PDT: keep one reusable script beside the work log

At the user's request, moved the retained [motion renderer](render-video.py) into this
work-log folder and removed the temporary selection and benchmark helpers, their cached bytecode,
and the empty `scripts/` directory. Updated the renderer's repository-root lookup for its new
location and corrected the documented commands. Defaults remain four frame workers, four Rayon
threads per worker, and PPM handoff. No processing behavior changed.

Logs retain the decisions, settings, measurements, and useful reproduction steps. Generated data
belongs under `output/`; do not add disposable helpers or raw measurement dumps to the repository.

### 2026-09-20 02:19 PDT: rename the renderer and support the full video

The user approved full-length support and renaming. Renamed the retained script to
[render-video.py](render-video.py). No additional helper scripts were kept.

Defaults are now start zero and duration through the final video frame. `--start` and `--duration`
remain available for previews; a requested duration beyond the source end is clipped to the end.
The renderer includes the frame visible at the requested start and shortens its display duration
when the start falls between source timestamps. Four workers, four Rayon threads each, PPM handoff,
180 × 320 native pixels, and 64 colors are unchanged.

Replaced the fixed-24-fps requirement with the actual source timestamps. This file has a 1/600-second
source time base and 353 frames. Selected frame indices drive extraction so frames cannot be
silently dropped or duplicated. The output uses a 1/60000-second time base, retaining this source's
timestamps exactly. The final frame ends at 14.7116666667 seconds.

Encoding uses a generated `frames.ffconcat` file under `output/`. Concat parses durations in
microseconds, so the renderer rounds absolute boundaries first and takes their differences. The
initial preview test caught accumulated rounding error when each duration was rounded separately;
that was fixed before the successful preview and full render. The final packet duration is set
explicitly, with B-frame reordering disabled, to preserve the last frame's display time. x264's
nominal rate comes from the source metadata, while presentation timestamps remain variable.
Audio uses the same selected interval.

Run the complete video from the repository root, with a fresh output directory:

```sh
/Users/noah/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3 \
  logs/2026-09-20-01-17-video-pixel-art/render-video.py \
  '/Users/noah/Downloads/71661760565__22FDC8AB-AC83-415A-8482-3C2CA2959220.MOV' \
  --output output/full-pixel-video
```

For a preview, add `--start 4 --duration 3` and choose another output directory. Omitting `--output`
uses `output/pixel-video`. Generated filenames now use `pixel-art-180x320-64-colors.mp4`.

Validation:

- Re-rendered the three-second preview. All 72 native frames match the original pixels in order,
  and every encoded timestamp matches exactly.
- Rendered all 353 full-video frames. All are opaque 180 × 320 images containing 64 visible colors.
- Every encoded full-video timestamp matches its source timestamp, with zero measured error.
  Final-frame duration and total video duration match the source interval.
- Verified that full-video frames 96–167 match the original 72-frame preview exactly.
- Video and audio both start at zero. Video duration is 14.711667 seconds; audio is 14.710998 seconds.
  The first 14.7 seconds of decoded audio correlate with source audio at zero lag by
  0.999844, consistent with aligned AAC re-encoding.
- Decoded the entire MP4 without FFmpeg errors and visually inspected a six-frame filmstrip.
- Checked script syntax, repository-root lookup, and rejection of invalid start/duration/worker
  arguments. `git diff --check` passed.

Observed full-run timing: 17.443 seconds inspecting the source,
26.935 seconds processing frames, and 51.015 seconds
total including encoding and output validation. All work ran locally with no paid processing.

- [Full video](/Users/noah/Downloads/pixel-art-180x320-64-colors.mp4).
- [Verified preview](../../output/video-render-preview-verified/pixel-art-180x320-64-colors.mp4).
- [Full-video filmstrip](../../output/full-pixel-video/filmstrip.png).

Detailed machine measurements and metadata remain under `output/`, not in this log folder. The
failed initial preview directory was removed; original review artifacts were preserved. The full
video is ready for the user's visual review.

At the user's request, moved the finished full video to `/Users/noah/Downloads/pixel-art-180x320-64-colors.mp4`. Generated frame and validation data remain under `output/full-pixel-video/`; its original run manifest records the pre-move output path.
