use std::fs;

use engine::config::DetectConfig;
use engine::detection::detect;
use files::names::Names;
use media::decode::Threads;
use media::encode::Codec;
use render::config::RenderConfig;
use types::detection::Detection;
use types::frame::FrameIndex;
use types::language::Language;
use types::timeline::Timeline;
use types::track::Track;

use super::{Clips, Error, render_clips};
use crate::flight;
use crate::jobs::Job;

fn still(id: usize) -> Track {
    let points = (0..3)
        .map(|frame| Detection {
            frame: FrameIndex::new(frame),
            x: 100.0,
            y: 100.0,
            left: 95.0,
            top: 95.0,
            width: 10.0,
            height: 10.0,
            area: 100.0,
            amplitude: 40.0,
        })
        .collect();
    Track::new(id, points).unwrap()
}

#[test]
fn above_the_track_limit_the_clips_folder_is_emptied_and_no_clip_is_written() {
    let folder = tempfile::tempdir().unwrap();
    let video = folder.path().join("flight.mp4");
    flight::write_flight(&video);
    let detected = detect(&video, Threads::Automatic, &DetectConfig::default()).unwrap();
    let reading = detected.reading();
    let pts = reading.frames().iter().map(|frame| frame.pts()).collect();
    let timeline = Timeline::new(pts, reading.video().time_base(), reading.video().frame_rate()).unwrap();
    let job = Job {
        video,
        dest: folder.path().join("out"),
    };
    let names = Names::new(Language::English);
    let clips_dir = names.clips_dir(&job.dest);
    fs::create_dir_all(&clips_dir).unwrap();
    fs::write(clips_dir.join("track_01_0m00s.mp4"), "").unwrap();
    let clips = Clips {
        job: &job,
        names,
        detected: &detected,
        timeline: &timeline,
        render: RenderConfig::new(12, 1.0, 1.5, 1).unwrap(),
    };
    let error = render_clips(&clips, &[still(1), still(2)], &[Codec::OpenH264]).unwrap_err();
    assert!(matches!(error, Error::TooManyTracks { count: 2, limit: 1 }));
    assert_eq!(clips_dir.read_dir().unwrap().count(), 0);
}
