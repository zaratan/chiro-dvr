import sys
from collections import defaultdict


def load(p):
    d = defaultdict(list)
    for line in open(p):
        f = line.split()
        d[int(f[0])].append(line)
    return d


a, b = load(sys.argv[1]), load(sys.argv[2])
print(sorted(k for k in set(a) | set(b) if a.get(k) != b.get(k)))
