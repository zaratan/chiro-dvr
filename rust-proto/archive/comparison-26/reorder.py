import pickle
import sys
from pathlib import Path

from analyze import POINTS_TOL, ROOT, load_fast, nearest, pair_tracks, tc

from batdetect.detect import DetectConfig
from batdetect.exclusion import exclude
from batdetect.stability import StabilityConfig
from batdetect.track import TrackConfig, track_detections
from batdetect.video import open_video

VIDEOS = Path("/Users/zaratan/dossier sans titre/img_0000")
for vid in sys.argv[1:]:
    d = ROOT / vid
    py = pickle.loads((d / "py.pkl").read_bytes())
    fa, probe = load_fast(d / "fast.txt", d / "frames.txt")
    cap, info = open_video(VIDEOS / f"video_{vid}.mp4", DetectConfig().work_width)
    cap.release()
    fps = info.fps
    reordered = {}
    for fr, b in fa.items():
        a = py.get(fr, [])
        ms = [nearest(e, b) for e in a]
        if len(a) == len(b) and all(m is not None for m in ms) and len({id(m) for m in ms}) == len(b):
            reordered[fr] = ms
        else:
            reordered[fr] = b

    def tracks(dets):
        an = exclude(dets, probe, DetectConfig().half_window(fps), fps, StabilityConfig())
        return track_detections(an.detections, TrackConfig())

    tp, tr = tracks(py), tracks(reordered)
    pairs = pair_tracks(tp, tr)
    ok = [(i, j) for i, j in pairs if abs(len(tp[i].points) - len(tr[j].points)) <= POINTS_TOL]
    lone_p = [
        tc(tp[i].first.frame, fps) + f"({len(tp[i].points)})" for i in range(len(tp)) if i not in {p[0] for p in pairs}
    ]
    lone_r = [
        tc(tr[j].first.frame, fps) + f"({len(tr[j].points)})" for j in range(len(tr)) if j not in {p[1] for p in pairs}
    ]
    off = [
        f"{tc(tp[i].first.frame, fps)} {len(tp[i].points)}/{len(tr[j].points)}" for i, j in pairs if (i, j) not in ok
    ]
    print(
        vid,
        f"py {len(tp)} fast réordonné {len(tr)} dans tol. {len(ok)}; py seules {lone_p}; fast seules {lone_r}; hors tol. {off}",
    )
