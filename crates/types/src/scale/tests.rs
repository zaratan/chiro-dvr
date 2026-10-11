use std::num::NonZeroU32;

use super::WorkScale;
use crate::size::SourceSize;

fn size(width: u32, height: u32) -> SourceSize {
    SourceSize::new(NonZeroU32::new(width).unwrap(), NonZeroU32::new(height).unwrap())
}

fn scale(width: u32, height: u32, requested: u32) -> WorkScale {
    WorkScale::new(size(width, height), NonZeroU32::new(requested).unwrap())
}

#[test]
fn the_binoculars_frame_is_worked_at_two_thirds() {
    let binoculars = scale(1440, 1080, 960);
    assert_eq!(
        (binoculars.work_width().get(), binoculars.work_height().get()),
        (960, 720)
    );
    assert_eq!(binoculars.source(), size(1440, 1080));
    assert!((binoculars.factor() - 1.5).abs() < 1e-12);
}

#[test]
fn a_frame_narrower_than_the_work_width_is_not_enlarged() {
    let small = scale(640, 480, 960);
    assert_eq!((small.work_width().get(), small.work_height().get()), (640, 480));
    assert!((small.factor() - 1.0).abs() < 1e-12);
}

#[test]
fn a_very_flat_frame_keeps_at_least_one_work_row() {
    assert_eq!(scale(4000, 1, 960).work_height().get(), 1);
}

#[test]
fn a_pixel_centre_maps_to_the_centre_of_the_source_pixels_it_covers() {
    let binoculars = scale(1440, 1080, 960);
    assert!((binoculars.coordinate_to_source(0.0) - 0.25).abs() < 1e-12);
    assert!((binoculars.coordinate_to_source(959.0) - 1439.0 + 0.25).abs() < 1e-12);
}

#[test]
fn lengths_scale_once_and_areas_twice() {
    let binoculars = scale(1440, 1080, 960);
    assert!((binoculars.length_to_source(2.0) - 3.0).abs() < 1e-12);
    assert!((binoculars.area_to_source(2.0) - 4.5).abs() < 1e-12);
}
