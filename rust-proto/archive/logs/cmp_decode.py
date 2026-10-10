import sys
from pathlib import Path

import numpy as np

from batdetect.video import open_video, read_frames, to_work_gray

raw = np.fromfile(sys.argv[2], dtype=np.uint8)
cap, info = open_video(Path(sys.argv[1]), 960)
n = raw.size // (info.width * info.height * 3)
rs = raw.reshape(n, info.height, info.width, 3)
worst = worst_gray = same = same_gray = 0
for k, frame in enumerate(read_frames(cap)):
    if k >= n:
        break
    d = np.abs(frame.astype(int) - rs[k].astype(int)).max()
    g = np.abs(to_work_gray(frame, info).astype(int) - to_work_gray(rs[k], info).astype(int)).max()
    worst, worst_gray = max(worst, d), max(worst_gray, g)
    same += d == 0
    same_gray += g == 0
print(
    f"{n} frames: BGR identical {same}/{n} (max diff {worst}), work gray identical {same_gray}/{n} (max diff {worst_gray})"
)
