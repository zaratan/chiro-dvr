use draw::canvas::PixelBox;
use media::convert::Region;

use super::{even_region, merged};

#[test]
fn a_zone_starts_on_even_pixels_and_spans_a_multiple_of_16_inside_the_image() {
    let zone = even_region(
        PixelBox {
            left: 3,
            top: -5,
            right: 20,
            bottom: 9,
        },
        64,
        48,
    )
    .unwrap();
    assert_eq!(
        zone,
        Region {
            left: 2,
            top: 0,
            width: 32,
            height: 16
        }
    );
}

#[test]
fn touching_zones_become_one_and_apart_zones_stay_apart() {
    let zone = |left| Region {
        left,
        top: 0,
        width: 16,
        height: 16,
    };
    assert_eq!(merged(vec![zone(0), zone(16)], 64, 48).len(), 1);
    assert_eq!(merged(vec![zone(0), zone(40)], 64, 48).len(), 2);
}
