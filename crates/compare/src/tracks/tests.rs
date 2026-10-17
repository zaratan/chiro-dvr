use reference::location::{Reference, committed};
use reference::recording::Recording;
use reference::track_matching::{Discrepancy, DiscrepancyKind};
use types::frame::FrameIndex;
use types::rate::FrameRate;
use types::rational::Rational;
use types::size::SourceSize;

use super::{TrackAgreement, TrackCounts};
use crate::case::EXTRACT;
use crate::testing::nothing_detected;

const EXTRACT_VIDEO: &str = "video_092_original_3m24-4m05.mp4";

fn extract() -> Recording {
    Recording::read(&committed(Reference::V011, EXTRACT)).unwrap()
}

fn size(recording: &Recording) -> SourceSize {
    recording.summary.stream.size().unwrap()
}

fn rate() -> FrameRate {
    FrameRate::new(Rational::new(30_000, 1001).unwrap()).unwrap()
}

fn written(write: impl FnOnce(&mut Vec<u8>) -> std::io::Result<()>) -> String {
    let mut out = Vec::new();
    write(&mut out).unwrap();
    String::from_utf8(out).unwrap()
}

#[test]
fn the_reference_detections_tracked_again_give_back_the_four_tracks_of_the_extract() {
    let recording = extract();

    let agreement = TrackAgreement::of(
        EXTRACT_VIDEO,
        recording.frames.detections(),
        (rate(), size(&recording)),
        &recording,
    )
    .unwrap();

    assert_eq!(
        agreement.counts(),
        TrackCounts {
            reference: 4,
            paired: 4,
            agreeing: 4,
            lost: 0,
            added: 0
        }
    );
    assert_eq!(written(|out| agreement.write_discrepancies(out)), "");
}

#[test]
fn without_detections_every_reference_track_is_lost_and_listed_with_its_time() {
    let recording = extract();
    let empty = nothing_detected(&recording);

    let agreement = TrackAgreement::of(EXTRACT_VIDEO, &empty, (rate(), size(&recording)), &recording).unwrap();

    assert_eq!((agreement.counts().lost, agreement.counts().paired), (4, 0));
    assert!(
        written(|out| agreement.write_summary(out)).ends_with("tracks 4\tpaired 0 (agreeing 0)\tlost 4\tadded 0\n")
    );
    assert_eq!(written(|out| agreement.write_discrepancies(out)).lines().count(), 4);
}

#[test]
fn each_kind_of_track_gap_is_written_on_its_own_line() {
    let at = |kind| Discrepancy {
        video: "v.mp4".to_owned(),
        frame: FrameIndex::new(90),
        seconds: 3.0,
        kind,
    };
    let agreement = TrackAgreement {
        name: "v.mp4".to_owned(),
        counts: TrackCounts::default(),
        discrepancies: vec![
            at(DiscrepancyKind::Missed { reference_id: 1 }),
            at(DiscrepancyKind::Extra { candidate_id: 2 }),
            at(DiscrepancyKind::Points {
                reference_id: 3,
                candidate_id: 4,
                reference_points: 20,
                candidate_points: 12,
            }),
        ],
    };

    assert_eq!(
        written(|out| agreement.write_discrepancies(out)),
        "  v.mp4 00:03.00 frame 90\tlost reference track 1\n\
         \x20 v.mp4 00:03.00 frame 90\tadded track 2\n\
         \x20 v.mp4 00:03.00 frame 90\treference track 3 has 20 points, track 4 has 12\n"
    );
}

#[test]
fn track_counts_add_up_field_by_field() {
    let one = TrackCounts {
        reference: 1,
        paired: 2,
        agreeing: 3,
        lost: 4,
        added: 5,
    };

    assert_eq!(
        [one, one].into_iter().sum::<TrackCounts>(),
        TrackCounts {
            reference: 2,
            paired: 4,
            agreeing: 6,
            lost: 8,
            added: 10
        }
    );
}
