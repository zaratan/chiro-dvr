import pickle
import sys
from pathlib import Path

from analyze import ROOT, load_fast, tc

from batdetect.detect import DetectConfig
from batdetect.exclusion import exclude
from batdetect.stability import StabilityConfig
from batdetect.track import TrackConfig, track_detections
from batdetect.video import open_video

vid, first, last = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
d = ROOT / vid
py = pickle.loads((d / "py.pkl").read_bytes())
fa, probe = load_fast(d / "fast.txt", d / "frames.txt")
cap, info = open_video(
    Path("/Users/zaratan/dossier sans titre/img_0000") / f"video_{vid}.mp4", DetectConfig().work_width
)
cap.release()
fps = info.fps


def sig(dets):
    an = exclude(dets, probe, DetectConfig().half_window(fps), fps, StabilityConfig())
    return sorted(
        (t.first.frame, len(t.points))
        for t in track_detections(an.detections, TrackConfig())
        if t.last.frame >= first and t.first.frame <= last
    )


key = lambda e: (round(e.x, 3), round(e.y, 3), round(e.area, 3), round(e.amplitude, 3))
diff = [f for f in range(first, last + 1) if sorted(map(key, py.get(f, []))) != sorted(map(key, fa.get(f, [])))]
base, target = sig(py), sig(fa)
print("py", base)
print("fast", target)
print("images qui diffèrent:", diff)
for f in diff:
    s = sig({**py, f: fa[f]})
    if s != base:
        print(f"fast seulement à l'image {f} ({tc(f, fps)}) -> {s}", "= fast" if s == target else "")
