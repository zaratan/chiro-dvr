use super::{FrameIndex, Pts};

#[test]
fn a_frame_index_gives_back_its_number() {
    assert_eq!(FrameIndex::new(9010).get(), 9010);
}

#[test]
fn the_distance_between_frames_ignores_their_order() {
    assert_eq!(FrameIndex::new(222).abs_diff(FrameIndex::new(217)), 5);
    assert_eq!(FrameIndex::new(217).abs_diff(FrameIndex::new(222)), 5);
}

#[test]
fn frames_sort_in_decoding_order() {
    assert!(FrameIndex::new(1019) < FrameIndex::new(1550));
}

#[test]
fn a_pts_keeps_its_ticks_even_when_negative() {
    assert_eq!(Pts::new(-333).get(), -333);
}

#[test]
fn frames_and_pts_are_plain_json_numbers() {
    assert_eq!(
        serde_json::to_string(&(FrameIndex::new(1301), Pts::new(2_094_237))).unwrap(),
        "[1301,2094237]"
    );
    let read: (FrameIndex, Option<Pts>) = serde_json::from_str("[7,null]").unwrap();
    assert_eq!(read, (FrameIndex::new(7), None));
}

#[test]
fn a_frame_index_prints_as_its_number() {
    assert_eq!(format!("frame {}", FrameIndex::new(6294)), "frame 6294");
}
