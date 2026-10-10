from __future__ import annotations

import pickle
import sys
from pathlib import Path

from analyze import ROOT, load_fast, nearest, tc

from batdetect.detect import DetectConfig
from batdetect.exclusion import exclude
from batdetect.stability import StabilityConfig
from batdetect.track import TrackConfig, track_detections
from batdetect.video import open_video

vid, video, first, last = sys.argv[1], Path(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
d = ROOT / vid
py = pickle.loads((d / "py.pkl").read_bytes())
fa, probe = load_fast(d / "fast.txt", d / "frames.txt")
cfg = DetectConfig()
cap, info = open_video(video, cfg.work_width)
cap.release()
fps = info.fps
sides = {}
for name, dets in (("py", py), ("fa", fa)):
    an = exclude(dets, probe, cfg.half_window(fps), fps, StabilityConfig())
    sides[name] = (dets, track_detections(an.detections, TrackConfig()))
for name, (_, tracks) in sides.items():
    for t in tracks:
        if t.first.frame <= last and t.last.frame >= first:
            print(
                f"{name} track {tc(t.first.frame, fps)} frames {t.first.frame}-{t.last.frame} n={len(t.points)}: "
                + " ".join(f"{p.frame}:({p.x:.0f},{p.y:.0f})" for p in t.points)
            )
for fr in range(first, last + 1):
    a, b = py.get(fr, []), fa.get(fr, [])
    fmt = lambda e, other: (
        f"({e.x:.1f},{e.y:.1f} a={e.area:.0f} amp={e.amplitude:.1f}{'' if nearest(e, other) else ' *'})"
    )
    if a or b:
        print(f"{fr} {tc(fr, fps)} | py " + " ".join(fmt(e, b) for e in a) + " | fa " + " ".join(fmt(e, a) for e in b))
