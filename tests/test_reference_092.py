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
        start_frames=[222, 1019, 1550, 1614, 2182, 3280, 3688, 4344, 4714, 4923, 6294, 6855, 6937, 7126],
        hits=[17, 39, 9, 34, 31, 23, 12, 11, 7, 44, 287, 16, 21, 18],
    )
