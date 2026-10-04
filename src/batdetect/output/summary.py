from __future__ import annotations

from pathlib import Path

import cv2
import numpy as np

from batdetect.output.overlay import BOX_COLOR, TRAIL_COLOR
from batdetect.output.timefmt import format_time
from batdetect.track import Track
from batdetect.video import VideoError, VideoInfo

SUMMARY_FRAME = 30


def summary_image(video: Path, tracks: list[Track], info: VideoInfo, out_path: Path) -> None:
    cap = cv2.VideoCapture(str(video))
    cap.set(cv2.CAP_PROP_POS_FRAMES, min(SUMMARY_FRAME, max(0, info.frame_count - 1)))
    ok, raw = cap.read()
    cap.release()
    if not ok:
        raise VideoError(f"cannot read a background frame from {video}")
    image = (np.asarray(raw, dtype=np.float64) * 0.6).astype(np.uint8)
    for track in tracks:
        pts = np.array([(round(p.x), round(p.y)) for p in track.points], dtype=np.int32)
        cv2.polylines(image, [pts], isClosed=False, color=TRAIL_COLOR, thickness=3)
        start = (int(pts[0][0]) + 6, int(pts[0][1]))
        label = f"#{track.id} {format_time(track.first.frame / info.fps)}"
        cv2.putText(image, label, start, cv2.FONT_HERSHEY_SIMPLEX, 0.9, BOX_COLOR, 2)
    if not cv2.imwrite(str(out_path), image):
        raise VideoError(f"cannot write {out_path}")
