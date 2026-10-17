use std::path::{Path, PathBuf};

use super::{
    ANNOTATED_PER_VIDEO_BYTE, GIGABYTE, Need, OUTPUTS_PER_VIDEO_BYTE, Rendering, Short, Volume, launch_estimate,
    render_estimate, short_volumes, with_margin,
};

fn need(folder: &str, bytes: u64) -> Need {
    Need {
        folder: PathBuf::from(folder),
        bytes,
    }
}

#[test]
fn a_video_is_expected_to_write_its_size_times_the_measured_share_and_more_with_its_annotated_video() {
    let video = 400_000_000;
    let plain = launch_estimate(video, false);
    let annotated = launch_estimate(video, true);
    assert_eq!(plain, (400_000_000.0 * OUTPUTS_PER_VIDEO_BYTE).ceil() as u64);
    let added = (400_000_000.0 * ANNOTATED_PER_VIDEO_BYTE) as u64;
    assert!((annotated - plain).abs_diff(added) <= 1, "{annotated} {plain}");
    assert_eq!(launch_estimate(0, true), 0);
}

#[test]
fn the_margin_is_a_tenth_of_the_estimate_but_never_less_than_a_gigabyte() {
    assert_eq!(with_margin(0), GIGABYTE);
    assert_eq!(with_margin(3 * GIGABYTE), 4 * GIGABYTE);
    assert_eq!(with_margin(20 * GIGABYTE), 22 * GIGABYTE);
    assert_eq!(with_margin(u64::MAX), u64::MAX);
}

#[test]
fn the_needs_of_one_volume_add_up_and_name_its_first_results_folder() {
    let volumes = |folder: &Path| {
        Some(if folder.starts_with("/Volumes/BATS") {
            Volume {
                device: 2,
                free: 4_900_000_000,
            }
        } else {
            Volume {
                device: 1,
                free: 500 * GIGABYTE,
            }
        })
    };
    let needs = [
        need("/Volumes/BATS/nuit-12-oct/batdetect", 2 * GIGABYTE),
        need("/Users/manon/Desktop/batdetect", 2 * GIGABYTE),
        need("/Volumes/BATS/nuit-13-oct/batdetect", 2 * GIGABYTE),
    ];
    assert_eq!(
        short_volumes(&needs, volumes),
        [Short {
            folder: PathBuf::from("/Volumes/BATS/nuit-12-oct/batdetect"),
            needed: 5 * GIGABYTE,
            free: 4_900_000_000,
        }]
    );
}

#[test]
fn a_volume_with_room_for_the_estimate_and_its_margin_is_not_short() {
    let room = |free| move |_: &Path| Some(Volume { device: 1, free });
    let needs = [need("nuit/batdetect", 2 * GIGABYTE)];
    assert_eq!(short_volumes(&needs, room(3 * GIGABYTE)), []);
    assert_eq!(short_volumes(&needs, room(3 * GIGABYTE - 1)).len(), 1);
}

#[test]
fn a_volume_whose_free_space_cannot_be_read_is_not_refused() {
    assert_eq!(short_volumes(&[need("nuit/batdetect", u64::MAX)], |_| None), []);
}

#[test]
fn the_clips_of_a_video_are_expected_in_proportion_to_their_seconds_and_the_bitrate_of_the_video() {
    let rendering = |clip_s, zoom_s, annotated| Rendering {
        clip_s,
        zoom_s,
        annotated,
    };
    assert_eq!(render_estimate(300_000_000, 300.0, rendering(0.0, 0.0, false)), 0);
    let ten = render_estimate(300_000_000, 300.0, rendering(10.0, 0.0, false));
    let twenty = render_estimate(300_000_000, 300.0, rendering(20.0, 0.0, false));
    assert!(ten > 10_000_000 && twenty.abs_diff(2 * ten) <= 1, "{ten} {twenty}");
    assert!(render_estimate(300_000_000, 300.0, rendering(10.0, 10.0, false)) > ten);
    assert_eq!(
        render_estimate(300_000_000, 300.0, rendering(0.0, 0.0, true)),
        (300_000_000.0 * ANNOTATED_PER_VIDEO_BYTE).ceil() as u64
    );
}

#[test]
fn a_video_without_a_length_counts_only_its_annotated_video_rather_than_an_invented_bitrate() {
    let rendering = Rendering {
        clip_s: 10.0,
        zoom_s: 10.0,
        annotated: true,
    };
    let annotated = (300_000_000.0 * ANNOTATED_PER_VIDEO_BYTE).ceil() as u64;
    assert_eq!(render_estimate(300_000_000, 0.0, rendering), annotated);
    assert_eq!(render_estimate(300_000_000, f64::NAN, rendering), annotated);
}
