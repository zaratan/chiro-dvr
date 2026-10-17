use std::fs;

use types::detection::Detection;
use types::frame::{FrameIndex, Pts};
use types::language::Language;
use types::rate::FrameRate;
use types::rational::Rational;
use types::timeline::Timeline;
use types::track::Track;

use super::{COLUMNS, Column, Headings, row, write_tracks_csv};
use crate::dialect::Dialect;
use crate::error::Error;

const FRENCH_HEADER: &str = "\u{feff}piste;début;fin;durée (s);points;corde (px);chemin (px);vitesse (px/s);surface max (px);amplitude (niveaux de gris);images comblées;début (s)\r\n";
const ENGLISH_HEADER: &str = "track,start,end,duration (s),points,chord (px),path (px),speed (px/s),max area (px),amplitude (grey levels),filled frames,start (s)\r\n";

#[derive(Debug)]
struct Spelled;

impl Headings for Spelled {
    fn heading(&self, column: Column, language: Language) -> String {
        let [french, english] = match column {
            Column::Track => ["piste", "track"],
            Column::Start => ["début", "start"],
            Column::End => ["fin", "end"],
            Column::Duration => ["durée (s)", "duration (s)"],
            Column::Hits => ["points", "points"],
            Column::Chord => ["corde (px)", "chord (px)"],
            Column::Path => ["chemin (px)", "path (px)"],
            Column::Speed => ["vitesse (px/s)", "speed (px/s)"],
            Column::MaxArea => ["surface max (px)", "max area (px)"],
            Column::Amplitude => ["amplitude (niveaux de gris)", "amplitude (grey levels)"],
            Column::FilledFrames => ["images comblées", "filled frames"],
            Column::StartSeconds => ["début (s)", "start (s)"],
        };
        match language {
            Language::French => french,
            Language::English => english,
        }
        .to_owned()
    }
}

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

fn english() -> Dialect {
    Dialect::of(Language::English)
}

fn written(language: Language, tracks: &[Track]) -> (tempfile::TempDir, std::path::PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("v_pistes.csv");
    write_tracks_csv(&path, tracks, &thirty_per_second(), language, &Spelled).unwrap();
    (directory, path)
}

#[test]
fn a_track_row_reports_times_hits_distances_filled_frames_and_the_start_in_seconds() {
    assert_eq!(
        row(&three_points(), &thirty_per_second(), english()),
        "3,0:00:01,0:00:01,0.1,3,100.0,100.0,1000.0,50,80.0,1,1.0"
    );
}

#[test]
fn a_french_row_takes_semicolons_and_decimal_commas() {
    assert_eq!(
        row(&three_points(), &thirty_per_second(), Dialect::of(Language::French)),
        "3;0:00:01;0:00:01;0,1;3;100,0;100,0;1000,0;50;80,0;1;1,0"
    );
}

#[test]
fn a_single_frame_track_lasts_one_frame() {
    let single = Track::new(1, vec![point(60, 5.0, 5.0, 9.0)]).unwrap();
    assert_eq!(
        row(&single, &thirty_per_second(), english()),
        "1,0:00:02,0:00:02,0.03,1,0.0,0.0,0.0,9,80.0,0,2.0"
    );
}

#[test]
fn the_french_table_starts_with_a_byte_order_mark_and_its_headings_and_ends_its_lines_for_windows() {
    let (_directory, path) = written(Language::French, &[three_points()]);
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        format!("{FRENCH_HEADER}3;0:00:01;0:00:01;0,1;3;100,0;100,0;1000,0;50;80,0;1;1,0\r\n")
    );
}

#[test]
fn the_english_table_has_no_byte_order_mark_and_keeps_commas_and_points() {
    let (_directory, path) = written(Language::English, &[three_points()]);
    let text = fs::read_to_string(&path).unwrap();
    assert_eq!(
        text,
        format!("{ENGLISH_HEADER}3,0:00:01,0:00:01,0.1,3,100.0,100.0,1000.0,50,80.0,1,1.0\r\n")
    );
    assert!(!text.contains('"'));
}

#[test]
fn a_table_without_tracks_is_its_header_alone() {
    let (_directory, path) = written(Language::English, &[]);
    assert_eq!(fs::read_to_string(&path).unwrap(), ENGLISH_HEADER);
}

#[test]
fn a_table_is_never_written_over_a_file_already_there() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("v_tracks.csv");
    fs::write(&path, "id;verdict\r\n1;oui\r\n").unwrap();
    let error = write_tracks_csv(
        &path,
        &[three_points()],
        &thirty_per_second(),
        Language::English,
        &Spelled,
    )
    .unwrap_err();
    assert!(matches!(error, Error::Taken { .. }), "{error}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "id;verdict\r\n1;oui\r\n");
}

#[test]
fn a_table_is_named_only_once_whole_and_leaves_no_partial_file() {
    let (directory, path) = written(Language::French, &[three_points()]);
    let names: Vec<_> = fs::read_dir(directory.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(names, ["v_pistes.csv"]);
    assert!(fs::read_to_string(&path).unwrap().starts_with('\u{feff}'));
}

#[test]
fn a_table_in_a_missing_folder_is_reported() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("absent").join("v_tracks.csv");
    let error = write_tracks_csv(&path, &[], &thirty_per_second(), Language::English, &Spelled).unwrap_err();
    assert!(matches!(error, Error::Write { .. }));
    assert!(error.to_string().contains("v_tracks.csv"));
}

#[test]
fn an_area_ending_in_a_half_rounds_up() {
    let half = Track::new(1, vec![point(0, 0.0, 0.0, 274.5)]).unwrap();
    let area = COLUMNS.iter().position(|&column| column == Column::MaxArea).unwrap();
    assert_eq!(
        row(&half, &thirty_per_second(), english()).split(',').nth(area),
        Some("275")
    );
}

#[test]
fn a_track_after_a_gap_in_the_video_starts_at_the_time_of_its_pts() {
    let rate = FrameRate::new(Rational::new(30, 1).unwrap()).unwrap();
    let pts = [0, 1, 16, 17].map(|ticks| Some(Pts::new(ticks))).to_vec();
    let gapped = Timeline::new(pts, Rational::new(1, 30).unwrap(), rate).unwrap();
    let after = Track::new(1, vec![point(2, 0.0, 0.0, 9.0), point(3, 3.0, 4.0, 9.0)]).unwrap();
    let written = row(&after, &gapped, english());
    assert!(written.starts_with("1,0:00:00,0:00:00,0.03,"), "{written}");
    assert!(written.ends_with(",0.53"), "{written}");
}
