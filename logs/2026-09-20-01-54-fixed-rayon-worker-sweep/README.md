# Worker-count sweep with fixed Rayon threads

Recorded 2026-09-20 01:57 PDT. Supersedes worker-count recommendations from the earlier experiment, which varied both frame workers and Rayon threads per worker.

## Controlled comparison

The user explicitly requested changing only worker count. All 12 trials here use **four Rayon threads per Rust process**. Only the frame-worker limit varies: 4, 8, 16, and 32.

Held fixed: uncompressed PPM handoff, the same frozen release executable, source interval 00:04–00:07, all 72 source frames, HDR filter, resolution 180 × 320, 64 colors, 16 palette partitions, frame verification, bounded streaming scheduler, and FFmpeg configuration. Tests ran sequentially, never concurrently. No source-code changes were made for this sweep.

Scope: elapsed time includes decode/HDR conversion, handoff files, Rust conversion, and comparison of every output pixel with the original motion-test reference. Initial full-video probing and final MP4/audio encoding are excluded. Machine activity and thermal state were not controlled; repeated ranges quantify observed variation but are not statistical confidence intervals.

## Results

| Frame workers | Rayon threads each | Runs | Median seconds | Range seconds | Median busy cores |
| --- | ---: | ---: | ---: | ---: | ---: |
| 4 | 4 | 3 | 6.563 | 5.592–6.746 | 14.29 |
| 8 | 4 | 3 | 6.029 | 5.400–6.105 | 15.54 |
| 16 | 4 | 3 | 5.822 | 5.605–6.408 | 15.26 |
| 32 | 4 | 3 | 5.748 | 5.513–6.148 | 15.07 |

Four was not the fastest configuration in this controlled sample. The median time falls through sixteen workers, then improves only 1.27% at thirty-two. That final difference is small relative to the observed run-to-run variation. Stopped at thirty-two because the curve had plateaued, as requested; sixty-four was not tested.

**Practical recommendation: sixteen frame workers, keeping Rayon fixed at four threads per worker.** Thirty-two has the lowest measured median, but the evidence does not establish a reliable benefit over sixteen. This is a practical operating point for this clip and machine, not a proven global optimum. It supersedes the earlier provisional six-worker recommendation, which came from changing two settings at once.

All 864 frames across these 12 trials matched the original decoded RGBA pixels exactly. CPU accounting includes Python, FFmpeg, and all Rust processes. Worker count is an upper limit; the streaming producer may not keep that many processes active simultaneously. Do not infer actual simultaneous threads from worker limit × Rayon pool size.

## Run order

```text
Round 1: 4, 8, 16, 32
Round 2: 32, 16, 8, 4
Round 3: 8, 32, 4, 16
```

Reversing and changing order helps avoid assigning every larger worker count a later run, but it does not eliminate thermal or background-load effects. Later runs generally took longer. The table above preserves the results; raw timings remain under `output/worker-benchmark-2026-09-20/`.

## Future controlled comparisons

The temporary benchmark helper was removed during cleanup. Keep Rayon fixed and vary only worker
count with the retained renderer, using a fresh output directory for each run:

```sh
pixel_python=/Users/noah/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3
pixel_source='/Users/noah/Downloads/71661760565__22FDC8AB-AC83-415A-8482-3C2CA2959220.MOV' --start 4 --duration 3
for workers in 4 8 16 32; do
  "$pixel_python" logs/2026-09-20-01-17-video-pixel-art/render-video.py "$pixel_source" --start 4 --duration 3 \
    --workers "$workers" --rayon-threads 4 --output "output/worker-rerun-${workers}"
done
```

Compare `frame_processing_seconds` in each generated manifest. This excludes source probing and
final encoding, but unlike the historical benchmark does not include comparison against reference
pixels. Repeat with changed run order to assess variation. These commands describe a future
comparison using the maintained renderer, not an exact reproduction of the retired helper.

- Raw artifacts: `output/worker-benchmark-2026-09-20/fixed-r4-<workers>w-<round>/`, including frame images, per-frame Rust logs, FFmpeg log, and full result JSON.
- Frozen executable SHA-256: `3861753836e1ba04ffa984654939ae4941e54d1e7ebdd2f67253e50beaa34938`.
- No changes to Rust, the main render script, or output style. No paid processing. Integration into the main render workflow has not been performed by this side task.
