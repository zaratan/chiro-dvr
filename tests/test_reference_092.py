from __future__ import annotations

from pathlib import Path

import pytest

from helpers import assert_matches_reference, tracks_with_defaults

WHOLE_092 = Path(__file__).parent / "fixtures" / "video_092_original.mp4"


@pytest.mark.reference
def test_whole_092_keeps_its_fourteen_tracks_within_the_noise_between_platforms() -> None:
    tracks, _ = tracks_with_defaults(WHOLE_092)

    assert_matches_reference(
        tracks,
        start_frames=[222, 1019, 1550, 1614, 2180, 3271, 3688, 4343, 4714, 4923, 6294, 6855, 6933, 7122],
        hits=[20, 40, 7, 34, 35, 32, 17, 13, 10, 44, 290, 17, 24, 27],
    )
