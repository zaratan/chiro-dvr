"""Compare the detections printed by the Rust binary with a pickle written by compare.py, float for float."""

import pickle
import sys
from pathlib import Path

reference = pickle.loads(Path(sys.argv[1]).read_bytes())
expected = [
    (frame, d.x, d.y, d.left, d.top, d.width, d.height, d.area, d.amplitude)
    for frame, found in reference.items()
    for d in found
]
lines = Path(sys.argv[2]).read_text().splitlines()
actual = [(int(f[0]), *map(float, f[1:])) for f in (line.split() for line in lines)]
print(f"python {len(expected)} detections, rust {len(actual)}: {'identical' if expected == actual else 'DIFFERENT'}")
if expected != actual:
    for k, (a, b) in enumerate(zip(expected, actual)):
        if a != b:
            print("first difference at", k, a, b)
            break
