from __future__ import annotations

import itertools
import threading
import time
from collections.abc import Callable, Iterator

import pytest

from batdetect.prefetch import prefetched
from helpers import GUARD_TIMEOUT_S, finishes

DEPTH = 3


class ReaderTestError(Exception):
    pass


def wait_until(condition: Callable[[], bool]) -> None:
    deadline = time.monotonic() + GUARD_TIMEOUT_S
    while not condition() and time.monotonic() < deadline:
        time.sleep(0.001)


def counted(pulled: list[int]) -> Iterator[int]:
    for i in itertools.count():
        pulled.append(i)
        yield i


def test_items_arrive_in_order_as_the_very_same_objects() -> None:
    source = [object() for _ in range(50)]
    received: list[object] = []

    def read_all() -> None:
        with prefetched(source, depth=4) as items:
            received.extend(items)

    assert finishes(read_all)
    assert len(received) == len(source)
    assert all(a is b for a, b in zip(received, source, strict=True))


def test_empty_source_gives_nothing() -> None:
    nothing: list[int] = []
    received: list[int] = []

    def read_all() -> None:
        with prefetched(nothing, depth=4) as items:
            received.extend(items)

    assert finishes(read_all)
    assert received == []


def test_producer_error_comes_after_the_items_already_read_with_its_own_type() -> None:
    def failing() -> Iterator[int]:
        yield 1
        yield 2
        raise ReaderTestError

    received: list[int] = []
    raised: list[BaseException] = []

    def read_all() -> None:
        try:
            with prefetched(failing(), depth=4) as items:
                received.extend(items)
        except ReaderTestError as error:
            raised.append(error)

    assert finishes(read_all)
    assert received == [1, 2]
    assert len(raised) == 1


def test_stopping_early_ends_a_reader_blocked_on_a_full_queue() -> None:
    before = threading.active_count()
    pulled: list[int] = []

    def read_one_then_leave() -> None:
        with prefetched(counted(pulled), depth=DEPTH) as items:
            next(items)
            wait_until(lambda: len(pulled) == DEPTH + 2)

    assert finishes(read_one_then_leave)
    assert threading.active_count() == before


def test_consumer_error_ends_the_reader_and_wins() -> None:
    before = threading.active_count()
    raised: list[BaseException] = []

    def fail_while_reading() -> None:
        try:
            with prefetched(itertools.count(), depth=DEPTH) as items:
                next(items)
                raise ReaderTestError
        except ReaderTestError as error:
            raised.append(error)

    assert finishes(fail_while_reading)
    assert len(raised) == 1
    assert threading.active_count() == before


def test_reader_stops_exactly_one_queue_and_one_pending_item_ahead() -> None:
    pulled: list[int] = []
    ahead: list[int] = []

    def read_one() -> None:
        with prefetched(counted(pulled), depth=DEPTH) as items:
            next(items)
            wait_until(lambda: len(pulled) >= DEPTH + 2)
            time.sleep(0.05)
            ahead.append(len(pulled))

    assert finishes(read_one)
    assert ahead == [DEPTH + 2]


def test_depth_must_be_positive() -> None:
    with pytest.raises(ValueError, match="depth"), prefetched([1], depth=0):
        pass
