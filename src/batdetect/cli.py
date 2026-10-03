import argparse
import csv
import subprocess
from collections import deque
from dataclasses import dataclass, field
from pathlib import Path

import cv2
import numpy as np

WORK_W, WORK_H = 480, 360


@dataclass
class Track:
    id: int
    points: list = field(default_factory=list)

    @property
    def last(self):
        return self.points[-1]

    def predicted(self, frame):
        f, x, y = self.last[:3]
        if len(self.points) < 2:
            return x, y
        f0, x0, y0, *_ = self.points[-2]
        dt = max(1, f - f0)
        return x + (x - x0) / dt * (frame - f), y + (y - y0) / dt * (frame - f)

    def displacement(self):
        _, x0, y0, *_ = self.points[0]
        _, x1, y1, *_ = self.last
        return float(np.hypot(x1 - x0, y1 - y0))


def osd_mask(top, bottom):
    mask = np.ones((WORK_H, WORK_W), bool)
    mask[: int(WORK_H * top)] = False
    mask[int(WORK_H * (1 - bottom)) :] = False
    return mask


def detect(path, args):
    cap = cv2.VideoCapture(str(path))
    fps = cap.get(cv2.CAP_PROP_FPS) or 30
    mask = osd_mask(args.osd_top, args.osd_bottom)
    half = int(args.bg_window * fps / 2)
    buf, means = deque(maxlen=2 * half + 1), deque(maxlen=2 * half + 1)
    detections = {}
    index = 0

    def process(center):
        frames = np.stack(list(buf)[:: args.bg_step]).astype(np.int16)
        offsets = np.array(list(means)[:: args.bg_step])
        bg = np.median(frames, axis=0)
        cur = buf[center].astype(np.int16)
        resid = cur - bg - (means[center] - offsets.mean())
        resid[~mask] = 0
        binm = (np.abs(resid) > args.threshold).astype(np.uint8)
        k, _, stats, cents = cv2.connectedComponentsWithStats(binm)
        return [(*cents[i], *stats[i, :4]) for i in range(1, k) if args.min_area <= stats[i, 4] <= args.max_area]

    while True:
        ok, frame = cap.read()
        if not ok:
            break
        g = cv2.cvtColor(cv2.resize(frame, (WORK_W, WORK_H), interpolation=cv2.INTER_AREA), cv2.COLOR_BGR2GRAY)
        buf.append(g)
        means.append(float(g[mask].mean()))
        if len(buf) == buf.maxlen:
            detections[index - half] = process(half)
        index += 1
    cap.release()
    return detections, fps, index


def track(detections, args):
    active, done, next_id = [], [], 1
    for frame in sorted(detections):
        unmatched = list(detections[frame])
        for t in sorted(active, key=lambda t: -len(t.points)):
            if not unmatched:
                break
            px, py = t.predicted(frame)
            dists = [np.hypot(d[0] - px, d[1] - py) for d in unmatched]
            j = int(np.argmin(dists))
            if dists[j] <= args.max_jump:
                t.points.append((frame, *unmatched.pop(j)))
        for d in unmatched:
            active.append(Track(next_id, [(frame, *d)]))
            next_id += 1
        still = []
        for t in active:
            (done if frame - t.last[0] > args.max_gap else still).append(t)
        active = still
    done.extend(active)
    kept = [t for t in done if len(t.points) >= args.min_hits and t.displacement() >= args.min_travel]
    for n, t in enumerate(sorted(kept, key=lambda t: t.points[0][0]), 1):
        t.id = n
    return sorted(kept, key=lambda t: t.id)


def scale_box(p, sx, sy, pad):
    _, _, _, x, y, w, h = p
    return (int(x * sx) - pad, int(y * sy) - pad, int((x + w) * sx) + pad, int((y + h) * sy) + pad)


def render(path, tracks, fps, out_path, args):
    cap = cv2.VideoCapture(str(path))
    W, H = int(cap.get(cv2.CAP_PROP_FRAME_WIDTH)), int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))
    sx, sy = W / WORK_W, H / WORK_H
    by_frame = {}
    for t in tracks:
        for p in t.points:
            by_frame.setdefault(p[0], []).append((t, p))
    trail_len = int(args.trail * fps)
    ff = subprocess.Popen(
        [
            "ffmpeg",
            "-v",
            "error",
            "-y",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "bgr24",
            "-s",
            f"{W}x{H}",
            "-r",
            str(fps),
            "-i",
            "-",
            "-c:v",
            "libx264",
            "-crf",
            "20",
            "-pix_fmt",
            "yuv420p",
            str(out_path),
        ],
        stdin=subprocess.PIPE,
    )
    frame_no = 0
    while True:
        ok, frame = cap.read()
        if not ok:
            break
        for t, p in by_frame.get(frame_no, []):
            x0, y0, x1, y1 = scale_box(p, sx, sy, args.box_pad)
            cv2.rectangle(frame, (x0, y0), (x1, y1), (0, 0, 255), 2)
            cv2.putText(frame, f"#{t.id}", (x0, y0 - 6), cv2.FONT_HERSHEY_SIMPLEX, 0.8, (0, 0, 255), 2)
            trail = [q for q in t.points if frame_no - trail_len <= q[0] <= frame_no]
            pts = np.array([(int(q[1] * sx), int(q[2] * sy)) for q in trail], np.int32)
            if len(pts) > 1:
                cv2.polylines(frame, [pts], False, (0, 200, 255), 2)
        ff.stdin.write(frame.tobytes())
        frame_no += 1
    ff.stdin.close()
    ff.wait()
    cap.release()


def summary_image(path, tracks, fps, out_path):
    cap = cv2.VideoCapture(str(path))
    cap.set(cv2.CAP_PROP_POS_FRAMES, 30)
    ok, frame = cap.read()
    cap.release()
    H, W = frame.shape[:2]
    sx, sy = W / WORK_W, H / WORK_H
    img = (frame * 0.6).astype(np.uint8)
    for t in tracks:
        pts = np.array([(int(p[1] * sx), int(p[2] * sy)) for p in t.points], np.int32)
        cv2.polylines(img, [pts], False, (0, 200, 255), 3)
        x, y = pts[0]
        cv2.putText(
            img, f"#{t.id} {t.points[0][0] / fps:.1f}s", (x + 6, y), cv2.FONT_HERSHEY_SIMPLEX, 0.9, (0, 0, 255), 2
        )
    cv2.imwrite(str(out_path), img)


def fmt(sec):
    return f"{int(sec // 60)}:{sec % 60:05.2f}"


def write_csv(tracks, fps, out_path):
    with open(out_path, "w", newline="") as fh:
        w = csv.writer(fh)
        w.writerow(["id", "start", "end", "start_s", "duration_s", "hits", "travel_px", "speed_px_s"])
        for t in tracks:
            s, e = t.points[0][0] / fps, t.last[0] / fps
            dur = max(e - s, 1 / fps)
            w.writerow(
                [
                    t.id,
                    fmt(s),
                    fmt(e),
                    round(s, 2),
                    round(dur, 2),
                    len(t.points),
                    round(t.displacement(), 1),
                    round(t.displacement() / dur, 1),
                ]
            )


VIDEO_EXTENSIONS = {".mp4", ".mov", ".avi", ".mkv", ".m4v"}


def collect_videos(inputs):
    videos = []
    for p in inputs:
        if p.is_dir():
            videos.extend(sorted(f for f in p.iterdir() if f.suffix.lower() in VIDEO_EXTENSIONS))
        else:
            videos.append(p)
    return videos


def split_clips(annotated, tracks, fps, total_frames, split_dir, margin):
    split_dir.mkdir(exist_ok=True)
    duration = total_frames / fps
    for t in tracks:
        start = max(0.0, t.points[0][0] / fps - margin)
        end = min(duration, t.last[0] / fps + margin)
        name = f"{t.id:02d}_{fmt(t.points[0][0] / fps).replace(':', 'm').replace('.', 's')}.mp4"
        subprocess.run(
            [
                "ffmpeg",
                "-v",
                "error",
                "-y",
                "-ss",
                f"{start:.3f}",
                "-i",
                str(annotated),
                "-t",
                f"{end - start:.3f}",
                "-c:v",
                "libx264",
                "-crf",
                "20",
                "-pix_fmt",
                "yuv420p",
                str(split_dir / name),
            ],
            check=True,
        )


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("inputs", nargs="+", type=Path, help="video files or folders")
    ap.add_argument("-o", "--out-dir", type=Path, default=Path("out"))
    ap.add_argument("--threshold", type=float, default=25)
    ap.add_argument("--min-area", type=int, default=2)
    ap.add_argument("--max-area", type=int, default=300)
    ap.add_argument("--bg-window", type=float, default=1.0, help="seconds of rolling median background")
    ap.add_argument("--bg-step", type=int, default=3)
    ap.add_argument("--osd-top", type=float, default=0.07)
    ap.add_argument("--osd-bottom", type=float, default=0.10)
    ap.add_argument("--max-jump", type=float, default=40)
    ap.add_argument("--max-gap", type=int, default=6)
    ap.add_argument("--min-hits", type=int, default=5)
    ap.add_argument("--min-travel", type=float, default=15)
    ap.add_argument("--box-pad", type=int, default=12)
    ap.add_argument("--trail", type=float, default=1.0)
    ap.add_argument("--clip-margin", type=float, default=1.5, help="seconds kept before and after each track")
    args = ap.parse_args()

    for video in collect_videos(args.inputs):
        dest = args.out_dir / video.stem
        dest.mkdir(parents=True, exist_ok=True)
        detections, fps, n = detect(video, args)
        tracks = track(detections, args)
        write_csv(tracks, fps, dest / f"{video.stem}.tracks.csv")
        summary_image(video, tracks, fps, dest / f"{video.stem}.tracks.png")
        annotated = dest / f"{video.stem}_boxes.mp4"
        render(video, tracks, fps, annotated, args)
        split_clips(annotated, tracks, fps, n, dest / "split", args.clip_margin)
        print(f"{video.name}: {n} frames, {len(tracks)} tracks -> {dest}")
        for t in tracks:
            s, e = t.points[0][0] / fps, t.last[0] / fps
            print(f"  #{t.id:<3} {fmt(s)} -> {fmt(e)}  hits={len(t.points):<4} travel={t.displacement():.0f}px")


if __name__ == "__main__":
    main()
