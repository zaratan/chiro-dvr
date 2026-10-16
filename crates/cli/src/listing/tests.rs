use std::path::{Path, PathBuf};

use draw::color::Rgb;
use files::params::TrackEntry;
use render::clips::{Clip, Outputs};
use summary::Painted;
use types::detection::Detection;
use types::frame::{FrameIndex, Pts};
use types::rate::FrameRate;
use types::rational::Rational;
use types::timeline::Timeline;
use types::track::Track;

use super::listed;

fn ten_per_second() -> Timeline {
    let rate = FrameRate::new(Rational::new(10, 1).unwrap()).unwrap();
    let pts = (0..100).map(|frame| Some(Pts::new(frame))).collect();
    Timeline::new(pts, Rational::new(1, 10).unwrap(), rate).unwrap()
}

fn seen(id: usize, frames: [usize; 2]) -> Track {
    let points = frames
        .map(|frame| Detection {
            frame: FrameIndex::new(frame),
            x: 10.0,
            y: 10.0,
            left: 9.0,
            top: 9.0,
            width: 2.0,
            height: 2.0,
            area: 4.0,
            amplitude: 30.0,
        })
        .to_vec();
    Track::new(id, points).unwrap()
}

fn clip(track: usize, path: &str) -> Clip {
    Clip {
        track,
        path: PathBuf::from(path),
    }
}

#[test]
fn each_track_takes_its_times_its_colour_and_the_clips_written_for_its_position() {
    let tracks = [seen(1, [12, 30]), seen(2, [45, 46])];
    let painted = [
        Painted {
            track: 2,
            color: Rgb::new(76, 201, 240),
        },
        Painted {
            track: 1,
            color: Rgb::new(255, 214, 10),
        },
    ];
    let outputs = Outputs {
        clips: vec![
            clip(0, "out/v/extraits/piste_01_0m01s.mp4"),
            clip(1, "out/v/extraits/piste_02_0m04s.mp4"),
        ],
        zooms: vec![clip(1, "out/v/extraits/piste_02_0m04s_zoom.mp4")],
        annotated: None,
    };
    let entries = listed(Path::new("out/v"), &tracks, &ten_per_second(), &painted, Some(&outputs));
    assert_eq!(
        entries,
        [
            TrackEntry {
                id: 1,
                start_s: 1.2,
                duration_s: 1.8,
                color: Some([255, 214, 10]),
                clip: Some("extraits/piste_01_0m01s.mp4".to_owned()),
                zoom: None,
            },
            TrackEntry {
                id: 2,
                start_s: 4.5,
                duration_s: 0.1,
                color: Some([76, 201, 240]),
                clip: Some("extraits/piste_02_0m04s.mp4".to_owned()),
                zoom: Some("extraits/piste_02_0m04s_zoom.mp4".to_owned()),
            },
        ]
    );
}

#[test]
fn without_clips_written_a_track_keeps_its_times_and_colour_but_names_no_file() {
    let tracks = [seen(1, [12, 30])];
    let painted = [Painted {
        track: 1,
        color: Rgb::new(255, 214, 10),
    }];
    let entries = listed(Path::new("out/v"), &tracks, &ten_per_second(), &painted, None);
    assert_eq!((entries[0].clip.as_deref(), entries[0].zoom.as_deref()), (None, None));
    assert_eq!(entries[0].color, Some([255, 214, 10]));
}

#[test]
fn a_track_missing_from_the_summary_has_no_colour_and_a_clip_outside_the_folder_has_no_name() {
    let tracks = [seen(1, [12, 30])];
    let outputs = Outputs {
        clips: vec![clip(0, "elsewhere/piste_01_0m01s.mp4")],
        ..Outputs::default()
    };
    let entries = listed(Path::new("out/v"), &tracks, &ten_per_second(), &[], Some(&outputs));
    assert_eq!((entries[0].color, entries[0].clip.as_deref()), (None, None));
}
