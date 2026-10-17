use reference::recording::Recording;
use types::detection::{Detection, FrameDetections, Measures};
use types::frame::FrameIndex;
use types::rate::FrameRate;
use types::rational::Rational;
use types::span::Span;

use super::{Agreement, DetectionCounts, Gap, SAME_POINT_PX, clock, counts};
use crate::testing::{extract, nothing_detected};

const EXTRACT_VIDEO: &str = "video_092_original_3m24-4m05.mp4";

fn rate() -> FrameRate {
    FrameRate::new(Rational::new(30, 1).unwrap()).unwrap()
}

fn edited(detections: &FrameDetections, edit: impl Fn(&mut Vec<Detection>)) -> FrameDetections {
    let mut edited = FrameDetections::default();
    for (_, found) in detections.iter() {
        let mut found = found.to_vec();
        edit(&mut found);
        edited.push(found.iter().map(Measures::of));
    }
    edited
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

fn agreement(detections: &FrameDetections, recording: &Recording) -> Agreement {
    Agreement::of(EXTRACT_VIDEO, detections, rate(), recording, &[])
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

    let agreement = agreement(recording.frames.detections(), &recording);

    assert_eq!((agreement.reference_rate(), agreement.candidate_rate()), (1.0, 1.0));
    assert!(!agreement.differs());
    assert_eq!(
        agreement.counts(),
        DetectionCounts {
            reference: 403,
            current: 403,
            differing_frames: 0,
            paired: 403,
            lost: 0,
            added: 0
        }
    );
    assert_eq!(agreement.name(), "video_092_original_3m24-4m05.mp4");
    assert_eq!(written(|out| agreement.write_groups(out)), "");
}

#[test]
fn a_detection_moved_by_exactly_3_px_is_lost_and_added_and_one_moved_by_2_9_px_pairs_but_still_makes_its_frame_differ()
{
    let recording = extract();
    let frame = first_frame_with_detections(&recording);
    let moved_by = |shift: f64| {
        edited(recording.frames.detections(), |found| {
            if let Some(first) = found.first_mut().filter(|first| first.frame == frame) {
                *first = shifted(first, shift);
            }
        })
    };

    let far = agreement(&moved_by(3.0), &recording);
    let near = agreement(&moved_by(2.9), &recording);

    assert_eq!(lost_and_added(&far), (1, 1));
    assert_eq!(lost_and_added(&near), (0, 0));
    assert_eq!(near.counts().differing_frames, 1);
    assert!(near.differs());
}

#[test]
fn a_detection_whose_amplitude_alone_changed_pairs_but_makes_its_frame_differ_and_is_listed() {
    let recording = extract();
    let frame = first_frame_with_detections(&recording);
    let brighter = edited(recording.frames.detections(), |found| {
        if let Some(first) = found.first_mut().filter(|first| first.frame == frame) {
            first.amplitude += 0.001;
        }
    });

    let agreement = agreement(&brighter, &recording);

    assert_eq!(lost_and_added(&agreement), (0, 0));
    assert_eq!(agreement.counts().differing_frames, 1);
    assert!(written(|out| agreement.write_summary(out)).starts_with(&format!(
        "video_092_original_3m24-4m05.mp4\tdiffering frames 1: {frame}\t"
    )));
}

#[test]
fn missing_detections_lower_the_reference_rate_and_differ() {
    let recording = extract();
    let total = recording.frames.detections().detection_count();
    let lone = edited(recording.frames.detections(), |found| found.truncate(1));

    let agreement = agreement(&lone, &recording);

    assert!((agreement.reference_rate() - lone.detection_count() as f64 / total as f64).abs() < 1e-12);
    assert_eq!(agreement.candidate_rate(), 1.0);
    assert!(agreement.differs());
}

#[test]
fn extra_detections_lower_the_current_rate_and_differ() {
    let recording = extract();
    let doubled = edited(recording.frames.detections(), |found| {
        let far: Vec<Detection> = found.iter().map(|detection| shifted(detection, 50.0)).collect();
        found.extend(far);
    });

    let agreement = agreement(&doubled, &recording);

    assert_eq!((agreement.reference_rate(), agreement.candidate_rate()), (1.0, 0.5));
    assert!(agreement.differs());
}

#[test]
fn detections_inside_the_ignored_spans_count_on_neither_side_nor_as_differing_frames() {
    let recording = extract();
    let frame = first_frame_with_detections(&recording);
    let ignored = [Span::new(frame, frame).unwrap()];
    let reference = recording.frames.detections();
    let only_current = edited(reference, |found| {
        if let Some(first) = found.first().filter(|first| first.frame == frame) {
            found.push(shifted(first, 50.0));
        }
    });
    let only_reference = edited(reference, |found| {
        found.retain(|detection| detection.frame != frame);
    });

    let with_extra = Agreement::of(EXTRACT_VIDEO, &only_current, rate(), &recording, &ignored);
    let with_missing = Agreement::of(EXTRACT_VIDEO, &only_reference, rate(), &recording, &ignored);

    assert!(!with_extra.differs());
    assert!(!with_missing.differs());
    assert!(agreement(&only_current, &recording).differs());
}

#[test]
fn nothing_to_find_on_either_side_agrees_fully() {
    let mut recording = extract();
    let empty = nothing_detected(&recording);
    recording.frames = reference::frames::Frames::new(recording.frames.entries().to_vec(), empty.clone()).unwrap();
    recording.tracks.clear();

    let agreement = agreement(&empty, &recording);

    assert_eq!((agreement.reference_rate(), agreement.candidate_rate()), (1.0, 1.0));
}

#[test]
fn a_lost_detection_on_a_reference_track_is_told_apart_from_a_lone_one() {
    let recording = extract();
    let point = recording.tracks[0].points()[0];
    let without_point = edited(recording.frames.detections(), |found| {
        found.retain(|detection| detection.frame != point.frame || detection.distance(&point) >= SAME_POINT_PX);
    });

    let agreement = agreement(&without_point, &recording);

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
    let mut agreement = agreement(recording.frames.detections(), &recording);
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
