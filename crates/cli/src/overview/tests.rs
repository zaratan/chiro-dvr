use draw::canvas::PixelBox;
use engine::settings::Region;

use super::display_boxes;

#[test]
fn a_display_region_is_painted_from_its_rounded_down_start_to_its_rounded_up_end_included() {
    let band = Region {
        x0: 0.15,
        y0: 0.0,
        x1: 0.35,
        y1: 0.07,
    };

    assert_eq!(
        display_boxes(&[band], 1440, 1080),
        [PixelBox {
            left: 216,
            top: 0,
            right: 503,
            bottom: 75,
        }]
    );
}

#[test]
fn without_display_region_nothing_is_painted() {
    assert_eq!(display_boxes(&[], 1440, 1080), []);
}
