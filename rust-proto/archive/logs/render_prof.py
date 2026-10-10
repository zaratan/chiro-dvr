import functools
import sys
import time

sys.path.insert(0, "/Users/zaratan/Projects/dvr-wt/rust/rust-proto")
from collections import defaultdict

import fast_run

from batdetect.output import render, writer

T = defaultdict(float)
N = defaultdict(int)


def timed(name, fn):
    @functools.wraps(fn)
    def w(*a, **k):
        t = time.perf_counter()
        try:
            return fn(*a, **k)
        finally:
            T[name] += time.perf_counter() - t
            N[name] += 1

    return w


render.write_frame = timed("write_frame", render.write_frame)
writer.FrameWriter.__init__ = timed("writer_open", writer.FrameWriter.__init__)
writer.FrameWriter.close = timed("writer_close", writer.FrameWriter.close)
writer.FrameWriter.write = timed("writer_write_thread", writer.FrameWriter.write)
fast_run.ThreadedWriter.write = timed("enqueue", fast_run.ThreadedWriter.write)
orig = fast_run.served_frames


def served(*a, **k):
    it = orig(*a, **k)
    while True:
        t = time.perf_counter()
        try:
            x = next(it)
        except StopIteration:
            return
        finally:
            T["read"] += time.perf_counter() - t
        N["read"] += 1
        yield x


fast_run.served_frames = served
rc = fast_run.cli.main()
print({k: (round(v, 2), N[k]) for k, v in T.items()}, fast_run.TIMINGS, file=sys.stderr)
