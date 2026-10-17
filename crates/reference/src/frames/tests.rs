use std::fs;

use types::detection::{FrameDetections, Measures, Strength};
use types::frame::Pts;

use super::{FrameEntry, Frames};
use crate::error::Error;

fn measures() -> Measures {
    Measures {
        x: 1400.5,
        y: 246.25,
        left: 1399.5,
        top: 246.0,
        width: 3.0,
        height: 1.5,
        area: 4.5,
        amplitude: 13.370_320_638_020_829,
        strength: Strength::Strong,
    }
}

fn entry(pts: Option<i64>) -> FrameEntry {
    FrameEntry {
        pts: pts.map(Pts::new),
        key_frame: pts == Some(0),
        decode_error: pts.is_none(),
    }
}

#[test]
fn empty_frames_and_missing_pts_survive_a_round_trip() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("frames.jsonl");
    let mut detections = FrameDetections::default();
    detections.push([measures(), measures()]);
    detections.push([]);
    detections.push([measures()]);
    let frames = Frames::new(vec![entry(Some(0)), entry(None), entry(Some(666))], detections).unwrap();
    frames.write(&path).unwrap();
    assert!(
        fs::read_to_string(&path)
            .unwrap()
            .contains("{\"frame\":1,\"pts\":null,\"key_frame\":false,\"decode_error\":true,\"detections\":[]}")
    );
    assert_eq!(Frames::read(&path).unwrap(), frames);
    assert_eq!(frames.pts(), [Some(Pts::new(0)), None, Some(Pts::new(666))]);
}

#[test]
fn a_skipped_frame_number_is_refused() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("frames.jsonl");
    fs::write(
        &path,
        "{\"frame\":0,\"pts\":0,\"key_frame\":true,\"decode_error\":false,\"detections\":[]}\n\
         {\"frame\":2,\"pts\":666,\"key_frame\":false,\"decode_error\":false,\"detections\":[]}\n",
    )
    .unwrap();
    let error = Frames::read(&path).unwrap_err();
    assert!(matches!(
        error,
        Error::FrameOrder {
            line: 2,
            expected: 1,
            found: 2,
            ..
        }
    ));
}

#[test]
fn a_frame_table_and_detections_of_different_lengths_are_refused() {
    let mut detections = FrameDetections::default();
    detections.push([]);
    detections.push([]);
    let error = Frames::new(vec![entry(Some(0))], detections).unwrap_err();
    assert!(matches!(
        error,
        Error::FrameCount {
            entries: 1,
            detections: 2
        }
    ));
}
