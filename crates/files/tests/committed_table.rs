use std::fs;

use files::table::{COLUMNS, Column, Headings, write_tracks_csv};
use reference::location::{Reference, committed};
use reference::recording::Recording;
use types::language::Language;

const STARTS_FROM_PTS: [(usize, f64, f64); 11] = [
    (2, 33.94, 33.93),
    (5, 72.6, 72.59),
    (6, 108.93, 108.92),
    (7, 122.82, 122.81),
    (8, 144.63, 144.62),
    (9, 156.99, 156.98),
    (10, 163.95, 163.94),
    (11, 209.61, 209.59),
    (12, 228.29, 228.27),
    (13, 230.89, 230.87),
    (14, 237.18, 237.16),
];

const ENDS_ACROSS_A_SECOND: [(usize, &str, &str); 1] = [(6, "0:01:50", "0:01:49")];

const AREAS_ENDING_IN_A_HALF: [usize; 4] = [1, 5, 6, 10];
const FASTER_TRACKS: [usize; 10] = [2, 4, 5, 6, 7, 8, 9, 12, 13, 14];
const SECONDS_PER_MINUTE: u64 = 60;

#[derive(Debug)]
struct Debugged;

impl Headings for Debugged {
    fn heading(&self, column: Column, _language: Language) -> String {
        format!("{column:?}")
    }
}

#[derive(Debug)]
struct Written {
    id: usize,
    start: String,
    end: String,
    duration_s: f64,
    verdict: String,
    hits: usize,
    chord_px: f64,
    path_px: f64,
    speed_px_s: f64,
    max_area_px: u64,
    peak_amplitude: f64,
    filled_frames: usize,
    start_s: f64,
}

#[cfg(test)]
impl Written {
    fn of(line: &str) -> Self {
        let cells: Vec<&str> = line.split(',').collect();
        assert_eq!(cells.len(), COLUMNS.len(), "{line}");
        let cell = |wanted: Column| cells[COLUMNS.iter().position(|&column| column == wanted).unwrap()];
        Self {
            id: cell(Column::Track).parse().unwrap(),
            start: cell(Column::Start).to_owned(),
            end: cell(Column::End).to_owned(),
            duration_s: cell(Column::Duration).parse().unwrap(),
            verdict: cell(Column::Verdict).to_owned(),
            hits: cell(Column::Hits).parse().unwrap(),
            chord_px: cell(Column::Chord).parse().unwrap(),
            path_px: cell(Column::Path).parse().unwrap(),
            speed_px_s: cell(Column::Speed).parse().unwrap(),
            max_area_px: cell(Column::MaxArea).parse().unwrap(),
            peak_amplitude: cell(Column::Amplitude).parse().unwrap(),
            filled_frames: cell(Column::FilledFrames).parse().unwrap(),
            start_s: cell(Column::StartSeconds).parse().unwrap(),
        }
    }
}

#[cfg(test)]
fn written(recording: &Recording) -> Vec<String> {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tracks.csv");
    write_tracks_csv(
        &path,
        &recording.tracks,
        &recording.timeline().unwrap(),
        Language::English,
        &Debugged,
    )
    .unwrap();
    fs::read_to_string(&path)
        .unwrap()
        .lines()
        .skip(1)
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
fn whole_seconds(clock: &str) -> String {
    let (minutes, seconds) = clock.split_once(':').unwrap();
    let minutes: u64 = minutes.parse().unwrap();
    let seconds: u64 = seconds.split_once('.').unwrap().0.parse().unwrap();
    format!(
        "{}:{:02}:{seconds:02}",
        minutes / SECONDS_PER_MINUTE,
        minutes % SECONDS_PER_MINUTE
    )
}

#[test]
fn the_excerpt_table_holds_the_values_of_the_reference_with_its_times_cut_to_the_second() {
    let recording = Recording::read(&committed(Reference::V011, "video_092_original_3m24-4m05")).unwrap();
    let lines = written(&recording);
    assert_eq!(lines[0], "1,0:00:04,0:00:14,9.62,,290,709.7,912.7,94.8,338,110.3,0,4.8");
    let written: Vec<Written> = lines.iter().map(|line| Written::of(line)).collect();
    assert_eq!(written.len(), recording.rows.as_deref().unwrap().len());
    for (kept, found) in recording.rows.as_deref().unwrap().iter().zip(&written) {
        assert_eq!(
            (found.id, found.hits, found.max_area_px, found.filled_frames),
            (kept.id, kept.hits, kept.max_area_px, kept.filled_frames)
        );
        assert_eq!(
            [
                found.start_s,
                found.duration_s,
                found.chord_px,
                found.path_px,
                found.speed_px_s,
                found.peak_amplitude
            ],
            [
                kept.start_s,
                kept.duration_s,
                kept.chord_px,
                kept.path_px,
                kept.speed_px_s,
                kept.peak_amplitude
            ]
        );
        assert_eq!(
            (found.start.as_str(), found.end.as_str()),
            (whole_seconds(&kept.start).as_str(), whole_seconds(&kept.end).as_str())
        );
        assert_eq!(found.verdict, "");
    }
}

#[test]
fn the_092_table_differs_only_by_times_from_pts_and_areas_ending_in_a_half() {
    let recording = Recording::read(&committed(Reference::V011, "video_092_original")).unwrap();
    let written: Vec<Written> = written(&recording).iter().map(|line| Written::of(line)).collect();
    assert_eq!(written.len(), 14);
    let (mut starts, mut ends, mut larger_areas, mut faster) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for (kept, found) in recording.rows.as_deref().unwrap().iter().zip(&written) {
        assert_eq!(
            (found.id, found.hits, found.filled_frames),
            (kept.id, kept.hits, kept.filled_frames)
        );
        assert_eq!(
            [found.duration_s, found.chord_px, found.path_px, found.peak_amplitude],
            [kept.duration_s, kept.chord_px, kept.path_px, kept.peak_amplitude]
        );
        assert_eq!(found.start, whole_seconds(&kept.start));
        if (found.start_s - kept.start_s).abs() > f64::EPSILON {
            starts.push((found.id, kept.start_s, found.start_s));
        }
        let kept_end = whole_seconds(&kept.end);
        if found.end != kept_end {
            ends.push((found.id, kept_end, found.end.clone()));
        }
        if found.max_area_px != kept.max_area_px {
            assert_eq!(found.max_area_px, kept.max_area_px + 1);
            larger_areas.push(found.id);
        }
        let faster_by = found.speed_px_s - kept.speed_px_s;
        assert!(
            (-1e-9..=0.2 + 1e-9).contains(&faster_by),
            "track {}: {faster_by}",
            found.id
        );
        if faster_by > 1e-9 {
            faster.push(found.id);
        }
    }
    assert_eq!(starts, STARTS_FROM_PTS);
    let ends: Vec<(usize, &str, &str)> = ends
        .iter()
        .map(|(id, kept, found)| (*id, kept.as_str(), found.as_str()))
        .collect();
    assert_eq!(ends, ENDS_ACROSS_A_SECOND);
    assert_eq!(larger_areas, AREAS_ENDING_IN_A_HALF);
    assert_eq!(faster, FASTER_TRACKS);
}
