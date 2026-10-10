"""Detect a video with the kernels chosen by BATDETECT_RUST and pickle every detection."""

from __future__ import annotations

import pickle
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

import run  # noqa: F401  (installs the kernels)

from batdetect.detect import DetectConfig
from batdetect.parallel import detect_video
from batdetect.video import open_video

video, dest = Path(sys.argv[1]), Path(sys.argv[2])
cfg = DetectConfig()
cap, info = open_video(video, cfg.work_width)
cap.release()
t0 = time.perf_counter()
detections, _ = detect_video(video, cfg, info, 1)
print(f"{time.perf_counter() - t0:.2f} s, {sum(map(len, detections.values()))} detections", file=sys.stderr)
dest.write_bytes(pickle.dumps(detections))
