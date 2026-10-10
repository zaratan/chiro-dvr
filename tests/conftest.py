from __future__ import annotations

from pathlib import Path

import pytest

LOCALE_VARIABLES = ("LANGUAGE", "LC_ALL", "LC_MESSAGES", "LANG")
FIXTURES = Path(__file__).parent / "fixtures"
LFS_POINTER_MAX_BYTES = 1024
VIDEO_BY_MARKER = {
    "slow": FIXTURES / "video_092_original_3m24-4m05.mp4",
    "reference": FIXTURES / "video_092_original.mp4",
}


def pytest_runtest_setup(item: pytest.Item) -> None:
    for marker, video in VIDEO_BY_MARKER.items():
        if item.get_closest_marker(marker) and video.stat().st_size <= LFS_POINTER_MAX_BYTES:
            pytest.fail(f"{video.name} is a Git LFS pointer: run git lfs pull", pytrace=False)


@pytest.fixture(autouse=True)
def english(monkeypatch: pytest.MonkeyPatch) -> None:
    for variable in LOCALE_VARIABLES:
        monkeypatch.delenv(variable, raising=False)
    monkeypatch.setenv("PYTHON_COLORS", "0")


@pytest.fixture
def french(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("LANG", "fr_FR.UTF-8")
