use reference::detection_matching::match_detections;
use reference::location::{recorded_video, recording_directories};
use reference::recording::Recording;
use reference::track_matching::match_tracks;

#[test]
#[ignore = "local"]
fn the_twenty_six_recorded_videos_read_and_match_themselves() {
    let directories = recording_directories().unwrap();
    assert_eq!(directories.len(), 26);
    let mut tracks = 0;
    for directory in directories {
        let recording = Recording::read(&directory).unwrap();
        let counts = recording.summary.counts;
        let detections = recording.frames.detections();
        assert_eq!(
            recording.frames.entries().len(),
            counts.frames,
            "{}",
            directory.display()
        );
        assert_eq!(
            detections.detection_count(),
            counts.detections,
            "{}",
            directory.display()
        );
        assert_eq!(recording.tracks.len(), counts.tracks, "{}", directory.display());
        assert_eq!(recording.rows.len(), counts.tracks, "{}", directory.display());
        assert!(
            match_detections(detections, detections).is_complete(),
            "{}",
            directory.display()
        );
        let matching = match_tracks(&recording.tracks, &recording.tracks);
        assert_eq!(matching.agreeing(), counts.tracks, "{}", directory.display());
        assert!(
            recorded_video(&recording.summary.video.name).is_ok(),
            "{}",
            directory.display()
        );
        tracks += recording.tracks.len();
    }
    assert_eq!(tracks, 966);
}
