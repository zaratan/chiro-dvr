from __future__ import annotations

from collections import Counter
from dataclasses import dataclass
from pathlib import Path

VIDEO_EXTENSIONS = frozenset({".mp4", ".mov", ".avi", ".mkv", ".m4v"})


@dataclass(frozen=True, slots=True)
class Job:
    video: Path
    dest: Path


def collect_videos(inputs: list[Path]) -> list[Path]:
    videos: list[Path] = []
    for path in inputs:
        if path.is_dir():
            videos.extend(sorted(f for f in path.iterdir() if f.is_file() and f.suffix.lower() in VIDEO_EXTENSIONS))
        else:
            videos.append(path)
    return videos


def output_name(video: Path, ambiguous: bool) -> str:
    if not ambiguous:
        return video.stem
    return f"{video.parent.name}_{video.stem}_{video.suffix.lstrip('.').lower()}"


def plan_jobs(videos: list[Path], out_dir: Path) -> list[Job]:
    stems = Counter(v.stem for v in videos)
    jobs = [Job(v, out_dir / output_name(v, stems[v.stem] > 1)) for v in videos]
    clashes = [dest for dest, n in Counter(j.dest for j in jobs).items() if n > 1]
    if clashes:
        raise ValueError(f"several videos would write to {clashes[0]}")
    return jobs
