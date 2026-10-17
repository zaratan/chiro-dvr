use std::num::NonZeroUsize;

use engine::damage::Reports;
use reference::frames::FrameEntry;
use reference::location::{Reference, committed};
use reference::recording::SUMMARY_FILE;
use reference::video::{Decoder, DecoderThreads, VideoSummary};
use types::frame::{FrameIndex, Pts};
use types::losses::EdgeLosses;
use types::span::Span;

use super::{joined, merged, reported_by};
use crate::case::EXTRACT;

const KEY_EVERY: usize = 30;

fn frames(indices: &[usize]) -> Vec<FrameIndex> {
    indices.iter().copied().map(FrameIndex::new).collect()
}

fn span(first: usize, last: usize) -> Span {
    Span::new(FrameIndex::new(first), FrameIndex::new(last)).unwrap()
}

fn entries(count: usize) -> Vec<FrameEntry> {
    (0..count)
        .map(|frame| FrameEntry {
            pts: Some(Pts::new(frame as i64)),
            key_frame: frame % KEY_EVERY == 0,
            decode_error: false,
        })
        .collect()
}

fn summary(reference: Reference) -> VideoSummary {
    VideoSummary::read(&committed(reference, EXTRACT).join(SUMMARY_FILE)).unwrap()
}

#[test]
fn the_v020_reports_its_decoder_and_the_v011_what_ffprobe_saw_without_edge_losses() {
    let mut decoded = summary(Reference::V020);
    decoded.decoder = Some(Decoder {
        error_frames: frames(&[5]),
        gap_frames: frames(&[40]),
        losses: EdgeLosses {
            before_first: 0,
            after_last: 3,
        },
        threads: DecoderThreads::Frame(NonZeroUsize::MIN),
    });
    let mut probed = summary(Reference::V011);
    if let Some(probe) = probed.ffprobe.as_mut() {
        probe.error_frames = frames(&[8]);
        probe.gap_frames = frames(&[12]);
    }

    assert_eq!(
        reported_by(&decoded),
        Reports {
            error_frames: frames(&[5]),
            jump_frames: frames(&[40]),
            edges: EdgeLosses {
                before_first: 0,
                after_last: 3
            }
        }
    );
    assert_eq!(
        reported_by(&probed),
        Reports {
            error_frames: frames(&[8]),
            jump_frames: frames(&[12]),
            edges: EdgeLosses::default()
        }
    );
}

#[test]
fn damage_reported_on_either_side_is_drawn_again_with_the_key_frames_and_counted_once() {
    let reference = Reports {
        error_frames: frames(&[17, 40]),
        ..Reports::default()
    };
    let current = Reports {
        error_frames: frames(&[40, 65]),
        jump_frames: frames(&[17]),
        edges: EdgeLosses {
            before_first: 1,
            after_last: 0,
        },
    };

    let damage = joined(&reference, &current, &entries(90), 2);

    assert_eq!(damage.error_frames(), frames(&[17, 40, 65]));
    assert_eq!(damage.reported_frames(), 3);
    assert_eq!(damage.spans(), [span(0, 31), span(38, 61), span(63, 89)]);
}

#[test]
fn nothing_reported_on_either_side_leaves_nothing_damaged() {
    let damage = joined(&Reports::default(), &Reports::default(), &entries(90), 15);

    assert_eq!(damage.spans(), []);
    assert_eq!(damage.reported_frames(), 0);
}

#[test]
fn overlapping_or_touching_spans_merge_before_they_are_counted_and_apart_ones_stay_apart() {
    assert_eq!(
        merged([span(40, 50), span(10, 20), span(15, 25), span(26, 30), span(32, 33)]),
        [span(10, 30), span(32, 33), span(40, 50)]
    );
    assert_eq!(merged([]), []);
}
