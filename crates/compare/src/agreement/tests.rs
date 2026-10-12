use reference::location::committed;
use reference::recording::Recording;
use types::detection::{Detection, FrameDetections};
use types::frame::FrameIndex;
use types::rate::FrameRate;
use types::rational::Rational;
use types::span::Span;

use super::{Agreement, Gap, SAME_POINT_PX, clock, counts};
use crate::case::EXTRACT;

const EXTRACT_VIDEO: &str = "video_092_original_3m24-4m05.mp4";

fn extract() -> Recording {
    Recording::read(&committed(EXTRACT)).unwrap()
}

fn rate() -> FrameRate {
    FrameRate::new(Rational::new(30, 1).unwrap()).unwrap()
}

fn edited(detections: &FrameDetections, edit: impl Fn(&mut Vec<Detection>)) -> FrameDetections {
    FrameDetections::new(
        detections
            .iter()
            .map(|(_, found)| {
                let mut found = found.to_vec();
                edit(&mut found);
                found
            })
            .collect(),
    )
    .unwrap()
}

fn first_frame_with_detections(recording: &Recording) -> FrameIndex {
    recording
        .frames
        .detections()
        .iter()
        .find(|(_, found)| !found.is_empty())
        .unwrap()
        .0
}

fn lost_and_added(agreement: &Agreement) -> (usize, usize) {
    let (lost, _, added) = counts(&agreement.gaps);
    (lost, added)
}

fn shifted(detection: &Detection, by: f64) -> Detection {
    Detection {
        x: detection.x + by,
        ..*detection
    }
}

fn written(write: impl FnOnce(&mut Vec<u8>) -> std::io::Result<()>) -> String {
    let mut out = Vec::new();
    write(&mut out).unwrap();
    String::from_utf8(out).unwrap()
}

#[test]
fn the_reference_agrees_with_itself_both_ways() {
    let recording = extract();

    let agreement = Agreement::of(EXTRACT_VIDEO, recording.frames.detections(), rate(), &recording);

    assert_eq!((agreement.reference_rate(), agreement.candidate_rate()), (1.0, 1.0));
    assert!(agreement.holds());
    assert_eq!(agreement.name(), "video_092_original_3m24-4m05.mp4");
    assert_eq!(written(|out| agreement.write_groups(out)), "");
}

#[test]
fn a_detection_moved_by_exactly_3_px_is_lost_and_added_but_not_at_2_9_px() {
    let recording = extract();
    let frame = first_frame_with_detections(&recording);
    let moved_by = |shift: f64| {
        edited(recording.frames.detections(), |found| {
            if let Some(first) = found.first_mut().filter(|first| first.frame == frame) {
                *first = shifted(first, shift);
            }
        })
    };

    let far = Agreement::of(EXTRACT_VIDEO, &moved_by(3.0), rate(), &recording);
    let near = Agreement::of(EXTRACT_VIDEO, &moved_by(2.9), rate(), &recording);

    assert_eq!(lost_and_added(&far), (1, 1));
    assert_eq!(lost_and_added(&near), (0, 0));
}

#[test]
fn missing_detections_lower_the_reference_rate_under_98_percent() {
    let recording = extract();
    let total = recording.frames.detections().detection_count();
    let lone = edited(recording.frames.detections(), |found| found.truncate(1));

    let agreement = Agreement::of(EXTRACT_VIDEO, &lone, rate(), &recording);

    assert!((agreement.reference_rate() - lone.detection_count() as f64 / total as f64).abs() < 1e-12);
    assert_eq!(agreement.candidate_rate(), 1.0);
    assert!(!agreement.holds());
}

#[test]
fn extra_detections_lower_the_candidate_rate_under_98_percent() {
    let recording = extract();
    let doubled = edited(recording.frames.detections(), |found| {
        let far: Vec<Detection> = found.iter().map(|detection| shifted(detection, 50.0)).collect();
        found.extend(far);
    });

    let agreement = Agreement::of(EXTRACT_VIDEO, &doubled, rate(), &recording);

    assert_eq!((agreement.reference_rate(), agreement.candidate_rate()), (1.0, 0.5));
    assert!(!agreement.holds());
}

#[test]
fn detections_inside_the_ignored_spans_of_the_reference_count_on_neither_side() {
    let mut recording = extract();
    let frame = first_frame_with_detections(&recording);
    recording.summary.unstable_spans = vec![Span::new(frame, frame).unwrap()];
    let reference = recording.frames.detections();
    let only_rust = edited(reference, |found| {
        if let Some(first) = found.first().filter(|first| first.frame == frame) {
            found.push(shifted(first, 50.0));
        }
    });
    let only_reference = edited(reference, |found| {
        found.retain(|detection| detection.frame != frame);
    });

    let with_extra = Agreement::of(EXTRACT_VIDEO, &only_rust, rate(), &recording);
    let with_missing = Agreement::of(EXTRACT_VIDEO, &only_reference, rate(), &recording);

    assert_eq!(lost_and_added(&with_extra), (0, 0));
    assert_eq!(lost_and_added(&with_missing), (0, 0));
}

#[test]
fn nothing_to_find_on_either_side_agrees_fully() {
    let mut recording = extract();
    let empty = edited(recording.frames.detections(), Vec::clear);
    recording.frames = reference::frames::Frames::new(recording.frames.entries().to_vec(), empty.clone()).unwrap();
    recording.tracks.clear();

    let agreement = Agreement::of(EXTRACT_VIDEO, &empty, rate(), &recording);

    assert_eq!((agreement.reference_rate(), agreement.candidate_rate()), (1.0, 1.0));
}

#[test]
fn a_lost_detection_on_a_reference_track_is_told_apart_from_a_lone_one() {
    let recording = extract();
    let point = recording.tracks[0].points()[0];
    let without_point = edited(recording.frames.detections(), |found| {
        found.retain(|detection| detection.frame != point.frame || detection.distance(&point) >= SAME_POINT_PX);
    });

    let agreement = Agreement::of(EXTRACT_VIDEO, &without_point, rate(), &recording);

    assert_eq!(agreement.gaps, vec![(point, Gap::Lost { on_track: true })]);
    assert!(written(|out| agreement.write_summary(out)).contains("lost 1 (1 on tracks)\tadded 0"));
}

#[test]
fn gaps_at_most_30_frames_apart_form_one_group_and_31_frames_split() {
    let recording = extract();
    let at = |frame: usize| Detection {
        frame: FrameIndex::new(frame),
        x: 100.0,
        y: 200.0,
        left: 99.0,
        top: 199.0,
        width: 3.0,
        height: 3.0,
        area: 4.5,
        amplitude: 20.0,
    };
    let mut agreement = Agreement::of(EXTRACT_VIDEO, recording.frames.detections(), rate(), &recording);
    agreement.gaps = vec![
        (at(100), Gap::Added),
        (at(120), Gap::Lost { on_track: false }),
        (at(150), Gap::Added),
        (at(181), Gap::Added),
    ];

    let groups = written(|out| agreement.write_groups(out));

    assert_eq!(
        groups,
        "  video_092_original_3m24-4m05.mp4 00:03.33 frames 100-150\tlost 1 (0 on tracks)\tadded 2\tat (100, 200)\tpeak 20.0\n\
         \x20 video_092_original_3m24-4m05.mp4 00:06.03 frames 181-181\tlost 0 (0 on tracks)\tadded 1\tat (100, 200)\tpeak 20.0\n"
    );
}

#[test]
fn a_clock_shows_minutes_and_hundredths_of_seconds_and_carries_a_full_minute() {
    assert_eq!(clock(75.5), "01:15.50");
    assert_eq!(clock(0.0), "00:00.00");
    assert_eq!(clock(59.996), "01:00.00");
    assert_eq!(clock(299.996), "05:00.00");
}
