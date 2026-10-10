import sys
from pathlib import Path

from batdetect.damage import Probe, damaged_spans, frames_after_pts_gaps, parse_probe

probe_json, frames_txt = Path(sys.argv[1]), Path(sys.argv[2])
ref = parse_probe(probe_json.read_text())
fr = [l.split() for l in frames_txt.read_text().splitlines()]
pts = [None if f[1] == "-" else int(f[1]) for f in fr]
mine = Probe(
    len(fr),
    tuple(int(f[0]) for f in fr if int(f[4]) != 0),
    tuple(frames_after_pts_gaps(pts)),
    tuple(int(f[0]) for f in fr if f[2] == "1"),
)
same = lambda a, b: "=" if a == b else "DIFF"
print(
    f"{sys.argv[3]} | {ref.frame_count}/{mine.frame_count} {same(ref.frame_count, mine.frame_count)} | errors {len(ref.error_frames)}/{len(mine.error_frames)} {same(ref.error_frames, mine.error_frames)} | gaps {len(ref.gap_frames)}/{len(mine.gap_frames)} {same(ref.gap_frames, mine.gap_frames)} | keys {same(ref.key_frames, mine.key_frames)} | spans {same(damaged_spans(ref, 15), damaged_spans(mine, 15))}"
)
