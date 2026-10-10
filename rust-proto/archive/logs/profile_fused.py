import functools
import json
import sys
import time

sys.path.insert(0, "/Users/zaratan/Projects/dvr-wt/rust/rust-proto")
from collections import defaultdict

from batdetect import cli, detect, noise, parallel

wall = defaultdict(float)
cpu = defaultdict(float)
calls = defaultdict(int)


def timed(name, fn):
    @functools.wraps(fn)
    def w(*a, **k):
        t0 = time.perf_counter()
        c0 = time.thread_time()
        try:
            return fn(*a, **k)
        finally:
            wall[name] += time.perf_counter() - t0
            cpu[name] += time.thread_time() - c0
            calls[name] += 1

    return w


for n in [
    "detect_video",
    "exclude",
    "track_detections",
    "write_tracks_csv",
    "median_background",
    "summary_image",
    "render_videos",
    "zoom_windows",
    "track_overlay",
]:
    setattr(cli, n, timed("stage." + n, getattr(cli, n)))
for n in ["target_blur", "find_blobs"]:
    setattr(detect, n, timed("detect." + n, getattr(detect, n)))
noise.temporal_median = timed("noise.temporal_median", noise.temporal_median)
parallel.to_work_gray = timed("reader.to_work_gray", parallel.to_work_gray)
orig_read = parallel.read_frames


def read_frames(cap):
    it = orig_read(cap)
    while True:
        t0 = time.perf_counter()
        c0 = time.thread_time()
        try:
            f = next(it)
        except StopIteration:
            return
        finally:
            wall["reader.decode"] += time.perf_counter() - t0
            cpu["reader.decode"] += time.thread_time() - c0
            calls["reader.decode"] += 1
        yield f


parallel.read_frames = read_frames
orig_probing = cli.probing
import contextlib


@contextlib.contextmanager
def probing(video):
    with orig_probing(video) as o:
        yield o
        globals()["_t_exit"] = time.perf_counter()
    wall["stage.probe_wait_after_detection"] += time.perf_counter() - globals()["_t_exit"]


cli.probing = probing
t0 = time.perf_counter()
rc = cli.main(sys.argv[1:])
total = time.perf_counter() - t0
out = {k: {"wall_s": round(wall[k], 2), "thread_cpu_s": round(cpu[k], 2), "calls": calls[k]} for k in sorted(wall)}
out["total_wall_s"] = round(total, 2)
print(json.dumps(out, indent=1), file=sys.stderr)
sys.exit(rc)
