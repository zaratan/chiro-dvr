import sys
import time

sys.path.insert(0, "rust-proto/target/pylib")
import batdetect_rs
import cv2
import numpy as np

from batdetect.median import temporal_median

rng = np.random.default_rng(1)
frames = [
    cv2.GaussianBlur(rng.integers(0, 255, (720, 960), dtype=np.uint8).astype(np.float32), (0, 0), 1.5)
    for _ in range(11)
]


def bench(f, n=40):
    f()
    t = time.perf_counter()
    for _ in range(n):
        f()
    return (time.perf_counter() - t) / n * 1e3


print("numpy network   %.2f ms" % bench(lambda: temporal_median(frames)))
print("np.median       %.2f ms" % bench(lambda: np.median(np.stack(frames), axis=0), 5))
print("rust 1 core     %.2f ms" % bench(lambda: batdetect_rs.temporal_median_f32(frames, False)))
print("rust rayon      %.2f ms" % bench(lambda: batdetect_rs.temporal_median_f32(frames, True)))
print("blur            %.2f ms" % bench(lambda: cv2.GaussianBlur(frames[0], (0, 0), 1.5)))
a = temporal_median(frames)
b = batdetect_rs.temporal_median_f32(frames, False)
print("identical", a.dtype == b.dtype and a.tobytes() == b.tobytes())
from concurrent.futures import ThreadPoolExecutor

for workers, bands in [(4, 4), (8, 8), (8, 32), (8, 90)]:
    pool = ThreadPoolExecutor(workers)
    h = frames[0].shape[0]
    edges = [round(k * h / bands) for k in range(bands + 1)]

    def banded():
        out = np.empty(frames[0].shape, np.float64)

        def one(k):
            out[edges[k] : edges[k + 1]] = temporal_median([f[edges[k] : edges[k + 1]] for f in frames])

        list(pool.map(one, range(bands)))
        return out

    print(
        f"numpy threads {workers} bands {bands}: %.2f ms" % bench(banded),
        "identical",
        banded().tobytes() == a.tobytes(),
    )
