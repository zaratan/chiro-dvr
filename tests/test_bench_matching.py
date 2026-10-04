from __future__ import annotations

from batdetect.bench.matching import assign_tracks
from batdetect.track import Track
from bench_support import MATCH, bat_along, observations_of
from helpers import detection


def test_track_mostly_elsewhere_is_not_credited_to_a_target_it_brushes() -> None:
    bat = bat_along(0, range(10, 30))
    brush = [detection(10, 30, 60)] + [detection(f, 30 + 18 * (f - 10), 300) for f in range(11, 30)]
    track = Track(1, brush)
    truth = {o.frame: [(0, o.x, o.y)] for o in observations_of(bat)}

    assert assign_tracks([track], truth, MATCH) == {}
