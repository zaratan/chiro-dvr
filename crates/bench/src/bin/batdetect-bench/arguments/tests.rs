use std::num::{NonZeroU32, NonZeroUsize};
use std::path::PathBuf;

use bench::config::MatchConfig;
use bench::metrics::Judging;
use bench::synthetic::{Class, Motion};
use clap::error::ErrorKind;
use engine::config::DetectConfig;
use engine::settings::{DetectSettings, Region};
use tracking::stability::StabilityConfig;
use tracking::track::{Reach, RejoinConfig, TrackConfig};

use super::{Request, Source, command, parse};

fn refused(arguments: &[&str]) -> clap::Error {
    parse(std::iter::once("batdetect-bench").chain(arguments.iter().copied())).unwrap_err()
}

fn parsed(arguments: &[&str]) -> Request {
    parse(std::iter::once("batdetect-bench").chain(arguments.iter().copied())).unwrap()
}

#[test]
fn a_video_alone_draws_thirty_passing_targets_of_nine_classes_with_the_default_settings() {
    let request = parsed(&["video_092.mp4"]);

    let Source::Drawn { video, sampling } = request.source else {
        panic!("{:?}", request.source);
    };
    assert_eq!(video, PathBuf::from("video_092.mp4"));
    assert_eq!(
        (sampling.seed, sampling.per_class.get(), sampling.min_distance),
        (1, 30, 72.0)
    );
    assert_eq!(sampling.classes.len(), 9);
    assert_eq!(sampling.classes[0], Class::new(-15.0, 1.5, Motion::Pass).unwrap());
    assert_eq!(sampling.classes[8], Class::new(-60.0, 5.0, Motion::Pass).unwrap());
    assert_eq!(request.out_dir, PathBuf::from("out/bench"));
    assert_eq!(DetectSettings::from(&request.detect), DetectSettings::default());
    assert_eq!(request.settings, Judging::default());
}

#[test]
fn every_bench_option_reaches_the_drawing() {
    let request = parsed(&[
        "video.mp4",
        "-o",
        "bancs",
        "--per-class",
        "60",
        "--seed",
        "4",
        "--match-radius",
        "12",
        "--motions",
        "circle",
        "hunt",
        "--sigmas",
        "3",
        "--amplitudes",
        "-60",
        "-30",
    ]);

    let Source::Drawn { sampling, .. } = request.source else {
        panic!("{:?}", request.source);
    };
    let classes: Vec<_> = sampling
        .classes
        .iter()
        .map(|class| (class.motion(), class.amplitude(), class.sigma()))
        .collect();
    assert_eq!(
        classes,
        [
            (Motion::Hunt, -60.0, 3.0),
            (Motion::Hunt, -30.0, 3.0),
            (Motion::Circle, -60.0, 3.0),
            (Motion::Circle, -30.0, 3.0)
        ]
    );
    assert_eq!(
        (sampling.seed, sampling.per_class.get(), sampling.min_distance),
        (4, 60, 36.0)
    );
    assert_eq!(request.settings.matching, MatchConfig::new(12.0, 0.5).unwrap());
    assert_eq!(request.out_dir, PathBuf::from("bancs"));
}

#[test]
fn a_video_given_after_negative_amplitudes_is_read_as_one_more_amplitude_and_refused() {
    let error = refused(&["--amplitudes", "-60", "-30", "video.mp4"]);

    assert_eq!(error.kind(), ErrorKind::ValueValidation);
}

#[test]
fn every_detection_and_tracking_option_reaches_its_setting() {
    let request = parsed(&[
        "video.mp4",
        "--threshold",
        "10",
        "--min-area",
        "2",
        "--max-area",
        "900",
        "--bg-window",
        "2",
        "--bg-step",
        "2",
        "--osd-region",
        "0,0,1,0.07",
        "--osd-region",
        "0.8,0.9,1,1",
        "--merge-radius",
        "4",
        "--work-width",
        "1440",
        "--noise-factor",
        "0",
        "--target-sigma",
        "0",
        "--max-blobs",
        "0",
        "--unstable-pad",
        "0.5",
        "--max-jump",
        "100",
        "--max-gap",
        "4",
        "--min-hits",
        "5",
        "--min-travel",
        "30",
        "--twin-distance",
        "0",
        "--max-median-turn",
        "3.15",
        "--rejoin-hidden",
        "0",
    ]);

    let expected = DetectSettings {
        threshold: 10.0,
        min_area: 2.0,
        max_area: 900.0,
        bg_window_s: 2.0,
        bg_step: 2,
        osd_regions: vec![
            Region {
                x0: 0.0,
                y0: 0.0,
                x1: 1.0,
                y1: 0.07,
            },
            Region {
                x0: 0.8,
                y0: 0.9,
                x1: 1.0,
                y1: 1.0,
            },
        ],
        merge_radius: 4.0,
        work_width: 1440,
        noise_factor: 0.0,
        target_sigma: 0.0,
    };
    assert_eq!(DetectSettings::from(&request.detect), expected);
    assert_eq!(request.settings.stability, StabilityConfig::new(None, 0.5).unwrap());
    let four = NonZeroUsize::new(4).unwrap();
    let five = NonZeroUsize::new(5).unwrap();
    assert_eq!(
        request.settings.track,
        TrackConfig::new(100.0, four, five, 30.0, None, 3.15, None).unwrap()
    );
}

#[test]
fn the_default_stability_and_twin_settings_stay_enabled() {
    let request = parsed(&["video.mp4"]);

    assert_eq!(request.settings.stability.max_blobs(), NonZeroU32::new(20));
    assert_eq!(request.settings.track.twin_distance(), Some(36.0));
    assert_eq!(request.settings.track.rejoin(), Some(RejoinConfig::default()));
    assert_eq!(request.detect, DetectConfig::default());
}

#[test]
fn the_rejoin_is_set_by_its_two_options_and_a_zero_distance_disables_it() {
    let set = parsed(&["video.mp4", "--rejoin-hidden", "12", "--rejoin-distance", "40"]);
    let expected = RejoinConfig::new(NonZeroUsize::new(12).unwrap(), 40.0).unwrap();
    assert_eq!(set.settings.track.rejoin(), Some(expected));
    assert_eq!(
        parsed(&["video.mp4", "--rejoin-distance", "0"]).settings.track.rejoin(),
        None
    );
}

#[test]
fn the_reach_is_set_by_its_two_options_and_a_zero_margin_disables_it() {
    let set = parsed(&["video.mp4", "--reach-margin", "40", "--reach-travel", "2"]);
    assert_eq!(set.settings.track.reach(), Some(Reach::new(40.0, 2.0).unwrap()));
    assert_eq!(parsed(&["video.mp4"]).settings.track.reach(), Some(Reach::default()));
    assert_eq!(
        parsed(&["video.mp4", "--reach-margin", "0"]).settings.track.reach(),
        None
    );
}

#[test]
fn replayed_targets_and_a_recorded_bench_name_their_file_or_folder() {
    assert_eq!(
        parsed(&["video.mp4", "--targets", "bats.jsonl.gz"]).source,
        Source::Targets {
            video: PathBuf::from("video.mp4"),
            file: PathBuf::from("bats.jsonl.gz")
        }
    );
    assert_eq!(
        parsed(&["video.mp4", "--against", "transit"]).source,
        Source::Against {
            video: PathBuf::from("video.mp4"),
            folder: PathBuf::from("transit")
        }
    );
}

#[test]
fn a_replay_needs_no_video_and_takes_other_tracking_settings() {
    let request = parsed(&[
        "--replay",
        "out/bench/video_092",
        "--min-hits",
        "4",
        "--match-radius",
        "30",
    ]);

    assert_eq!(
        request.source,
        Source::Replay {
            folder: PathBuf::from("out/bench/video_092")
        }
    );
    assert_eq!(request.settings.track.min_hits().get(), 4);
    assert_eq!(request.settings.matching.radius(), 30.0);
}

#[test]
fn options_that_would_contradict_the_replayed_or_recorded_targets_are_refused() {
    for arguments in [
        &["video.mp4", "--targets", "bats.jsonl.gz", "--seed", "2"][..],
        &["video.mp4", "--targets", "bats.jsonl.gz", "--against", "transit"],
        &["video.mp4", "--against", "transit", "--per-class", "10"],
        &["video.mp4", "--replay", "run"],
        &["--replay", "run", "--threshold", "10"],
        &["--replay", "run", "--osd-region", "0,0,1,0.1"],
        &["--replay", "run", "--motions", "hunt"],
        &["--replay", "run", "--targets", "bats.jsonl.gz"],
    ] {
        assert_eq!(refused(arguments).kind(), ErrorKind::ArgumentConflict, "{arguments:?}");
    }
}

#[test]
fn a_setting_out_of_bounds_is_refused_with_the_option_that_holds_it() {
    for (arguments, message) in [
        (
            &["v.mp4", "--threshold", "-1"][..],
            "--threshold must be a finite number > 0, not -1",
        ),
        (
            &["v.mp4", "--bg-window", "0"],
            "--bg-window must be a finite number > 0, not 0",
        ),
        (
            &["v.mp4", "--min-area", "50", "--max-area", "40"],
            "--min-area 50 is larger than --max-area 40",
        ),
        (&["v.mp4", "--osd-region", "0,0,0,1"], "--osd-region 1 does not satisfy"),
        (&["v.mp4", "--max-jump", "0"], "--max-jump is out of bounds: 0"),
        (
            &["v.mp4", "--rejoin-distance", "-1"],
            "--rejoin-distance is out of bounds: -1",
        ),
        (&["v.mp4", "--max-gap", "19"], "--rejoin-hidden is out of bounds: 19"),
        (
            &["v.mp4", "--reach-margin", "-1"],
            "--reach-margin is out of bounds: -1",
        ),
        (
            &["v.mp4", "--reach-travel", "-1"],
            "--reach-travel is out of bounds: -1",
        ),
        (
            &["v.mp4", "--unstable-pad", "-1"],
            "--unstable-pad is out of bounds: -1",
        ),
        (
            &["v.mp4", "--match-radius", "0"],
            "--match-radius must be a finite number > 0, not 0",
        ),
        (
            &["v.mp4", "--sigmas", "0"],
            "--sigmas must be finite numbers > 0, not 0",
        ),
        (
            &["v.mp4", "--amplitudes", "inf"],
            "--amplitudes must be finite numbers, not inf",
        ),
    ] {
        let error = refused(arguments);

        assert_eq!(error.kind(), ErrorKind::ValueValidation, "{arguments:?}");
        assert!(error.to_string().contains(message), "{arguments:?}: {error}");
    }
}

#[test]
fn a_display_region_that_is_not_four_numbers_is_refused() {
    for region in ["0,0,1", "0,0,1,a", ""] {
        let error = refused(&["v.mp4", "--osd-region", region]);

        assert!(
            error.to_string().contains("expected four fractions X0,Y0,X1,Y1"),
            "{region}: {error}"
        );
    }
}

#[test]
fn without_any_argument_the_help_is_shown() {
    assert_eq!(refused(&[]).kind(), ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand);
}

#[test]
fn the_help_lists_every_option_with_its_default() {
    let help = command().render_long_help().to_string();

    for option in [
        "--out-dir <FOLDER>",
        "[default: out/bench]",
        "--per-class",
        "[default: 30]",
        "--amplitudes",
        "[default: -15 -30 -60]",
        "--sigmas",
        "[default: 1.5 3 5]",
        "--motions",
        "[default: pass]",
        "--seed",
        "[default: 1]",
        "--match-radius",
        "[default: 24]",
        "--targets <FILE>",
        "--against <FOLDER>",
        "--replay <FOLDER>",
        "--threshold",
        "[default: 12]",
        "--min-area",
        "[default: 4]",
        "--max-area",
        "[default: 2700]",
        "--bg-window",
        "--bg-step",
        "[default: 3]",
        "--osd-region <X0,Y0,X1,Y1>",
        "--merge-radius",
        "[default: 6]",
        "--work-width",
        "[default: 960]",
        "--noise-factor",
        "[default: 8]",
        "--target-sigma",
        "--max-blobs",
        "[default: 20]",
        "--unstable-pad",
        "--max-jump",
        "[default: 120]",
        "--max-gap",
        "--min-hits",
        "--min-travel",
        "[default: 45]",
        "--twin-distance",
        "[default: 36]",
        "--max-median-turn",
        "[default: 0.8]",
    ] {
        assert!(help.contains(option), "{option}\n{help}");
    }
}
