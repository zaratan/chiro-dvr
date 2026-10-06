from __future__ import annotations

import hashlib
import json
import pickle
from dataclasses import asdict
from pathlib import Path

from batdetect.bench.collect import BenchRun, collect
from batdetect.bench.config import BenchSetup
from batdetect.detect import DetectConfig
from batdetect.probe import ffprobe_version
from batdetect.stability import StabilityConfig
from batdetect.synthetic.sampling import Sampling

CACHED_SOURCES = (
    "video.py",
    "median.py",
    "detect.py",
    "track.py",
    "stability.py",
    "spans.py",
    "damage.py",
    "probe.py",
    "exclusion.py",
    "parallel.py",
    "synthetic",
)
PACKAGE_ROOT = Path(__file__).resolve().parent.parent


def code_fingerprint() -> list[str]:
    files = sorted(
        f
        for name in CACHED_SOURCES
        for f in ([PACKAGE_ROOT / name] if name.endswith(".py") else (PACKAGE_ROOT / name).rglob("*.py"))
    )
    return [hashlib.sha256(f.read_bytes()).hexdigest() for f in files]


def cache_key(video: Path, detect: DetectConfig, sampling: Sampling, stability: StabilityConfig) -> str:
    stat = video.stat()
    payload = {
        "code": code_fingerprint(),
        "ffprobe": ffprobe_version(),
        "video": str(video.resolve()),
        "size": stat.st_size,
        "mtime": stat.st_mtime,
        "detect": asdict(detect),
        "sampling": asdict(sampling),
        "stability": asdict(stability),
    }
    normalized = json.loads(json.dumps(payload), parse_int=float)
    return hashlib.sha256(json.dumps(normalized, sort_keys=True).encode()).hexdigest()


def load_or_collect(setup: BenchSetup, cache: Path) -> BenchRun:
    key = cache_key(setup.video, setup.detect, setup.sampling, setup.stability)
    if cache.exists():
        try:
            with cache.open("rb") as fh:
                cached: object = pickle.load(fh)
        except pickle.UnpicklingError, AttributeError, TypeError, EOFError, ImportError:
            cached = None
        if isinstance(cached, BenchRun) and cached.key == key:
            return cached
    run = collect(setup)
    run.key = key
    cache.parent.mkdir(parents=True, exist_ok=True)
    tmp = cache.with_suffix(".tmp")
    with tmp.open("wb") as fh:
        pickle.dump(run, fh)
    tmp.replace(cache)
    return run
