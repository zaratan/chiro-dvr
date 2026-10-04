from __future__ import annotations

import csv
import json
from pathlib import Path

from batdetect.detect import DetectConfig
from batdetect.output.tables import config_params, write_params, write_tracks_csv
from batdetect.track import Track, TrackConfig
from batdetect.video import VideoInfo
from helpers import detection

INFO = VideoInfo(fps=30.0, frame_count=300, width=1440, height=1080, work_width=480, work_height=360)


def test_track_row_reports_times_hits_distances_and_filled_frames(tmp_path: Path) -> None:
    track = Track(3, [detection(30, 0, 0, area=20), detection(31, 30, 40, area=50), detection(33, 60, 80, area=30)])
    out = tmp_path / "t.csv"

    write_tracks_csv([track], INFO, out)

    with out.open() as fh:
        row = next(csv.DictReader(fh))
    assert (row["id"], row["start"], row["end"]) == ("3", "0:01.00", "0:01.10")
    assert (row["hits"], row["filled_frames"]) == ("3", "1")
    assert (row["chord_px"], row["path_px"], row["max_area_px"]) == ("100.0", "100.0", "50")
    assert row["speed_px_s"] == "1000.0"


def test_params_are_written_as_readable_utf8_json(tmp_path: Path) -> None:
    out = tmp_path / "params.json"

    write_params({"vidéo": "nuit_1.mp4"}, out)

    assert "vidéo" in out.read_text()
    assert json.loads(out.read_text()) == {"vidéo": "nuit_1.mp4"}


def test_configs_are_keyed_by_their_class_name() -> None:
    params = config_params(DetectConfig(threshold=30), TrackConfig())

    assert params["DetectConfig"]["threshold"] == 30
    assert set(params) == {"DetectConfig", "TrackConfig"}
