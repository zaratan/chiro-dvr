import json, sys
ref = []
for line in open(sys.argv[1]):
    r = json.loads(line)
    for d in r["detections"]:
        ref.append((r["frame"], d["x"], d["y"], d["left"], d["top"], d["width"], d["height"], d["area"], d["amplitude"]))
got = []
for line in open(sys.argv[2]):
    p = line.split()
    if p[0].isdigit():
        got.append((int(p[0]), *map(float, p[1:])))
print("reference", len(ref), "candidate", len(got), "identical" if ref == got else "DIFFERENT")
if ref != got:
    diff = [i for i, (a, b) in enumerate(zip(ref, got)) if a != b]
    print("first differences", diff[:5], ref[diff[0]] if diff else None, got[diff[0]] if diff else None)
