from __future__ import annotations

from pathlib import Path

import pytest

from helpers import assert_matches_reference, tracks_with_defaults

WHOLE_092 = Path(__file__).parent / "fixtures" / "video_092_original.mp4"
LFS_POINTER_MAX_BYTES = 1024


@pytest.mark.reference
def test_whole_092_keeps_its_fourteen_tracks_within_the_noise_between_platforms() -> None:
    if WHOLE_092.stat().st_size <= LFS_POINTER_MAX_BYTES:
        pytest.fail(f"{WHOLE_092.name} is a Git LFS pointer: run git lfs pull")
    tracks, _ = tracks_with_defaults(WHOLE_092)

    assert_matches_reference(
        tracks,
        start_frames=[222, 1019, 1550, 1614, 2182, 3280, 3688, 4344, 4714, 4923, 6294, 6855, 6937, 7126],
        hits=[17, 39, 9, 34, 31, 23, 12, 11, 7, 44, 287, 16, 21, 18],
    )
