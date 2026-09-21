#!/usr/bin/env python3
"""Render the full video or a selected interval with the existing Rust pixelator.

Requires FFmpeg/FFprobe, Pillow, and target/release/pixel-art. Run from any
directory. Defaults to four frame workers and four Rayon threads per worker,
using uncompressed PPM handoff. Preserves source frame timing.
"""

import argparse
from collections import deque
from concurrent.futures import ThreadPoolExecutor
from fractions import Fraction
import json
import math
import os
from pathlib import Path
import subprocess
import tempfile
import time

from PIL import Image


ROOT = Path(__file__).resolve().parents[2]
OUTPUT_TIMESCALE = 60000
HDR_FILTER = (
    "scale=iw:ih:out_transfer=linear:out_primaries=bt709,"
    "format=gbrpf32le,tonemap=tonemap=hable:desat=0,"
    "scale=iw:ih:in_transfer=linear:out_transfer=iec61966-2-1:"
    "out_primaries=bt709,format=rgb24"
)
DISPLAY_FILTER = (
    "scale=1080:1920:flags=neighbor,"
    "scale=in_transfer=iec61966-2-1:out_transfer=bt709:"
    "in_primaries=bt709:out_primaries=bt709:out_color_matrix=bt709:"
    "in_range=pc:out_range=tv,format=yuv420p,setsar=1"
)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("--start", type=float, default=0, help="Start offset in seconds")
    parser.add_argument("--duration", type=float, help="Seconds to render; omit to reach the end")
    parser.add_argument("--output", type=Path, default=ROOT / "output/pixel-video")
    parser.add_argument("--workers", type=int, default=4)
    parser.add_argument("--rayon-threads", type=int, default=4)
    args = parser.parse_args()
    if not math.isfinite(args.start) or args.start < 0 or (
        args.duration is not None and (not math.isfinite(args.duration) or args.duration <= 0)
    ):
        parser.error("Start must be nonnegative and duration must be positive")
    if args.workers < 1 or args.rayon_threads < 1:
        parser.error("Workers and Rayon threads must be positive")
    source = args.input.resolve()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    frames = output / "frames"
    # A separate fresh directory prevents stale frames from entering the encode.
    frames.mkdir()
    started = time.monotonic()
    commands = []

    def run(command, **kwargs):
        commands.append([str(part) for part in command])
        return subprocess.run(command, check=True, **kwargs)

    probe = json.loads(run([
        "ffprobe", "-v", "error", "-select_streams", "v:0",
        "-show_streams", "-show_frames", "-show_entries",
        "stream=width,height,time_base,r_frame_rate,duration:stream_side_data=rotation:"
        "frame=best_effort_timestamp,duration",
        "-of", "json", source,
    ], capture_output=True).stdout)
    (output / "source-timestamps.json").write_text(json.dumps(probe, indent=2))
    stream = probe["streams"][0]
    width, height = stream["width"], stream["height"]
    rotation = next((d["rotation"] for d in stream.get("side_data_list", [])
                     if "rotation" in d), 0)
    if abs(rotation) % 180 == 90:
        width, height = height, width
    if (width, height) != (2160, 3840):
        raise ValueError("This renderer is configured for the selected portrait 4K source")
    source_time_base = Fraction(stream["time_base"])
    source_frames = probe["frames"]
    if not source_frames:
        raise ValueError("Source contains no video frames")
    absolute_times = [int(f["best_effort_timestamp"]) * source_time_base for f in source_frames]
    origin = absolute_times[0]
    times = [t - origin for t in absolute_times]
    if any(b <= a for a, b in zip(times, times[1:])):
        raise ValueError("Source frame timestamps must be strictly increasing")
    last_duration = int(source_frames[-1].get("duration", 0)) * source_time_base
    if last_duration <= 0:
        raise ValueError("Source does not provide the final frame duration")
    source_end = times[-1] + last_duration
    clip_start = Fraction(str(args.start))
    clip_end = source_end if args.duration is None else min(
        source_end, clip_start + Fraction(str(args.duration))
    )
    if clip_start >= clip_end:
        raise ValueError("Requested interval is outside the video")
    source_ends = times[1:] + [source_end]
    indices = [i for i, (start, end) in enumerate(zip(times, source_ends))
               if start < clip_end and end > clip_start]
    # Include the frame already visible at --start, clipping its display time.
    selected_times = [max(times[i], clip_start) - clip_start for i in indices]
    duration = clip_end - clip_start
    selected_ends = selected_times[1:] + [duration]
    durations = [end - start for start, end in zip(selected_times, selected_ends)]
    expected = len(indices)
    output_ticks = [round(t * OUTPUT_TIMESCALE) for t in selected_times]
    final_ticks = round(duration * OUTPUT_TIMESCALE) - output_ticks[-1]
    if final_ticks <= 0 or any(b <= a for a, b in zip(output_ticks, output_ticks[1:])):
        raise ValueError("Frame intervals are too short for the output time base")
    probe_seconds = time.monotonic() - started

    decode = [
        "ffmpeg", "-hide_banner", "-loglevel", "error", "-i", str(source),
        "-map", "0:v:0", "-vf",
        f"select=between(n\\,{indices[0]}\\,{indices[-1]}),{HDR_FILTER}",
        "-frames:v", str(expected), "-fps_mode", "passthrough", "-f", "rawvideo",
        "-pix_fmt", "rgb24", "pipe:1",
    ]
    commands.append(decode)
    count = 0
    visible_colors = []
    worker_environment = dict(os.environ, RAYON_NUM_THREADS=str(args.rayon_threads))
    worker_logs = output / "worker-logs"
    worker_logs.mkdir()

    def convert_frame(index, data, scratch):
        frame = Path(scratch) / f"source-{index:06d}.ppm"
        target = frames / f"frame-{index:06d}.png"
        log = worker_logs / f"frame-{index:06d}.log"
        command = [
            str(ROOT / "target/release/pixel-art"), "--input", str(frame),
            "--output", str(target), "--size", "320", "--colors", "64",
            "--partitions", "16",
        ]
        try:
            with frame.open("wb") as handoff:
                handoff.write(f"P6\n{width} {height}\n255\n".encode("ascii"))
                handoff.write(data)
            with log.open("w") as worker_log:
                subprocess.run(command, check=True, stdout=worker_log,
                               stderr=subprocess.STDOUT, env=worker_environment)
            with Image.open(target) as result:
                if result.size != (180, 320) or result.getextrema()[3] != (255, 255):
                    raise ValueError("Unexpected pixelated dimensions or opacity")
                colors = len(result.getcolors(180 * 320))
                if colors > 64:
                    raise ValueError("Pixelated frame exceeds the color limit")
            return colors, command, log
        finally:
            frame.unlink(missing_ok=True)

    processing_started = time.monotonic()
    with (output / "decode.log").open("wb") as errors, \
            (output / "pixelation.log").open("w") as pixel_log, \
            tempfile.TemporaryDirectory(prefix="pixel-video-") as scratch, \
            ThreadPoolExecutor(max_workers=args.workers) as pool:
        decoder = subprocess.Popen(decode, stdout=subprocess.PIPE, stderr=errors)
        pending = deque()

        def collect(future):
            colors, command, log = future.result()
            visible_colors.append(colors)
            commands.append(command)
            pixel_log.write(log.read_text())
            if len(visible_colors) % 12 == 0:
                print(f"Pixelated {len(visible_colors)}/{expected} frames", flush=True)

        try:
            while True:
                data = decoder.stdout.read(width * height * 3)
                if not data:
                    break
                if len(data) != width * height * 3:
                    raise ValueError("Incomplete raw frame from FFmpeg")
                # Keep at most `workers` conversions pending, plus this one
                # decoded frame. Collect in source order for logs and metadata.
                if len(pending) >= args.workers:
                    collect(pending.popleft())
                pending.append(pool.submit(convert_frame, count, data, scratch))
                count += 1
            for future in pending:
                collect(future)
            if decoder.wait() != 0:
                raise RuntimeError("FFmpeg extraction failed; see decode.log")
        finally:
            decoder.stdout.close()
            if decoder.poll() is None:
                decoder.terminate()
                decoder.wait()
    if count != expected:
        raise ValueError(f"Expected {expected} extracted frames, received {count}")
    processing_seconds = time.monotonic() - processing_started
    render_seconds = time.monotonic() - started
    # A fine image-demuxer time base prevents concat from rounding timestamps
    # to 1/25 second. Relative filenames are generated here, never user paths.
    timeline = output / "frames.ffconcat"
    lines = ["ffconcat version 1.0"]
    # Concat parses durations in microseconds. Round absolute boundaries first
    # so repeated 1/24-second durations cannot accumulate rounding error.
    boundaries_us = [round(t * 1000000) for t in selected_times] + [round(duration * 1000000)]
    for index in range(expected):
        duration_us = boundaries_us[index + 1] - boundaries_us[index]
        lines.extend([
            f"file frames/frame-{index:06d}.png",
            f"option framerate {OUTPUT_TIMESCALE}",
            f"duration {duration_us // 1000000}.{duration_us % 1000000:06d}",
        ])
    timeline.write_text("\n".join(lines) + "\n")
    video = output / "pixel-art-180x320-64-colors.mp4"
    # With B-frames disabled, packet and source order agree. Set the final
    # packet's duration explicitly: concat's duration controls the NEXT file's
    # timestamp, and cannot on its own hold the final still for the right time.
    packet_duration_filter = f"setts=duration=if(eq(N\\,{expected - 1})\\,{final_ticks}\\,DURATION)"
    nominal_rate = Fraction(stream["r_frame_rate"])
    if nominal_rate <= 0:
        raise ValueError("Invalid source nominal frame rate")
    with (output / "encode.log").open("w") as encode_log:
        run([
            "ffmpeg", "-hide_banner", "-y", "-f", "concat", "-safe", "0",
            "-i", timeline, "-ss", str(float(origin + clip_start)), "-t",
            str(float(duration)), "-i", source, "-map", "0:v:0", "-map", "1:a:0?",
            "-vf", DISPLAY_FILTER, "-c:v", "libx264", "-preset", "medium",
            "-fps_mode", "passthrough", "-enc_time_base", f"1:{OUTPUT_TIMESCALE}",
            "-video_track_timescale", str(OUTPUT_TIMESCALE), "-bf", "0",
            "-x264-params", f"fps={nominal_rate}:force-cfr=0",
            "-bsf:v", packet_duration_filter,
            "-crf", "16", "-pix_fmt", "yuv420p", "-color_primaries", "bt709",
            "-color_trc", "bt709", "-colorspace", "bt709", "-color_range", "tv",
            "-c:a", "aac", "-b:a", "192k",
            "-movflags", "+faststart", video,
        ], stdout=encode_log, stderr=subprocess.STDOUT)
    metadata = json.loads(run([
        "ffprobe", "-v", "error", "-count_frames", "-show_streams", "-show_format",
        "-show_frames", "-show_entries", "frame=media_type,pts,duration",
        "-of", "json", video,
    ], capture_output=True).stdout)
    (output / "output-probe.json").write_text(json.dumps(metadata, indent=2))
    v = next(s for s in metadata["streams"] if s["codec_type"] == "video")
    audio = [s for s in metadata["streams"] if s["codec_type"] == "audio"]
    if (v["width"], v["height"], int(v["nb_read_frames"])) != (1080, 1920, expected):
        raise ValueError("Encoded video dimensions or frame count do not match")
    output_frames = [f for f in metadata["frames"] if f["media_type"] == "video"]
    if len(output_frames) != expected:
        raise ValueError("Decoded frame count does not match the source interval")
    encoded_time_base = Fraction(v["time_base"])
    timestamp_errors = [abs(int(f["pts"]) * encoded_time_base - wanted)
                        for f, wanted in zip(output_frames, selected_times)]
    timing_tolerance = Fraction(1, OUTPUT_TIMESCALE)
    if max(timestamp_errors) > timing_tolerance:
        raise ValueError("Encoded frame timestamps differ from the source")
    encoded_end = (int(output_frames[-1]["pts"]) + int(output_frames[-1]["duration"])) * encoded_time_base
    if abs(encoded_end - duration) > timing_tolerance:
        raise ValueError("Final frame duration does not match the source interval")
    for s in audio:
        if abs(float(s.get("start_time", 0))) > 0.025 or abs(float(s["duration"]) - float(duration)) > 0.025:
            raise ValueError("Encoded audio timing does not match the selected interval")
    manifest = {
        "source": str(source), "start_seconds": args.start,
        "duration_seconds": float(duration), "frames": count,
        "average_fps": float(Fraction(v["avg_frame_rate"])),
        "timing": "source_timestamps", "source_time_base": str(source_time_base),
        "source_frame_indices": indices,
        "frame_timestamps_seconds": [float(t) for t in selected_times],
        "frame_durations_seconds": [float(d) for d in durations],
        "max_timestamp_error_seconds": float(max(timestamp_errors)),
        "pixel_dimensions": [180, 320], "display_dimensions": [1080, 1920],
        "colors": 64, "partitions": 16, "palette_per_frame": True,
        "workers": args.workers, "rayon_threads_per_worker": args.rayon_threads,
        "handoff_format": "ppm",
        "visible_colors_per_frame": visible_colors,
        "source_probe_seconds": probe_seconds,
        "frame_processing_seconds": processing_seconds,
        "extract_and_pixelate_seconds": render_seconds,
        "total_seconds": time.monotonic() - started,
        "commands": commands, "video": str(video),
    }
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2))
    print(f"Finished: {video}", flush=True)
    print(f"Total time: {manifest['total_seconds']:.1f} seconds", flush=True)


if __name__ == "__main__":
    main()
