use std::num::NonZeroU32;

use types::size::SourceSize;

use super::{Error, Geometry};

fn nonzero(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).unwrap()
}

fn geometry(width: u32, height: u32, work_width: u32) -> Result<Geometry, Error> {
    Geometry::new(SourceSize::new(nonzero(width), nonzero(height)), nonzero(work_width))
}

#[test]
fn the_binoculars_frame_is_worked_on_at_960_by_720_one_and_a_half_source_pixels_each() {
    let geometry = geometry(1440, 1080, 960).unwrap();

    assert_eq!(
        (geometry.width(), geometry.height(), geometry.pixels()),
        (960, 720, 691_200)
    );
    assert_eq!(geometry.source_width(), 1440);
    assert!((geometry.scale().factor() - 1.5).abs() < 1e-12);
    assert_eq!(geometry.scale().work_height().get(), 720);
}

#[test]
fn only_a_reduction_by_three_to_two_is_handled() {
    assert_eq!(
        geometry(1440, 1080, 720),
        Err(Error::Reduction {
            width: 1440,
            height: 1080,
            work_width: 720
        })
    );
    assert!(matches!(geometry(1441, 1080, 960), Err(Error::Reduction { .. })));
    assert!(matches!(geometry(1440, 1081, 960), Err(Error::Reduction { .. })));
}

#[test]
fn a_work_frame_too_small_for_the_target_filter_is_refused() {
    assert_eq!(geometry(12, 240, 8), Err(Error::TooSmall { width: 8, height: 160 }));
    assert!(matches!(geometry(240, 12, 160), Err(Error::TooSmall { .. })));
    assert!(geometry(15, 15, 10).is_ok());
}

#[test]
fn a_work_frame_whose_pixels_cannot_all_be_numbered_is_refused() {
    assert_eq!(
        geometry(98_304, 98_304, 65_536),
        Err(Error::TooLarge {
            width: 65_536,
            height: 65_536
        })
    );
}

#[test]
fn a_numbered_pixel_gives_back_its_column_and_row() {
    let geometry = geometry(1440, 1080, 960).unwrap();

    let pixel = crate::convert::numbered(3 * 960 + 7);

    assert_eq!((geometry.column(pixel), geometry.row(pixel)), (7, 3));
}
