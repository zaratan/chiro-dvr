from __future__ import annotations

import numpy as np

from batdetect.output.marker import draw_marker
from batdetect.output.style import style_for

COLOR = (10, 214, 255)


def test_marker_is_a_colored_disc_with_a_dark_number() -> None:
    image = np.full((100, 100, 3), 128, dtype=np.uint8)
    style = style_for(1440)

    draw_marker(image, (50, 50), "7", COLOR, style)

    disc = image[50 - style.radius + 4 : 50 + style.radius - 4, 50 - style.radius + 4 : 50 + style.radius - 4]
    pixels = disc.reshape(-1, 3).tolist()
    assert list(COLOR) in pixels
    assert [0, 0, 0] in pixels
    assert image[2, 2].tolist() == [128, 128, 128]
