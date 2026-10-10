"""Compare each detection stage of the Rust prototype with OpenCV and numpy on frames of a real video."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent / "target" / "pylib"))

import batdetect_rs as rs
import cv2
import numpy as np

from batdetect.detect import DetectConfig, find_blobs, target_blur
from batdetect.video import open_video, read_frames, to_work_gray


def same(a, b):
    return a.dtype == b.dtype and a.shape == b.shape and a.tobytes() == b.tobytes()


def main(video: Path, limit: int) -> None:
    cfg = DetectConfig()
    cap, info = open_video(video, cfg.work_width)
    sigma = cfg.target_sigma / info.scale
    ksize = round(sigma * 4 * 2 + 1) | 1
    kernel = [float(v) for v in np.asarray(cv2.getGaussianKernel(ksize, sigma, cv2.CV_32F), dtype=np.float32).ravel()]
    tally: dict[str, int] = {}
    worst: dict[str, float] = {}

    def score(name: str, ok: bool, diff: float = 0.0) -> None:
        tally[name] = tally.get(name, 0) + int(ok)
        worst[name] = max(worst.get(name, 0.0), diff)

    previous = []
    n = 0
    for n, frame in enumerate(read_frames(cap), 1):
        small = np.asarray(cv2.resize(frame, (info.work_width, info.work_height), interpolation=cv2.INTER_AREA))
        for fused in (False, True):
            mine = rs.resize_area_bgr(frame, info.work_width, info.work_height, fused)
            score(f"resize INTER_AREA fma={fused}", same(mine, small), float(np.abs(mine.astype(int) - small).max()))
        gray = to_work_gray(frame, info)
        mine = rs.bgr_to_gray(small)
        score("cvtColor BGR2GRAY", same(mine, gray), float(np.abs(mine.astype(int) - gray).max()))
        blurred = target_blur(gray, sigma)
        mine = rs.gaussian_blur_f32(gray, kernel)
        score("GaussianBlur f32", same(mine, blurred), float(np.abs(mine - blurred).max()))
        score("mean f32 (numpy)", rs.mean_f32(blurred) == float(blurred.mean()))
        previous.append(blurred)
        if len(previous) == 3:
            residual = previous[2].astype(np.float64) - previous[0]
            residual[::7, ::5] += 30.0
            above = (np.abs(residual) > cfg.threshold).astype(np.uint8)
            size = 2 * round(cfg.merge_radius / info.scale) + 1
            kern = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (size, size))
            closed = np.asarray(cv2.morphologyEx(above, cv2.MORPH_CLOSE, kern), dtype=np.uint8)
            score("morphologyEx CLOSE ellipse", same(rs.morph_close(above, size), closed))
            count, labels, stats, cents = cv2.connectedComponentsWithStats(closed)
            mcount, mlabels, mstats, mcents = rs.components(closed)
            score(
                "connectedComponentsWithStats",
                count == mcount
                and same(np.asarray(labels, np.int32), mlabels)
                and np.array_equal(np.asarray(stats, np.int64), np.array(mstats, np.int64))
                and np.array_equal(np.asarray(cents, np.float64), np.array(mcents, np.float64)),
            )
            threshold = np.full(residual.shape, cfg.threshold, dtype=np.float64)
            threshold[::11, ::3] = 40.0
            ref = [
                (d.x, d.y, d.left, d.top, d.width, d.height, d.area, d.amplitude)
                for d in find_blobs(residual, 0, cfg, info.scale, threshold)
            ]
            mine = rs.find_blobs(residual, threshold, cfg.min_area, cfg.max_area, cfg.merge_radius, info.scale)
            score("find_blobs (detections)", ref == mine)
            previous.pop(0)
        if n >= limit:
            break
    cap.release()
    for name in tally:
        total = (
            n
            if name in {"cvtColor BGR2GRAY", "GaussianBlur f32", "mean f32 (numpy)"} or name.startswith("resize")
            else n - 2
        )
        print(f"{name:34} identical {tally[name]}/{total}  max diff {worst[name]:g}")


if __name__ == "__main__":
    main(Path(sys.argv[1]), int(sys.argv[2]))
