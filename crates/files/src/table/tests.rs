use std::fs;

use types::detection::Detection;
use types::frame::{FrameIndex, Pts};
use types::rate::FrameRate;
use types::rational::Rational;
use types::timeline::Timeline;
use types::track::Track;

use super::{decimal, is_tracks_table, row, write_tracks_csv};
use crate::error::Error;

fn thirty_per_second() -> Timeline {
    let rate = FrameRate::new(Rational::new(30, 1).unwrap()).unwrap();
    let pts = (0..300).map(|frame| Some(Pts::new(frame))).collect();
    Timeline::new(pts, Rational::new(1, 30).unwrap(), rate).unwrap()
}

fn point(frame: usize, x: f64, y: f64, area: f64) -> Detection {
    Detection {
        frame: FrameIndex::new(frame),
        x,
        y,
        left: x - 1.0,
        top: y - 1.0,
        width: 3.0,
        height: 3.0,
        area,
        amplitude: 80.0,
    }
}

fn three_points() -> Track {
    Track::new(
        3,
        vec![
            point(30, 0.0, 0.0, 20.0),
            point(31, 30.0, 40.0, 50.0),
            point(33, 60.0, 80.0, 30.0),
        ],
    )
    .unwrap()
}

#[test]
fn a_track_row_reports_times_hits_distances_and_filled_frames() {
    assert_eq!(
        row(&three_points(), &thirty_per_second()),
        "3,0:01.00,0:01.10,1.0,0.1,3,100.0,100.0,1000.0,50,80.0,1"
    );
}

#[test]
fn a_single_frame_track_lasts_one_frame() {
    let single = Track::new(1, vec![point(60, 5.0, 5.0, 9.0)]).unwrap();
    assert_eq!(
        row(&single, &thirty_per_second()),
        "1,0:02.00,0:02.00,2.0,0.03,1,0.0,0.0,0.0,9,80.0,0"
    );
}

#[test]
fn the_table_has_an_english_header_and_windows_line_ends() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("v_pistes.csv");
    write_tracks_csv(&path, &[three_points()], &thirty_per_second()).unwrap();
    let text = fs::read_to_string(&path).unwrap();
    assert_eq!(
        text,
        "id,start,end,start_s,duration_s,hits,chord_px,path_px,speed_px_s,max_area_px,peak_amplitude,filled_frames\r\n\
         3,0:01.00,0:01.10,1.0,0.1,3,100.0,100.0,1000.0,50,80.0,1\r\n"
    );
    assert!(!text.contains('"'));
}

#[test]
fn a_table_without_tracks_is_its_header_alone() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("v_tracks.csv");
    write_tracks_csv(&path, &[], &thirty_per_second()).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap().lines().count(), 1);
}

#[test]
fn a_table_we_write_with_or_without_tracks_is_recognised_as_ours() {
    let directory = tempfile::tempdir().unwrap();
    for (name, tracks) in [("empty.csv", vec![]), ("three.csv", vec![three_points()])] {
        let path = directory.path().join(name);
        write_tracks_csv(&path, &tracks, &thirty_per_second()).unwrap();
        assert!(is_tracks_table(&path).unwrap());
    }
}

#[test]
fn a_table_is_never_written_over_a_file_already_there() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("v_tracks.csv");
    fs::write(&path, "id;verdict\r\n1;oui\r\n").unwrap();
    let error = write_tracks_csv(&path, &[three_points()], &thirty_per_second()).unwrap_err();
    assert!(matches!(error, Error::Taken { .. }), "{error}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "id;verdict\r\n1;oui\r\n");
}

#[test]
fn a_table_is_named_only_once_whole_and_leaves_no_partial_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("v_tracks.csv");
    write_tracks_csv(&path, &[three_points()], &thirty_per_second()).unwrap();
    let names: Vec<_> = fs::read_dir(directory.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(names, ["v_tracks.csv"]);
    assert!(is_tracks_table(&path).unwrap());
}

#[test]
fn a_table_in_a_missing_folder_is_reported() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("absent").join("v_tracks.csv");
    let error = write_tracks_csv(&path, &[], &thirty_per_second()).unwrap_err();
    assert!(matches!(error, Error::Write { .. }));
    assert!(error.to_string().contains("v_tracks.csv"));
}

#[test]
fn a_whole_number_keeps_its_decimal_point() {
    assert_eq!(decimal(12.0, 1), "12.0");
    assert_eq!(decimal(7.3, 2), "7.3");
    assert_eq!(decimal(944.149, 1), "944.1");
    assert_eq!(decimal(0.000_4, 2), "0.0");
}

#[test]
fn a_negative_zero_is_written_as_zero() {
    assert_eq!(decimal(-0.0, 1), "0.0");
    assert_eq!(decimal(-0.01, 1), "0.0");
}

#[test]
fn an_area_ending_in_a_half_rounds_up() {
    let half = Track::new(1, vec![point(0, 0.0, 0.0, 274.5)]).unwrap();
    assert_eq!(row(&half, &thirty_per_second()).split(',').nth(9), Some("275"));
}

#[test]
fn a_track_after_a_gap_in_the_video_starts_at_the_time_of_its_pts() {
    let rate = FrameRate::new(Rational::new(30, 1).unwrap()).unwrap();
    let pts = [0, 1, 16, 17].map(|ticks| Some(Pts::new(ticks))).to_vec();
    let gapped = Timeline::new(pts, Rational::new(1, 30).unwrap(), rate).unwrap();
    let after = Track::new(1, vec![point(2, 0.0, 0.0, 9.0), point(3, 3.0, 4.0, 9.0)]).unwrap();
    assert!(row(&after, &gapped).starts_with("1,0:00.53,0:00.57,0.53,0.03,"));
}
