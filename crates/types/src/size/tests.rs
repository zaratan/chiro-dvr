use std::num::NonZeroU32;

use super::SourceSize;

#[test]
fn a_source_size_keeps_width_and_height_apart() {
    let size = SourceSize::new(NonZeroU32::new(1440).unwrap(), NonZeroU32::new(1080).unwrap());
    assert_eq!((size.width().get(), size.height().get()), (1440, 1080));
}
