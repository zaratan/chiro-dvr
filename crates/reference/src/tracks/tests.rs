use std::fs;

use types::detection::Detection;
use types::frame::FrameIndex;
use types::track::Track;

use super::{read, write};
use crate::error::Error;

fn point(frame: usize, x: f64) -> Detection {
    Detection {
        frame: FrameIndex::new(frame),
        x,
        y: 497.022_727_272_727_25,
        left: x - 2.5,
        top: 495.0,
        width: 6.0,
        height: 6.0,
        area: 24.75,
        amplitude: 21.334_605_477_072_97,
    }
}

#[test]
fn tracks_keep_their_numbers_and_every_point() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tracks.jsonl");
    let tracks = vec![
        Track::new(1, vec![point(222, 169.2), point(224, 198.25)]).unwrap(),
        Track::new(2, vec![point(1019, 10.0)]).unwrap(),
    ];
    write(&path, &tracks).unwrap();
    assert!(
        fs::read_to_string(&path)
            .unwrap()
            .starts_with("{\"id\":1,\"points\":[{\"frame\":222,\"x\":169.2,")
    );
    assert_eq!(read(&path).unwrap(), tracks);
}

#[test]
fn a_track_without_points_is_reported_with_its_line() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tracks.jsonl");
    fs::write(&path, "{\"id\":4,\"points\":[]}\n").unwrap();
    let error = read(&path).unwrap_err();
    assert!(matches!(error, Error::Track { line: 1, .. }));
    assert!(error.to_string().ends_with("tracks.jsonl:1 holds an invalid track"));
}
