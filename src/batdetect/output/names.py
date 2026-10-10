from __future__ import annotations

import re
import shutil
import unicodedata
from dataclasses import dataclass
from pathlib import Path

from batdetect.language import is_french
from batdetect.track import Track

PARAMS = "params.json"
ZOOM_SUFFIX = "_zoom"
VIDEO_SUFFIX = ".mp4"
MIN_TRACK_DIGITS = 2
PERIOD_PATTERN = r"(_\d{3}m-\d{3}m)"


@dataclass(frozen=True, slots=True)
class Names:
    clips_dir: str
    track: str
    tracks_csv: str
    summary: str
    annotated: str


FRENCH = Names(clips_dir="extraits", track="piste", tracks_csv="_pistes", summary="_resume", annotated="_annotee")
ENGLISH = Names(clips_dir="clips", track="track", tracks_csv="_tracks", summary="_summary", annotated="_annotated")
LEGACY = Names(clips_dir="split", track="", tracks_csv=".tracks", summary=".tracks", annotated="_boxes")


def current() -> Names:
    return FRENCH if is_french() else ENGLISH


def clips_dir(dest: Path) -> Path:
    return dest / current().clips_dir


def clip_name(track: Track, fps: float, track_count: int) -> str:
    minutes, seconds = divmod(int(track.first.frame / fps), 60)
    number = str(track.id).zfill(max(MIN_TRACK_DIGITS, len(str(track_count))))
    return f"{current().track}_{number}_{minutes}m{seconds:02d}s{VIDEO_SUFFIX}"


def zoom_path(clip: Path) -> Path:
    return clip.with_stem(clip.stem + ZOOM_SUFFIX)


def tracks_csv(dest: Path, stem: str) -> Path:
    return dest / f"{stem}{current().tracks_csv}.csv"


def summary_png(dest: Path, stem: str, period_suffix: str = "") -> Path:
    return dest / f"{stem}{current().summary}{period_suffix}.png"


def annotated_video(dest: Path, stem: str) -> Path:
    return dest / f"{stem}{current().annotated}{VIDEO_SUFFIX}"


def params_json(dest: Path) -> Path:
    return dest / PARAMS


def _nfc(text: str) -> str:
    return unicodedata.normalize("NFC", text)


def previous_outputs(dest: Path, stem: str) -> list[Path]:
    if not dest.is_dir():
        return []
    every = (FRENCH, ENGLISH, LEGACY)
    stem = _nfc(stem)
    exact = {
        name
        for names in every
        for name in (names.clips_dir, f"{stem}{names.tracks_csv}.csv", f"{stem}{names.annotated}{VIDEO_SUFFIX}")
    }
    summaries = "|".join(re.escape(names.summary) for names in every)
    summary = re.compile(rf"{re.escape(stem)}({summaries}){PERIOD_PATTERN}?\.png")
    return sorted(p for p in dest.iterdir() if _nfc(p.name) in exact or summary.fullmatch(_nfc(p.name)))


def remove_previous_outputs(dest: Path, stem: str) -> None:
    for path in previous_outputs(dest, stem):
        if path.is_dir():
            shutil.rmtree(path)
        else:
            path.unlink()
