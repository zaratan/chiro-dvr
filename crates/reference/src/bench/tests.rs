use std::fs;

use types::detection::{FrameDetections, Measures};

use super::{
    Patch, TargetLine, read_detections, read_report, read_targets, write_detections, write_report, write_targets,
};
use crate::error::Error;

const RECORDED: &str = "{\"id\":275,\"motion\":\"hunt\",\"amplitude\":-30.0,\"sigma\":3.0,\"start_frame\":6665,\"positions\":[[-11.0,7.690508084453089],[3.4211755379388538,5.05458606861673],[32.25838866711354,-0.23340641507905407],[46.57377813581866,-3.393788216207418],[60.887873531372875,-6.560024592241656],[89.59463016658198,-12.520972290731319]],\"amplitudes\":[-20.27241545996305,-24.875957589695982,-22.43597969355627,-21.943092209656083,-28.888610590859596,-23.464101367002996],\"effective\":[[6666,0.0],[6667,0.0],[6668,0.0],[6669,0.0]],\"patch\":{\"frame\":6668,\"base\":128,\"left\":37,\"top\":0,\"values\":[[-1,-1,-1,-2,-3,-4,-6,-9,-11,-12,-12,-11,-9,-7,-4,-3,-2,-1,-1],[-1,-1,-1,-1,-2,-3,-4,-6,-7,-8,-8,-7,-6,-4,-3,-2,-1,-1,-1],[-1,-1,-1,-1,-1,-2,-3,-4,-4,-5,-5,-4,-4,-3,-2,-1,-1,-1,-1],[-1,-1,-1,-1,-1,-1,-2,-2,-2,-3,-3,-3,-2,-2,-1,-1,-1,-1,-1],[-1,-1,-1,-1,-1,-1,-1,-1,-1,-2,-2,-1,-1,-1,-1,-1,-1,-1,-1],[-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1]]}}";

fn line(id: usize, patch: Option<Patch>) -> TargetLine {
    TargetLine {
        id,
        motion: "pass".to_owned(),
        amplitude: -15.0,
        sigma: 1.5,
        start_frame: 7594,
        positions: vec![
            [251.043_150_777_257_98, -6.5],
            [264.848_641_953_394_9, 1.820_425_277_056_196_2],
        ],
        amplitudes: vec![-11.832_082_387_593_422, -11.048_010_087_072_71],
        effective: vec![(7595, -1.0)],
        patch,
    }
}

#[test]
fn a_target_recorded_by_the_v011_reads_and_writes_back_to_the_same_text() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("bats.jsonl");
    fs::write(&path, format!("{RECORDED}\n")).unwrap();

    let targets = read_targets(&path).unwrap();
    write_targets(&path, &targets).unwrap();

    let read_patch = targets[0].patch.as_ref().unwrap();
    assert_eq!(
        (targets[0].id, targets[0].motion.as_str(), targets[0].start_frame),
        (275, "hunt", 6665)
    );
    assert_eq!(targets[0].positions.len(), targets[0].amplitudes.len());
    assert_eq!(targets[0].effective[0], (6666, 0.0));
    assert_eq!(
        (read_patch.frame, read_patch.base, read_patch.left, read_patch.top),
        (6668, 128, 37, 0)
    );
    assert_eq!(read_patch.values[0][9], -12);
    assert_eq!(fs::read_to_string(&path).unwrap(), format!("{RECORDED}\n"));
}

#[test]
fn targets_with_and_without_patch_go_through_a_compressed_file_unchanged() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("bats.jsonl.gz");
    let recorded_patch = Patch {
        frame: 7595,
        base: 128,
        left: 3,
        top: 0,
        values: vec![vec![-1, -2], vec![-2, -4]],
    };
    let targets = [line(0, Some(recorded_patch)), line(1, None)];

    write_targets(&path, &targets).unwrap();

    assert_eq!(read_targets(&path).unwrap(), targets);
}

#[test]
fn a_target_line_with_a_field_the_v011_does_not_write_is_refused() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("bats.jsonl");
    fs::write(&path, RECORDED.replacen("{\"id\"", "{\"speed\":3,\"id\"", 1)).unwrap();

    let refused = read_targets(&path).unwrap_err();

    assert!(matches!(refused, Error::Json { line: 1, .. }), "{refused:?}");
}

const RECORDED_REPORT: &str = r#"{"video": "video_092_original.mp4", "code": "v0.1.1",
"detect": {"threshold": 12, "osd_regions": []}, "track": {"min_hits": 6}, "stability": {"max_blobs": 20, "pad_s": 1.0},
"match": {"radius": 24.0, "purity": 0.5}, "sampling": {"seed": 1, "per_class": 30},
"summary": [{"motion": "pass", "amplitude": -60.0, "sigma": 1.5, "n": 28, "found": 27, "found_ci95": [0.823, 0.994],
"median_completeness": 0.894, "median_start_delay": 0, "median_end_early": 0, "mean_fragments": 1, "median_effective": -45.9},
{"motion": "pass", "amplitude": -15.0, "sigma": 5.0, "n": 29, "found": 0, "found_ci95": [0.0, 0.117],
"median_completeness": 0.0, "median_start_delay": null, "median_end_early": null, "mean_fragments": null, "median_effective": -12.0}],
"not_visible": 17, "false_tracks": 0, "reference_tracks": 14, "injected_tracks": 163,
"bats": [{"id": 1, "motion": "pass", "amplitude": -15.0, "sigma": 1.5, "visible_frames": 19, "found": false,
"completeness": 0.0, "start_delay": null, "end_early": null, "fragments": 0, "median_effective": -12.011563133785629},
{"id": 8, "motion": "pass", "amplitude": -60.0, "sigma": 1.5, "visible_frames": 12, "found": true,
"completeness": 0.75, "start_delay": 2, "end_early": 1, "fragments": 1, "median_effective": -45.5}]}"#;

fn measures(x: f64) -> Measures {
    Measures {
        x,
        y: 497.022_727_272_727_25,
        left: x - 2.5,
        top: 495.0,
        width: 6.0,
        height: 6.0,
        area: 24.75,
        amplitude: -21.334_605_477_072_97,
    }
}

#[test]
fn the_detections_of_a_run_go_through_a_compressed_file_frame_by_frame_unchanged() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("injected.jsonl.gz");
    let mut detections = FrameDetections::default();
    detections.push([measures(10.0), measures(300.5)]);
    detections.push([]);
    detections.push([measures(12.0)]);

    write_detections(&path, &detections).unwrap();

    assert_eq!(read_detections(&path).unwrap(), detections);
}

#[test]
fn detections_listed_out_of_frame_order_are_refused_with_the_line() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("injected.jsonl");
    fs::write(
        &path,
        "{\"frame\":0,\"detections\":[]}\n{\"frame\":2,\"detections\":[]}\n",
    )
    .unwrap();

    let refused = read_detections(&path).unwrap_err();

    assert!(
        matches!(
            refused,
            Error::FrameOrder {
                line: 2,
                expected: 1,
                found: 2,
                ..
            }
        ),
        "{refused:?}"
    );
}

#[test]
fn a_report_of_the_v011_reads_its_classes_targets_and_totals_whole_numbers_included() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("bench.json");
    fs::write(&path, RECORDED_REPORT).unwrap();

    let report = read_report(&path).unwrap();

    assert_eq!(report.code, "v0.1.1");
    assert_eq!(report.matching["radius"], 24.0);
    assert_eq!(
        (
            report.not_visible,
            report.false_tracks,
            report.reference_tracks,
            report.injected_tracks
        ),
        (17, 0, 14, 163)
    );
    assert_eq!(report.summary[0].median_start_delay, Some(0.0));
    assert_eq!(report.summary[0].mean_fragments, Some(1.0));
    assert_eq!(report.summary[1].mean_fragments, None);
    assert_eq!(report.summary[1].found_ci95, [0.0, 0.117]);
    assert_eq!(report.bats[1].start_delay, Some(2));
    assert_eq!(report.bats[0].end_early, None);
    assert!(report.added.is_empty());
}

#[test]
fn a_report_with_keys_the_v011_does_not_write_keeps_them_through_writing_and_reading() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("bench.json");
    fs::write(&path, RECORDED_REPORT).unwrap();
    let mut report = read_report(&path).unwrap();
    report
        .added
        .insert("seconds".to_owned(), serde_json::json!({"total": 12.5}));

    write_report(&path, &report).unwrap();

    assert_eq!(read_report(&path).unwrap(), report);
    assert!(fs::read_to_string(&path).unwrap().ends_with("}\n"));
}
