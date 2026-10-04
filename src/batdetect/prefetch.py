from __future__ import annotations

import queue
import threading
from collections.abc import Generator, Iterable, Iterator
from contextlib import contextmanager
from dataclasses import dataclass

PUT_TIMEOUT_S = 0.05


@dataclass(frozen=True, slots=True)
class Failure:
    error: BaseException


class End:
    pass


def join_fully(thread: threading.Thread) -> None:
    try:
        thread.join()
    except BaseException:
        thread.join()
        raise


@contextmanager
def prefetched[T](source: Iterable[T], depth: int) -> Generator[Iterator[T]]:
    if depth < 1:
        raise ValueError("depth must be >= 1")
    items: queue.Queue[T | Failure | End] = queue.Queue(maxsize=depth)
    stop = threading.Event()

    def offer(item: T | Failure | End) -> bool:
        while not stop.is_set():
            try:
                items.put(item, timeout=PUT_TIMEOUT_S)
            except queue.Full:
                continue
            return True
        return False

    def produce() -> None:
        try:
            for item in source:
                if not offer(item):
                    return
        except BaseException as error:
            offer(Failure(error))
            return
        offer(End())

    def consume() -> Iterator[T]:
        while True:
            item = items.get()
            if isinstance(item, End):
                return
            if isinstance(item, Failure):
                raise item.error
            yield item

    thread = threading.Thread(target=produce, daemon=True)
    thread.start()
    try:
        yield consume()
    finally:
        stop.set()
        join_fully(thread)
