import numpy as np


def pw(a, t):
    n = len(a)
    if n < 8:
        r = t(-0.0)
        for v in a:
            r = t(r + v)
        return r
    if n <= 128:
        r = [t(x) for x in a[:8]]
        i = 8
        while i < n - (n % 8):
            for j in range(8):
                r[j] = t(r[j] + a[i + j])
            i += 8
        res = t(t(t(r[0] + r[1]) + t(r[2] + r[3])) + t(t(r[4] + r[5]) + t(r[6] + r[7])))
        while i < n:
            res = t(res + a[i])
            i += 1
        return res
    n2 = n // 2
    n2 -= n2 % 8
    return t(pw(a[:n2], t) + pw(a[n2:], t))


rng = np.random.default_rng(1)
bad = 0
for n in [5, 8, 11, 17, 127, 128, 129, 1000, 8191, 8192, 8193, 20000, 100003, 518400]:
    a = (rng.random(n) * 255).astype(np.float32)
    single = np.float32(pw(list(a), np.float32))
    mine = np.float32(single / np.float32(n))
    ref = a.mean()
    chunk = np.float32(-0.0)
    for s in range(0, n, 8192):
        chunk = np.float32(chunk + pw(list(a[s : s + 8192]), np.float32))
    print("f32", n, ref == mine, ref == np.float32(chunk / np.float32(n)), type(ref))
    b = (rng.random(n) * 255).astype(np.uint8)
    refu = b.mean()
    bf = b.astype(np.float64)
    one = pw(list(bf), np.float64) / n
    ch = -0.0
    for s in range(0, n, 8192):
        ch = ch + pw(list(bf[s : s + 8192]), np.float64)
    print("u8 ", n, refu == one, refu == ch / n)
    l = list(rng.random(n % 40 + 1) * 200)
    print("list", len(l), float(np.mean(l)) == pw(l, np.float64) / len(l))
