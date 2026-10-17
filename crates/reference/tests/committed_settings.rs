use reference::location::{Reference, committed, committed_settings};
use reference::recording::Recording;

const EXTRACT: &str = "video_092_original_3m24-4m05";
const WHOLE_AT_1440: &str = "video_092_original_width_1440";

fn name(recording: &Recording) -> &str {
    recording.summary.video.name.as_str()
}

#[test]
fn the_sixteen_setting_cases_read_as_whole_recordings_of_the_extract_or_its_reduction() {
    let cases = committed_settings(Reference::V011).unwrap();
    let names: Vec<_> = cases
        .iter()
        .map(|case| case.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        names,
        [
            "max_area_400",
            "merge_radius_0",
            "min_area_9",
            "noise_factor_0",
            "osd",
            "quick",
            "sigma_0",
            "sigma_3",
            "size_720x540",
            "step_1",
            "threshold_25",
            "width_1440",
            "width_1440_osd",
            "width_480",
            "width_720",
            "window_2s",
        ]
    );
    for case in &cases {
        let recording = Recording::read(case).unwrap();
        assert_eq!(recording.frames.detections().frame_count(), 1230, "{}", case.display());
        assert!(
            recording.summary.files.frames.ends_with(".jsonl.gz"),
            "{}",
            case.display()
        );
        assert!(
            [format!("{EXTRACT}.mp4"), format!("{EXTRACT}_720x540.mp4")]
                .iter()
                .any(|video| video == name(&recording)),
            "{}",
            case.display()
        );
    }
}

#[test]
fn only_the_cases_on_the_extract_record_other_detection_settings_than_the_defaults() {
    let defaults = Recording::read(&committed(Reference::V011, EXTRACT)).unwrap();
    let default_settings = &defaults.summary.recorded["config"]["DetectConfig"];
    for case in committed_settings(Reference::V011).unwrap() {
        let recording = Recording::read(&case).unwrap();
        let settings = &recording.summary.recorded["config"]["DetectConfig"];
        let on_the_extract = name(&recording) == name(&defaults);
        assert_eq!(settings != default_settings, on_the_extract, "{}", case.display());
    }
}

#[test]
fn the_whole_092_worked_at_1440_pixels_reads_as_a_whole_recording_of_the_092() {
    let recording = Recording::read(&committed(Reference::V011, WHOLE_AT_1440)).unwrap();

    assert_eq!(name(&recording), "video_092_original.mp4");
    assert_eq!(recording.frames.detections().frame_count(), 9011);
    assert!(recording.summary.files.frames.ends_with(".jsonl.gz"));
    assert_eq!(recording.summary.recorded["config"]["DetectConfig"]["work_width"], 1440);
}
