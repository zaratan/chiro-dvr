import sys

sys.path.insert(0, "/Users/zaratan/Projects/dvr-wt/rust/rust-proto/target/pylib")
import batdetect_rs as rs
import cv2
import numpy as np

rng = np.random.default_rng(7)


def same(a, b):
    return a.shape == b.shape and a.tobytes() == b.tobytes()


print("threads", cv2.getNumThreads())
for sw, sh, dw, dh in [
    (1440, 1080, 960, 720),
    (1440, 1080, 480, 360),
    (1920, 1080, 960, 540),
    (1280, 720, 960, 540),
    (1437, 1077, 958, 718),
    (1440, 1080, 720, 540),
]:
    img = rng.integers(0, 256, (sh, sw, 3), dtype=np.uint8)
    ref = cv2.resize(img, (dw, dh), interpolation=cv2.INTER_AREA)
    for f in (False, True):
        m = rs.resize_area_bgr(img, dw, dh, f)
        print("resize", sw, dw, "fma", f, same(m, ref), int(np.abs(m.astype(int) - ref).max()), int((m != ref).sum()))
for w, h in [(960, 720), (961, 541), (962, 540), (963, 17), (853, 480), (7, 5)]:
    g = rng.integers(0, 256, (h, w), dtype=np.uint8)
    for sigma in (1.0, 0.75, 0.4, 1.7):
        k = round(sigma * 8 + 1) | 1
        ker = [float(v) for v in cv2.getGaussianKernel(k, sigma, cv2.CV_32F).ravel()]
        ref = cv2.GaussianBlur(g.astype(np.float32), (0, 0), sigma)
        m = rs.gaussian_blur_f32(g, ker)
        bad = m != ref
        cols = sorted(set(np.nonzero(bad)[1].tolist()))
        print("blur", w, h, "sigma", sigma, "k", k, same(m, ref), int(bad.sum()), cols[:6], cols[-3:] if cols else "")
    print(
        "gray",
        w,
        h,
        same(
            rs.bgr_to_gray(rng.integers(0, 256, (h, w, 3), dtype=np.uint8)),
            cv2.cvtColor(_ := rng.integers(0, 256, (h, w, 3), dtype=np.uint8), cv2.COLOR_BGR2GRAY),
        )
        or same(rs.bgr_to_gray(_), cv2.cvtColor(_, cv2.COLOR_BGR2GRAY)),
    )
for w, h, p in [(960, 720, 0.3), (960, 720, 0.55), (961, 721, 0.45), (3000, 2000, 0.5), (5, 3, 0.6)]:
    im = (rng.random((h, w)) < p).astype(np.uint8)
    c, l, s, ce = cv2.connectedComponentsWithStats(im)
    mc, ml, ms, mce = rs.components(im)
    print(
        "cc",
        w,
        h,
        p,
        c,
        c == mc
        and same(np.asarray(l, np.int32), ml)
        and np.array_equal(s, np.array(ms))
        and np.array_equal(ce, np.array(mce)),
    )
    for size in (3, 9, 15):
        kern = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (size, size))
        print("  close", size, same(rs.morph_close(im, size), cv2.morphologyEx(im, cv2.MORPH_CLOSE, kern)))
