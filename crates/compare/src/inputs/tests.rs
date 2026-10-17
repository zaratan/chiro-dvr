use reference::frames::FrameEntry;
use types::frame::{FrameIndex, Pts};
use types::losses::EdgeLosses;

use super::{DecodeErrors, Field, InputGap, REFERENCE_FIELDS, STRICT_FIELDS, decode_errors, entry_gap, losses_gap};

fn entries(count: usize) -> Vec<FrameEntry> {
    (0..count)
        .map(|frame| FrameEntry {
            pts: Some(Pts::new(100 * frame as i64)),
            key_frame: frame % 10 == 0,
            decode_error: false,
        })
        .collect()
}

fn in_error(mut entries: Vec<FrameEntry>, frames: &[usize]) -> Vec<FrameEntry> {
    for &frame in frames {
        entries[frame].decode_error = true;
    }
    entries
}

#[test]
fn the_same_frames_have_no_gap_whatever_fields_are_compared() {
    assert_eq!(entry_gap(&entries(20), &entries(20), &STRICT_FIELDS), None);
    assert_eq!(entry_gap(&entries(20), &entries(20), &REFERENCE_FIELDS), None);
}

#[test]
fn another_frame_count_is_told_before_any_entry() {
    let gap = entry_gap(&entries(20), &entries(19), &REFERENCE_FIELDS).unwrap();

    assert_eq!(
        gap,
        InputGap::FrameCount {
            previous: 20,
            current: 19
        }
    );
    assert_eq!(gap.to_string(), "20 frames before, 19 now");
}

#[test]
fn the_first_frame_whose_timestamp_or_key_flag_moved_is_told_with_every_field_that_moved() {
    let mut moved = entries(20);
    moved[7].pts = Some(Pts::new(701));
    moved[7].key_frame = true;
    moved[12].pts = None;

    let gap = entry_gap(&entries(20), &moved, &REFERENCE_FIELDS).unwrap();

    assert_eq!(
        gap,
        InputGap::Entry {
            frame: FrameIndex::new(7),
            fields: vec![Field::Pts, Field::KeyFrame]
        }
    );
    assert_eq!(gap.to_string(), "frame 7: pts, key frame");
}

#[test]
fn a_decoding_error_is_an_input_gap_between_two_passes_but_not_against_a_reference() {
    let erring = in_error(entries(20), &[3]);

    assert_eq!(entry_gap(&entries(20), &erring, &REFERENCE_FIELDS), None);
    assert_eq!(
        entry_gap(&entries(20), &erring, &STRICT_FIELDS),
        Some(InputGap::Entry {
            frame: FrameIndex::new(3),
            fields: vec![Field::DecodeError]
        })
    );
}

#[test]
fn decoding_errors_are_counted_on_each_side_and_where_they_differ() {
    let reference = in_error(entries(20), &[3, 4, 9]);
    let current = in_error(entries(20), &[3, 4, 15]);

    assert_eq!(
        decode_errors(&reference, &current),
        DecodeErrors {
            reference: 3,
            current: 3,
            differing: 2
        }
    );
}

#[test]
fn frames_lost_at_the_edges_must_be_the_same_on_both_sides() {
    let lost_before = EdgeLosses {
        before_first: 2,
        after_last: 0,
    };

    let gap = losses_gap(EdgeLosses::default(), lost_before).unwrap();

    assert_eq!(losses_gap(lost_before, lost_before), None);
    assert_eq!(
        gap.to_string(),
        "frames lost at the start and at the end: 0/0 before, 2/0 now"
    );
}
