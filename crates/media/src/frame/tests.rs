use super::{ColorRange, Error, YuvFrame};

#[test]
fn chroma_planes_round_up_for_odd_sizes() {
    let frame = YuvFrame::new(5, 3, ColorRange::Limited).unwrap();
    assert_eq!((frame.y().len(), frame.u().len(), frame.v().len()), (15, 6, 6));
}

#[test]
fn a_zero_size_is_refused() {
    assert_eq!(
        YuvFrame::new(0, 1080, ColorRange::Limited),
        Err(Error::EmptySize { width: 0, height: 1080 })
    );
}

#[test]
fn a_size_that_overflows_memory_is_refused() {
    assert_eq!(
        YuvFrame::new(u32::MAX, u32::MAX, ColorRange::Full),
        Err(Error::TooLarge {
            width: u32::MAX,
            height: u32::MAX
        })
    );
}

#[test]
fn a_refused_reshape_leaves_the_frame_as_it_was() {
    let mut frame = YuvFrame::new(4, 2, ColorRange::Limited).unwrap();
    let before = frame.clone();
    assert!(frame.reshape(u32::MAX, u32::MAX, ColorRange::Full).is_err());
    assert_eq!(frame, before);
}

#[test]
fn reshaping_to_the_same_size_keeps_the_buffers() {
    let mut frame = YuvFrame::new(1440, 1080, ColorRange::Limited).unwrap();
    let luma = frame.y().as_ptr();
    frame.reshape(1440, 1080, ColorRange::Full).unwrap();
    assert_eq!((frame.y().as_ptr(), frame.range()), (luma, ColorRange::Full));
}

#[test]
fn planes_report_their_row_widths() {
    let mut frame = YuvFrame::new(5, 3, ColorRange::Limited).unwrap();
    frame.y_mut()[0] = 7;
    frame.u_mut()[0] = 8;
    frame.v_mut()[0] = 9;
    let rows: Vec<(u8, usize)> = frame.planes().iter().map(|(plane, row)| (plane[0], *row)).collect();
    assert_eq!(rows, [(7, 5), (8, 3), (9, 3)]);
    assert_eq!(frame.planes_mut().map(|(plane, row)| plane.len() / row), [3, 2, 2]);
    assert_eq!((frame.width(), frame.height()), (5, 3));
}
