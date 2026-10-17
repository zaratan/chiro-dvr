use compare::case::Target;
use engine::settings::DetectSettings;
use reference::bench::{INJECTED_FILE, REPORT_FILE, TARGETS_FILE, read_detections, read_report, read_targets};
use reference::location::{Reference, committed_bench};
use reference::recording::Recording;
use reference::video::DECODED_FORMAT;

const COMMITTED: [Target; 4] = [Target::Extract, Target::Whole, Target::WholeAt1440, Target::Settings];

#[test]
fn each_committed_case_of_the_v020_reads_in_the_second_format_with_the_video_and_settings_of_the_v011() {
    let pairs: Vec<_> = COMMITTED
        .iter()
        .flat_map(|target| {
            let (v011, v020) = (
                target.cases(Reference::V011).unwrap(),
                target.cases(Reference::V020).unwrap(),
            );
            assert_eq!(v011.len(), v020.len(), "{target:?}");
            v011.into_iter().zip(v020)
        })
        .collect();

    assert_eq!(pairs.len(), 19);
    for (v011, v020) in &pairs {
        assert_eq!(v011.folder(), v020.folder());
        let (probed, decoded) = (
            Recording::read(v011.recording()).unwrap(),
            Recording::read(v020.recording()).unwrap(),
        );
        let case = v020.recording().display();
        assert_eq!(decoded.summary.format_version, DECODED_FORMAT, "{case}");
        assert_eq!(decoded.summary.reference, "v0.2.0", "{case}");
        assert_eq!(
            (&decoded.summary.video, &decoded.summary.stream),
            (&probed.summary.video, &probed.summary.stream),
            "{case}"
        );
        assert_eq!(
            decoded.summary.detect_settings::<DetectSettings>().unwrap(),
            probed.summary.detect_settings::<DetectSettings>().unwrap(),
            "{case}"
        );
        assert_eq!(decoded.frames.entries().len(), probed.frames.entries().len(), "{case}");
        assert_eq!(decoded.summary.damaged_spans, [], "{case}");
        let counts = &decoded.summary.counts;
        assert_eq!(
            (
                decoded.frames.entries().len(),
                decoded.frames.detections().detection_count(),
                decoded.tracks.len(),
                decoded.tracks.iter().map(|track| track.points().len()).sum::<usize>()
            ),
            (counts.frames, counts.detections, counts.tracks, counts.track_points),
            "{case}"
        );
    }
}

#[test]
fn both_benches_of_the_v020_keep_as_many_targets_as_the_v011_drew_and_read_back() {
    for (name, count) in [("transit", 270), ("hunt", 720)] {
        let folder = committed_bench(name).join("video_092_original");

        let targets = read_targets(&folder.join(TARGETS_FILE)).unwrap();

        assert_eq!(targets.len(), count, "{name}");
        assert!(read_report(&folder.join(REPORT_FILE)).is_ok(), "{name}");
        assert!(read_detections(&folder.join(INJECTED_FILE)).is_ok(), "{name}");
    }
}
