# Frame concurrency and image handoff benchmark

**Superseded worker-count recommendation:** the later [fixed-Rayon sweep](../2026-09-20-01-54-fixed-rayon-worker-sweep/README.md) changed only frame-worker count and now provisionally recommends sixteen workers with four Rayon threads each. Earlier four/six-worker recommendations below came from confounded comparisons and should not be treated as the current default.

Recorded September 20, 2026, America/Los_Angeles. This was an explicitly authorized experiment in the side conversation. The main render script and pixelation implementation were not changed.

**Updated recommendation:** the expanded search below provisionally favors six PPM workers with three Rayon threads each. The original four-worker conclusion is retained as the initial result, not the current recommendation.

## Question and result

Would converting several independent frames concurrently improve CPU utilization and elapsed time? Does replacing the temporary PNG with an uncompressed format help?

Yes. Recommend four frame workers with four Rayon threads each, using uncompressed PPM input. This was the fastest tested configuration at 5.746–6.108 seconds for 72 frames, versus 21.507 seconds for the strictly serial PNG baseline. Two PPM workers used less CPU and Python memory, but took 6.383–6.793 seconds. These are small repeated samples, not a statistical confidence interval.

All 648 output frames across nine runs were compared at the decoded RGBA pixel level against the original motion-test frames and matched exactly. PPM is supported by the existing image crate, so no Rust changes were needed. The next main-render integration remains a separate step.

## Controlled inputs and scope

- Source: `/Users/noah/Downloads/71661760565__22FDC8AB-AC83-415A-8482-3C2CA2959220.MOV`.
- Same source interval 00:04–00:07, all 72 frames, no frame-rate conversion.
- Same HDR conversion as the still and motion studies, same 2160 × 3840 RGB data.
- Same pixelation settings: `--size 320 --colors 64 --partitions 16`.
- Same release binary copied into the benchmark output directory before testing, to avoid interference from concurrent main-task builds.
- Same output validation against `output/motion-test/frames/frame-000000.png` through `frame-000071.png`.
- Each trial runs in a fresh Python process and uses a fresh temporary directory. Files are removed as workers finish, with bounded frame read-ahead. Final native PNGs are kept for inspection.
- Timed scope: FFmpeg decode and HDR conversion, frame transfer, temporary input writing, Rust processes, and pixel-equivalence checks. Initial full-video timestamp probing and final MP4/audio encoding are excluded.
- The historical 38.815-second timing included the initial source probe and came from a different run. Do not claim that the complete motion-test workflow now takes six seconds or infer an exact end-to-end speedup from these results.
- CPU accounting covers the benchmark Python process, FFmpeg, and all Rust children. 100% means one busy CPU core. Reported logical CPU count from `os.cpu_count()` was 18. Capacity percentages use that count and do not account for unequal performance of different core types.
- CPU measurements are averages from user + system CPU seconds divided by elapsed time, not instantaneous utilization samples.

## Results

`png-serial` reads, converts, and checks each frame before reading the next, matching the original handoff order. Other trials let a dedicated thread read ahead by one frame while one, two, or four worker threads write inputs and wait for separate Rust processes. One-worker read-ahead alone made little difference in these runs.

| Trial | Rayon threads per worker | Elapsed seconds | Average busy cores | CPU capacity | Python peak RSS MiB | Largest child peak RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| png-serial | 18 | 21.507 | 4.22 | 23.4% | 107.6 | 1879.9 |
| png-1w | 18 | 21.286 | 4.23 | 23.5% | 108.1 | 1880.1 |
| png-2w | 9 | 12.694 | 7.17 | 39.8% | 164.6 | 1879.6 |
| png-4w | 4 | 9.472 | 10.34 | 57.5% | 277.7 | 1880.7 |
| ppm-1w | 18 | 8.451 | 9.24 | 51.4% | 74.7 | 1879.3 |
| ppm-2w | 9 | 6.383 | 12.70 | 70.6% | 101.4 | 1879.2 |
| ppm-2w-repeat | 9 | 6.793 | 12.54 | 69.7% | 100.5 | 1879.0 |
| ppm-4w | 4 | 6.108 | 14.36 | 79.8% | 148.8 | 1879.5 |
| ppm-4w-repeat | 4 | 5.746 | 14.57 | 80.9% | 148.8 | 1879.7 |

The first matrix ran PNG then PPM at each worker count, in the order 1, 2, 4. Follow-up runs were strictly serial PNG, four-worker PPM again, then two-worker PPM again. Trials never ran concurrently with each other. Other unrelated machine activity was not controlled.

Peak memory values use `getrusage`: Python's own peak RSS and the largest individual child-process peak. Live process monitoring was unavailable in the sandbox. These figures must not be added and described as a measured simultaneous total. The largest child peak remained around 1.84 GiB; Python rose from roughly 101 MiB for two PPM workers to 149 MiB for four. Exact combined process-tree peak memory was not measured.

## Where time changed

For one-worker runs, summed temporary-input writing time fell from 9.338 seconds for PNG to 0.175 seconds for PPM. Summed Rust subprocess wall time fell from 11.067 to 7.449 seconds, including its input decode and process overhead. The raw input files are larger but avoid compression and decompression.

With multiple workers, per-worker stage times overlap and must not be summed into an end-to-end duration. More workers increase concurrent serial work such as input reading and palette building; Rayon still parallelizes the existing cell-matching loop. Each worker gets a capped Rayon pool to avoid starting four unrestricted pools.

## Repeating the experiment

The temporary benchmark helper was removed during repository cleanup. Its measured results,
input settings, and run order are preserved here. The retained [motion renderer](../2026-09-20-01-17-video-pixel-art/render-video.py)
uses the selected PPM handoff and accepts independent `--workers` and `--rayon-threads` options.
For future comparisons, change only the setting under test and compare its
`frame_processing_seconds` measurement, excluding source probing and final encoding. Its timing
scope differs slightly from the historical helper because pixel-equivalence comparisons were
included in the original benchmark; do not treat new timings as exact reproductions.

## Artifacts and verification

- [Complete results including per-frame timings](../../output/worker-benchmark-2026-09-20/results.json).
- Each trial directory contains all 72 output PNGs, Rust timing logs, FFmpeg error log, and its result JSON.
- Script syntax was checked and `git diff --check` passed. Actual runs exercised both supported handoff formats and all worker configurations.
- No paid processing, changes to pixelation math, or changes to the main render script.

```text
Frozen release binary SHA-256: 3861753836e1ba04ffa984654939ae4941e54d1e7ebdd2f67253e50beaa34938
Benchmark script SHA-256: e41d17e2edd6ff65aa5ba7c0e95a4187e94932245ebdd2be6f53f46dbbe5ec14
```

## 2026-09-20 01:51 PDT: expand the worker-count search

The user asked whether eight workers had been tested and proposed searching for the optimum. The initial study had stopped at four. Ran 14 additional trials: bracketed with 4/8/16, refined with 6/12, checked nine because nine workers can each use two Rayon threads within an 18-thread budget, and repeated 4/6/8/9. No other benchmark configuration changed. All 1,008 additional frames matched; the entire study now covers 23 trials and 1,656 matching frames.

Current provisional recommendation: **six PPM workers with three Rayon threads each**. Six had the lowest median among configurations repeated three times in this search, with less Python memory than eight or nine. This supersedes the initial four-worker recommendation. It is not a proven global optimum: ranges overlap, later runs generally became slower, system activity and thermal state were not controlled, and twelve/sixteen were only sampled once. The evidence supports a broad useful range around six to nine workers. A full-video workload may have a different optimum.

Only the new search runs are summarized below; earlier runs remain in the original table. Single-run rows should not be interpreted as equally reliable estimates.

| Workers | Rayon threads per worker | Runs | Median seconds | Range seconds | Median busy cores | Max Python peak RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 4 | 4 | 3 | 6.384 | 5.617–7.025 | 14.26 | 149.0 |
| 6 | 3 | 3 | 5.816 | 5.713–6.698 | 14.96 | 197.1 |
| 8 | 2 | 3 | 6.008 | 5.496–6.437 | 14.96 | 244.1 |
| 9 | 2 | 3 | 6.342 | 5.450–6.564 | 14.92 | 268.6 |
| 12 | 1 | 1 | 5.801 | 5.801–5.801 | 14.20 | 338.6 |
| 16 | 1 | 1 | 5.719 | 5.719–5.719 | 14.73 | 434.2 |

The source-processing scope still excludes initial timestamp scanning and final encoding. CPU includes FFmpeg and Python as well as Rust. Rayon thread count is a per-worker setting; FFmpeg manages its own threads independently. Memory columns remain separate process peaks, not a combined simultaneous measurement.

Exact new trial order, as worker count / Rayon threads / repetition:

```text
4/4/1, 8/2/1, 16/1/1,
12/1/1, 6/3/1, 8/2/2,
9/2/1, 6/3/2, 4/4/2, 8/2/3, 4/4/3, 6/3/3,
9/2/2, 9/2/3
```

The helper used for these historical trials has been removed. The recorded thread settings and
run order remain above; use the retained renderer for future controlled comparisons as described
earlier in this note.

The table above preserves the medians and ranges. Raw measurements remain under `output/worker-benchmark-2026-09-20/`. The main renderer and Rust implementation were unchanged by this experiment.
