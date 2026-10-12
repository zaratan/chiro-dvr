from __future__ import annotations

import importlib.util
import sys
from pathlib import Path
from types import ModuleType

import pytest

from batdetect.detect import DetectConfig, Region

TOOL = Path(__file__).resolve().parent.parent / "tools" / "reference_v011.py"


def load_tool() -> ModuleType:
    spec = importlib.util.spec_from_file_location("reference_v011", TOOL)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


tool = load_tool()


def test_without_detection_settings_the_reference_keeps_the_defaults() -> None:
    assert tool.detect_config(None) == DetectConfig()


def test_detection_settings_replace_only_the_fields_they_name() -> None:
    assert tool.detect_config('{"threshold": 25, "work_width": 480}') == DetectConfig(threshold=25, work_width=480)


def test_osd_regions_are_read_as_four_bounds_each() -> None:
    chosen = tool.detect_config('{"osd_regions": [[0, 0, 1, 0.07]]}')

    assert chosen.osd_regions == (Region(0, 0, 1, 0.07),)


def test_osd_regions_written_by_a_recording_replay_as_they_were_written() -> None:
    chosen = tool.detect_config('{"osd_regions": [{"x0": 0, "y0": 0, "x1": 1, "y1": 0.07}]}')

    assert chosen.osd_regions == (Region(0, 0, 1, 0.07),)


@pytest.mark.parametrize(
    ("settings", "reason"),
    [
        ('{"treshold": 25}', "does not have: treshold"),
        ("[25]", "JSON object"),
        ("{threshold: 25}", "not JSON"),
        ('{"threshold": -1}', "threshold must be > 0"),
        ('{"osd_regions": [[0, 0, 2, 1]]}', "a region must satisfy"),
        ('{"osd_regions": [[0, 0, 1]]}', "not a valid DetectConfig"),
        ('{"work_width": 480.0}', "whole numbers for work_width"),
        ('{"bg_step": true}', "whole numbers for bg_step"),
    ],
)
def test_settings_a_recording_could_not_replay_stop_it_before_any_detection(settings: str, reason: str) -> None:
    with pytest.raises(SystemExit, match=reason):
        tool.detect_config(settings)
