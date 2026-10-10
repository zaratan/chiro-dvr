import math
import pickle
import sys
from collections import defaultdict

py = pickle.load(open(sys.argv[1], "rb"))
fa = defaultdict(list)
for line in open(sys.argv[2]):
    f = line.split()
    fa[int(f[0])].append(tuple(map(float, f[1:])))
n_py = sum(len(v) for v in py.values())
n_fa = sum(len(v) for v in fa.values())
matched = 0
area_err = []
amp_err = []
for fr, v in py.items():
    for d in v:
        best = min(fa.get(fr, []), key=lambda e: math.hypot(d.x - e[0], d.y - e[1]), default=None)
        if best and math.hypot(d.x - best[0], d.y - best[1]) < 3:
            matched += 1
            area_err.append(abs(best[6] - d.area) / d.area)
            amp_err.append(abs(best[7] - d.amplitude) / d.amplitude)
amp_err.sort()
area_err.sort()
q = lambda a, p: a[int(p * (len(a) - 1))] if a else float("nan")
print(
    f"python {n_py}, fast {n_fa}, python matched within 3 px: {matched} ({matched / max(1, n_py):.1%}); amplitude error median {q(amp_err, 0.5):.2%} p95 {q(amp_err, 0.95):.2%}; area error median {q(area_err, 0.5):.2%} p95 {q(area_err, 0.95):.2%}"
)
