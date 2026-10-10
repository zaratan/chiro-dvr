"""Launch batdetect with Rust kernels behind detect.py, selected by BATDETECT_RUST.

BATDETECT_RUST=median : background median of detect_frames in Rust, the rest unchanged.
BATDETECT_RUST=fused  : background, residual and per-pixel threshold in one Rust call.
BATDETECT_RUST_THREADS=0 keeps the Rust kernels on one core.
Unset: plain batdetect.
"""

from __future__ import annotations

import itertools
import math
import os
import sys
from collections import deque
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent / "target" / "pylib"))

import batdetect_rs
import numpy as np

from batdetect import detect, parallel

MAX_RUST_SAMPLES = 64
PARALLEL = os.environ.get("BATDETECT_RUST_THREADS", "1") != "0"
python_median = detect.temporal_median
python_detect_frames = detect.detect_frames


def rust_median(frames):
    if frames and all(f.dtype == np.float32 and f.flags.c_contiguous for f in frames):
        return batdetect_rs.temporal_median_f32(list(frames), PARALLEL)
    return python_median(frames)


def fused_detect_frames(frames, fps, cfg, scale=1.0):
    if cfg.target_sigma == 0:
        return python_detect_frames(frames, fps, cfg, scale)
    half = cfg.half_window(fps)
    iterator = iter(frames)
    first = next(iterator, None)
    if first is None:
        return {}
    blur = cfg.target_sigma / scale
    mask = detect.osd_mask(first.shape[1], first.shape[0], cfg, math.ceil(detect.BLUR_REACH * blur))
    if not mask.any():
        raise ValueError("osd_regions mask the whole frame")
    window = deque(maxlen=2 * half + 1)
    detections = {}

    def process(target):
        entries = [e for e in window if abs(e[0] - target) <= half]
        current = next(e for e in entries if e[0] == target)
        sampled = entries[:: cfg.bg_step]
        if len(sampled) > MAX_RUST_SAMPLES:
            raise SystemExit("fused mode keeps at most 64 background samples")
        mean_level = float(np.mean([e[2] for e in sampled]))
        use_noise = cfg.noise_factor > 0 and len(sampled) >= detect.MIN_NOISE_SAMPLES
        gains = [e[2] - mean_level for e in sampled]
        residual, threshold_map = batdetect_rs.residual_threshold(
            [e[1] for e in sampled],
            current[1],
            mask,
            current[2] - mean_level,
            gains,
            float(cfg.threshold),
            float(cfg.noise_factor),
            use_noise,
            PARALLEL,
        )
        threshold = cfg.threshold if threshold_map is None else threshold_map
        detections[target] = detect.find_blobs(residual, target, cfg, scale, threshold)

    index = -1
    for index, gray in enumerate(itertools.chain([first], iterator)):
        work = detect.target_blur(gray, blur)
        window.append((index, work, float(work[mask].mean())))
        if index - half >= 0:
            process(index - half)
    for target in range(max(0, index - half + 1), index + 1):
        process(target)
    return detections


def install(mode: str | None) -> None:
    if mode == "median":
        detect.temporal_median = rust_median
    elif mode == "fused":
        parallel.detect_frames = fused_detect_frames
    elif mode:
        raise SystemExit(f"unknown BATDETECT_RUST={mode}")


install(os.environ.get("BATDETECT_RUST"))

if __name__ == "__main__":
    from batdetect.cli import main

    sys.exit(main())
