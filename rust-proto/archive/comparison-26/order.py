import pickle
import sys

from analyze import ROOT, load_fast, nearest

tot_frames = tot_diff = 0
for vid in sys.argv[1:]:
    d = ROOT / vid
    py = pickle.loads((d / "py.pkl").read_bytes())
    fa, probe = load_fast(d / "fast.txt", d / "frames.txt")
    frames = diff = 0
    for fr, a in py.items():
        b = fa.get(fr, [])
        if len(a) < 2 or len(a) != len(b):
            continue
        idx = []
        for e in a:
            m = nearest(e, b)
            if m is None:
                break
            idx.append(b.index(m))
        else:
            frames += 1
            if idx != sorted(idx):
                diff += 1
    tot_frames += frames
    tot_diff += diff
    print(vid, f"images à >=2 détections toutes appariées: {frames}, ordre différent: {diff}")
print("total", tot_frames, tot_diff)
