from __future__ import annotations

import csv
import sys
from pathlib import Path

from batdetect.damage import Probe, damaged_spans, frame_entries, frames_after_pts_gaps, parse_probe
from batdetect.spans import Span, covered_frames

HALF_WINDOW = 15
INDICATORS = {
    "error_flags": lambda row: int(row["error_flags"]) != 0,
    "corrupt": lambda row: row["corrupt"] == "1",
    "logs_next": lambda row: int(row["logs_next"]) > 0,
    "logs_ctx": lambda row: int(row["logs_ctx"]) > 0,
}
NO_PTS = -(2**63)


def read_rows(path: Path) -> list[dict[str, str]]:
    with path.open(newline="") as table:
        return list(csv.DictReader(table, delimiter="\t"))


def probe_from_rows(rows: list[dict[str, str]], damaged: object) -> Probe:
    pts = [None if int(r["pts"]) == NO_PTS else int(r["pts"]) for r in rows]
    is_damaged = INDICATORS[str(damaged)]
    return Probe(
        frame_count=len(rows),
        error_frames=tuple(k for k, r in enumerate(rows) if is_damaged(r)),
        gap_frames=tuple(frames_after_pts_gaps(pts)),
        key_frames=tuple(k for k, r in enumerate(rows) if r["key"] == "1"),
    )


def describe(name: str, probe: Probe) -> list[Span]:
    spans = damaged_spans(probe, HALF_WINDOW)
    print(
        f"  {name:<12} frames {probe.frame_count}  errors {len(probe.error_frames)}  gaps {len(probe.gap_frames)}"
        f"  starts {len(probe.damaged_starts)}  spans {len(spans)}  excluded {len(covered_frames(spans))}"
    )
    return spans


def compare(reference: Probe, reference_spans: list[Span], candidate: Probe, spans: list[Span]) -> None:
    missing = sorted(set(reference.error_frames) - set(candidate.error_frames))
    extra = sorted(set(candidate.error_frames) - set(reference.error_frames))
    same_spans = spans == reference_spans
    print(
        f"    errors missing {len(missing)} {missing[:12]}  extra {len(extra)} {extra[:12]}"
        f"  gaps equal {candidate.gap_frames == reference.gap_frames}  keys equal {candidate.key_frames == reference.key_frames}"
        f"  spans equal {same_spans}"
    )
    if not same_spans:
        lost = covered_frames(reference_spans) - covered_frames(spans)
        added = covered_frames(spans) - covered_frames(reference_spans)
        print(f"    excluded frames lost {len(lost)}  added {len(added)}")


def main(probe_json: Path, tables: list[Path]) -> None:
    text = probe_json.read_text()
    reference = parse_probe(text)
    reference_pts = [p if isinstance(p := f.get("pts"), int) else None for f in frame_entries(text)]
    print(f"{probe_json.name}")
    reference_spans = describe("ffprobe", reference)
    for table in tables:
        rows = read_rows(table)
        pts = [None if int(r["pts"]) == NO_PTS else int(r["pts"]) for r in rows]
        print(f" {table.name}: pts equal {pts == reference_pts}")
        for indicator in INDICATORS:
            candidate = probe_from_rows(rows, indicator)
            compare(reference, reference_spans, candidate, describe(indicator, candidate))


if __name__ == "__main__":
    main(Path(sys.argv[1]), [Path(p) for p in sys.argv[2:]])
