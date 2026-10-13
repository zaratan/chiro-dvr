use std::ffi::OsString;
use std::num::{NonZeroU32, NonZeroUsize};
use std::path::PathBuf;

use batdetect::options::Options;
use clap::error::ErrorKind;
use engine::settings::Region;
use render::config::Zoom;

use super::{Level, Request, command, level, parse};

fn read(arguments: &[&str]) -> Options {
    let mut all = vec!["batdetect-rs", "a.mp4", "-o", "out"];
    all.extend_from_slice(arguments);
    parse(all).unwrap().options
}

fn refused(arguments: &[&str]) -> clap::Error {
    let mut all = vec!["batdetect-rs", "a.mp4", "-o", "out"];
    all.extend_from_slice(arguments);
    parse(all).unwrap_err()
}

fn help(level: Level) -> String {
    command(level).term_width(100).render_help().to_string()
}

fn level_of(arguments: &[&str]) -> Level {
    let arguments: Vec<OsString> = arguments.iter().map(OsString::from).collect();
    level(&arguments)
}

#[test]
fn several_videos_and_the_long_out_dir_option_are_read_with_the_v0_1_1_settings() {
    let request = parse(["batdetect-rs", "a.mp4", "nuit", "--out-dir", "out"]).unwrap();
    assert_eq!(
        request,
        Request {
            inputs: vec![PathBuf::from("a.mp4"), PathBuf::from("nuit")],
            out_dir: PathBuf::from("out"),
            options: Options::default(),
        }
    );
}

#[test]
fn the_short_out_dir_option_may_come_first_and_the_old_long_name_is_refused() {
    let request = parse(["batdetect-rs", "-o", "out", "a.mp4"]).unwrap();
    assert_eq!(request.out_dir, PathBuf::from("out"));
    let error = parse(["batdetect-rs", "a.mp4", "--output", "out"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::UnknownArgument);
}

#[test]
fn a_missing_out_dir_is_a_usage_error() {
    let error = parse(["batdetect-rs", "a.mp4"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::MissingRequiredArgument);
    assert_eq!(error.exit_code(), 2);
}

#[test]
fn an_out_dir_without_video_is_a_usage_error() {
    let error = parse(["batdetect-rs", "-o", "out"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::MissingRequiredArgument);
}

#[test]
fn the_zoom_choice_is_read_and_any_other_value_is_a_usage_error() {
    for (name, zoom) in [("auto", Zoom::Auto), ("all", Zoom::All), ("none", Zoom::Off)] {
        assert_eq!(read(&["--zoom", name]).render.zoom().choice(), zoom);
    }
    let error = refused(&["--zoom", "some"]);
    assert_eq!(error.kind(), ErrorKind::InvalidValue);
    assert_eq!(error.exit_code(), 2);
}

#[test]
fn each_kept_option_reaches_its_setting() {
    let options = read(&[
        "--annotated",
        "--max-tracks",
        "0",
        "--clip-margin",
        "2.5",
        "--osd-region",
        "0,0,1,0.1",
        "--osd-region",
        "0.8,0.9,1,1",
        "--threshold",
        "10",
        "--min-hits",
        "3",
        "--max-blobs",
        "0",
        "--work-width",
        "720",
    ]);
    assert!(options.annotated);
    assert_eq!((options.render.max_tracks(), options.render.clip_margin_s()), (0, 2.5));
    assert_eq!(
        options.detect.osd_regions(),
        [
            Region {
                x0: 0.0,
                y0: 0.0,
                x1: 1.0,
                y1: 0.1
            },
            Region {
                x0: 0.8,
                y0: 0.9,
                x1: 1.0,
                y1: 1.0
            }
        ]
    );
    assert_eq!(options.detect.threshold(), 10.0);
    assert_eq!(options.detect.work_width(), NonZeroU32::new(720).unwrap());
    assert_eq!(options.track.min_hits(), NonZeroUsize::new(3).unwrap());
    assert_eq!(options.stability.max_blobs(), None);
}

#[test]
fn a_value_a_setting_refuses_is_a_usage_error_that_names_the_option() {
    for (arguments, named) in [
        (&["--threshold", "-1"][..], "--threshold must be"),
        (&["--threshold", "nan"][..], "--threshold must be"),
        (&["--work-width", "8"][..], "--work-width must be"),
        (&["--clip-margin", "-2"][..], "--clip-margin must be"),
        (&["--osd-region", "0.5,0,0.2,1"][..], "--osd-region 1 does not satisfy"),
    ] {
        let error = refused(arguments);
        assert_eq!(error.exit_code(), 2, "{arguments:?}");
        assert!(error.to_string().contains(named), "{arguments:?}: {error}");
    }
}

#[test]
fn a_value_that_is_not_a_number_or_not_allowed_is_a_usage_error_that_names_the_option() {
    for (arguments, named) in [
        (&["--min-hits", "0"][..], "--min-hits"),
        (&["--max-blobs", "-3"][..], "--max-blobs"),
        (&["--max-tracks", "many"][..], "--max-tracks"),
    ] {
        let error = refused(arguments);
        assert_eq!(error.kind(), ErrorKind::ValueValidation, "{arguments:?}");
        assert!(error.to_string().contains(named), "{arguments:?}: {error}");
    }
}

#[test]
fn a_display_region_that_is_not_four_fractions_is_refused() {
    for region in ["0,0,1", "a,b,c,d"] {
        let error = refused(&["--osd-region", region]);
        assert!(
            error.to_string().contains("expected four fractions X0,Y0,X1,Y1"),
            "{region}: {error}"
        );
    }
}

#[test]
fn the_short_help_shows_only_the_essential_options_and_points_to_the_long_help() {
    let short = help(Level::Essential);
    for shown in ["<VIDEO>", "--out-dir <FOLDER>", "--zoom <WHICH>", "--help", "--version"] {
        assert!(short.contains(shown), "{shown}: {short}");
    }
    for hidden in [
        "--annotated",
        "--max-tracks",
        "--threshold",
        "Settings that change the detection",
    ] {
        assert!(!short.contains(hidden), "{hidden}: {short}");
    }
    assert!(short.contains("--help all of them"), "{short}");
}

#[test]
fn each_option_of_either_help_keeps_its_text_on_the_option_line_without_blank_lines_between_options() {
    for flag in [Level::Essential, Level::Full] {
        let shown = help(flag);
        let options = shown.split("Options:").nth(1).unwrap();
        assert!(!options.trim_end().contains("\n\n  "), "{flag:?}: {shown}");
        let out_dir = options
            .lines()
            .find(|line| line.contains("--out-dir <FOLDER>"))
            .unwrap();
        assert!(out_dir.contains("folder that receives"), "{flag:?}: {out_dir}");
    }
    let long = help(Level::Full);
    let threshold = long.lines().find(|line| line.contains("--threshold <LEVEL>")).unwrap();
    assert!(threshold.contains("smallest difference"), "{threshold}");
}

#[test]
fn the_help_takes_the_colours_of_argparse_headings_in_bold_blue_options_in_bold_cyan_values_in_yellow() {
    let coloured = command(Level::Full).render_help().ansi().to_string();
    for styled in [
        "\u{1b}[1m\u{1b}[34mOptions:",
        "\u{1b}[1m\u{1b}[34mUsage:",
        "\u{1b}[1m\u{1b}[36m--out-dir",
        "\u{1b}[33m<FOLDER>",
        "\u{1b}[32mauto\u{1b}[0m, ",
        "[default:\u{a0}\u{1b}[32m300\u{1b}[0m]",
        "[default:\u{a0}\u{1b}[32mauto\u{1b}[0m]",
    ] {
        assert!(coloured.contains(styled), "{styled:?}: {coloured:?}");
    }
    let refused = refused(&["--threshold", "-1"]).render().ansi().to_string();
    assert!(refused.contains("\u{1b}[1m\u{1b}[31merror:"), "{refused:?}");
}

#[test]
fn the_long_help_shows_every_option_with_its_effect_its_default_and_a_telling_value_name() {
    let long = help(Level::Full);
    for shown in [
        "--annotated",
        "--max-tracks <COUNT>",
        "[default:\u{a0}300]",
        "--clip-margin <SECONDS>",
        "[default:\u{a0}1.5]",
        "--osd-region <X0,Y0,X1,Y1>",
        "none by default",
        "Settings that change the detection:",
        "--threshold <LEVEL>",
        "[default:\u{a0}12]",
        "--min-hits <COUNT>",
        "[default:\u{a0}6]",
        "--max-blobs <COUNT>",
        "[default:\u{a0}20]",
        "--work-width <PIXELS>",
        "[default:\u{a0}960]",
        "[default:\u{a0}auto]",
    ] {
        assert!(long.contains(shown), "{shown}: {long}");
    }
    assert!(long.contains("[possible values: auto, all, none]"), "{long}");
    let version = long.lines().find(|line| line.contains("-V, --version")).unwrap();
    assert!(version.ends_with("print the version"), "{version}");
    let heading = long.find("Settings that change the detection").unwrap();
    assert!(long.find("--threshold").unwrap() > heading);
    assert!(long.find("--clip-margin").unwrap() < heading);
}

#[test]
fn the_long_help_is_chosen_by_a_help_flag_before_the_end_of_options_and_both_flags_exit_with_0() {
    assert_eq!(level_of(&["batdetect-rs", "-h"]), Level::Essential);
    assert_eq!(level_of(&["batdetect-rs", "--help"]), Level::Full);
    assert_eq!(level_of(&["batdetect-rs", "-h", "--help"]), Level::Full);
    assert_eq!(level_of(&["batdetect-rs", "a.mp4", "--", "--help"]), Level::Essential);
    for flag in ["-h", "--help"] {
        let shown = parse(["batdetect-rs", flag]).unwrap_err();
        assert_eq!(shown.kind(), ErrorKind::DisplayHelp);
        assert_eq!(shown.exit_code(), 0);
    }
}

#[test]
fn every_default_stands_whole_at_the_start_of_its_own_line() {
    let long = help(Level::Full);
    let defaults: Vec<&str> = long.lines().filter(|line| line.contains("[default:")).collect();
    assert_eq!(defaults.len(), 7, "{long}");
    for line in defaults {
        let tag = line.trim_start();
        assert!(tag.starts_with("[default:\u{a0}") && tag.ends_with(']'), "{line}");
    }
}
