from __future__ import annotations

from batdetect.output.legend import ELLIPSIS, LegendEntry, column_width, fit_text, legend_panel, row_height
from batdetect.output.style import PANEL_COLOR, style_for

STYLE = style_for(1440)
COLOR = (10, 214, 255)


def entries(count: int) -> list[LegendEntry]:
    return [LegendEntry(str(i + 1), COLOR, "0:07") for i in range(count)]


def test_panel_has_the_video_height_and_one_column_when_everything_fits() -> None:
    panel = legend_panel(["video"], entries(3), 1080, STYLE)

    assert panel.shape == (1080, column_width(STYLE, []) + STYLE.margin, 3)


def test_panel_adds_columns_instead_of_dropping_tracks() -> None:
    rows_in_one_column = 1080 // row_height(STYLE)

    panel = legend_panel(["video"], entries(2 * rows_in_one_column), 1080, STYLE)

    assert panel.shape[1] >= 2 * column_width(STYLE, [])


def test_each_entry_shows_its_track_color() -> None:
    panel = legend_panel(["video"], entries(1), 1080, STYLE)

    assert COLOR in [tuple(p) for p in panel.reshape(-1, 3).tolist()]
    assert tuple(panel[-1, -1].tolist()) == PANEL_COLOR


def test_long_video_name_is_cut_with_an_ellipsis() -> None:
    text = fit_text("a_very_long_video_name_from_another_camera", 150, STYLE)

    assert text.endswith(ELLIPSIS)
    assert len(text) < len("a_very_long_video_name_from_another_camera")


def test_short_text_is_left_untouched() -> None:
    assert fit_text("092", 500, STYLE) == "092"


def test_column_widens_for_start_times_past_one_hundred_minutes() -> None:
    assert column_width(STYLE, ["100:00"]) > column_width(STYLE, ["0:07"])
