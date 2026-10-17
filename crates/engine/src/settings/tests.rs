use std::fs;

use reference::location::{Reference, committed};
use reference::recording::SUMMARY_FILE;
use reference::video::{Decoder, DecoderThreads, Losses, VideoSummary};

use super::{DetectSettings, Region, RegionShape, Setting};

fn extract_summary() -> String {
    fs::read_to_string(committed(Reference::V011, "video_092_original_3m24-4m05").join(SUMMARY_FILE)).unwrap()
}

fn read(text: &str) -> Result<DetectSettings, reference::error::Error> {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join(SUMMARY_FILE);
    fs::write(&path, text).unwrap();
    VideoSummary::read(&path).unwrap().detect_settings()
}

#[test]
fn the_default_settings_are_those_the_v011_recorded() {
    assert_eq!(read(&extract_summary()).unwrap(), DetectSettings::default());
}

#[test]
fn settings_written_into_a_summary_of_the_decoder_read_back_unchanged() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join(SUMMARY_FILE);
    let tuned = DetectSettings {
        threshold: 25.0,
        ..DetectSettings::default()
    };
    let probed: VideoSummary = serde_json::from_str(&extract_summary()).unwrap();
    let mut summary = probed.decoded(
        "v0.2.0",
        Decoder {
            error_frames: Vec::new(),
            gap_frames: Vec::new(),
            losses: Losses {
                before_first: 0,
                after_last: 0,
            },
            threads: DecoderThreads::Single,
        },
    );
    summary.recorded["config"]["DetectConfig"] = serde_json::to_value(&tuned).unwrap();
    summary.write(&path).unwrap();

    assert_eq!(
        VideoSummary::read(&path)
            .unwrap()
            .detect_settings::<DetectSettings>()
            .unwrap(),
        tuned
    );
}

#[test]
fn osd_regions_read_by_their_four_named_bounds() {
    let text = extract_summary().replace(
        "\"osd_regions\": []",
        "\"osd_regions\": [{\"x0\": 0, \"y0\": 0, \"x1\": 1, \"y1\": 0.07}]",
    );
    assert_eq!(
        read(&text).unwrap().osd_regions,
        [Region {
            x0: 0.0,
            y0: 0.0,
            x1: 1.0,
            y1: 0.07
        }]
    );
}

#[test]
fn a_setting_the_v011_does_not_have_is_refused_rather_than_ignored() {
    let text = extract_summary().replace("\"threshold\": 12", "\"treshold\": 12");
    assert!(matches!(read(&text), Err(reference::error::Error::DetectConfig(_))));
}

#[test]
fn a_setting_missing_from_the_recording_is_refused_rather_than_defaulted() {
    let text = extract_summary().replace("\"threshold\": 12,", "");
    assert!(matches!(read(&text), Err(reference::error::Error::DetectConfig(_))));
}

#[test]
fn settings_are_named_as_in_the_v011() {
    let names = [
        Setting::Threshold,
        Setting::MinArea,
        Setting::MaxArea,
        Setting::BgWindowS,
        Setting::BgStep,
        Setting::MergeRadius,
        Setting::WorkWidth,
        Setting::NoiseFactor,
        Setting::TargetSigma,
    ]
    .map(|setting| setting.to_string());
    assert_eq!(
        names,
        [
            "threshold",
            "min_area",
            "max_area",
            "bg_window_s",
            "bg_step",
            "merge_radius",
            "work_width",
            "noise_factor",
            "target_sigma"
        ]
    );
}

#[test]
fn a_region_contains_the_points_on_its_four_edges_and_none_beyond() {
    let region = Region {
        x0: 0.25,
        y0: 0.5,
        x1: 0.75,
        y1: 1.0,
    };

    for (x, y) in [(0.25, 0.7), (0.75, 0.7), (0.5, 0.5), (0.5, 1.0), (0.5, 0.75)] {
        assert!(region.contains(x, y), "({x}, {y})");
    }
    for (x, y) in [(0.249, 0.7), (0.751, 0.7), (0.5, 0.499), (0.5, 1.001)] {
        assert!(!region.contains(x, y), "({x}, {y})");
    }
}

#[test]
fn a_display_region_reads_from_four_fractions_and_writes_back_the_same() {
    let region: Region = " 0, 0.9 ,1,1".parse().unwrap();
    assert_eq!(
        region,
        Region {
            x0: 0.0,
            y0: 0.9,
            x1: 1.0,
            y1: 1.0
        }
    );
    assert_eq!(region.to_string(), "0,0.9,1,1");
    assert_eq!(region.to_string().parse::<Region>(), Ok(region));
}

#[test]
fn a_display_region_that_is_not_four_numbers_is_refused() {
    for text in ["0,0,1", "a,b,c,d", "0,0,1,1,1", ""] {
        assert_eq!(text.parse::<Region>(), Err(RegionShape), "{text}");
    }
    assert_eq!(RegionShape.to_string(), "expected four fractions X0,Y0,X1,Y1");
}

#[test]
fn each_setting_names_the_long_option_of_both_commands_without_its_dashes() {
    let flags = [
        Setting::Threshold,
        Setting::MinArea,
        Setting::MaxArea,
        Setting::BgWindowS,
        Setting::BgStep,
        Setting::MergeRadius,
        Setting::WorkWidth,
        Setting::NoiseFactor,
        Setting::TargetSigma,
    ]
    .map(Setting::flag);
    assert_eq!(
        flags,
        [
            "threshold",
            "min-area",
            "max-area",
            "bg-window",
            "bg-step",
            "merge-radius",
            "work-width",
            "noise-factor",
            "target-sigma",
        ]
    );
}
