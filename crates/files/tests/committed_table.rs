use std::fs;

use files::table::write_tracks_csv;
use reference::location::committed;
use reference::recording::Recording;
use reference::table::{TrackRow, read};

const STARTS_FROM_PTS: [(usize, &str, &str); 11] = [
    (2, "0:33.94", "0:33.93"),
    (5, "1:12.60", "1:12.59"),
    (6, "1:48.93", "1:48.92"),
    (7, "2:02.82", "2:02.81"),
    (8, "2:24.63", "2:24.62"),
    (9, "2:36.99", "2:36.98"),
    (10, "2:43.95", "2:43.94"),
    (11, "3:29.61", "3:29.59"),
    (12, "3:48.29", "3:48.27"),
    (13, "3:50.89", "3:50.87"),
    (14, "3:57.18", "3:57.16"),
];

const ENDS_FROM_PTS: [(usize, &str, &str); 12] = [
    (2, "0:35.27", "0:35.26"),
    (3, "0:51.89", "0:51.88"),
    (5, "1:13.87", "1:13.86"),
    (6, "1:50.00", "1:49.99"),
    (7, "2:03.49", "2:03.48"),
    (8, "2:25.13", "2:25.12"),
    (9, "2:37.46", "2:37.44"),
    (10, "2:45.38", "2:45.37"),
    (11, "3:39.23", "3:39.21"),
    (12, "3:48.86", "3:48.84"),
    (13, "3:51.69", "3:51.67"),
    (14, "3:58.05", "3:58.03"),
];

const AREAS_ENDING_IN_A_HALF: [usize; 4] = [1, 5, 6, 10];
const FASTER_TRACKS: [usize; 10] = [2, 4, 5, 6, 7, 8, 9, 12, 13, 14];

fn differences<'a>(
    expected: &'a [TrackRow],
    written: &'a [TrackRow],
    column: fn(&TrackRow) -> &str,
) -> Vec<(usize, &'a str, &'a str)> {
    expected
        .iter()
        .zip(written)
        .filter(|(kept, found)| column(kept) != column(found))
        .map(|(kept, found)| (kept.id, column(kept), column(found)))
        .collect()
}

#[test]
fn the_excerpt_table_is_the_reference_byte_for_byte_but_for_its_windows_line_ends() {
    let recording = Recording::read(&committed("video_092_original_3m24-4m05")).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tracks.csv");
    write_tracks_csv(&path, &recording.tracks, &recording.timeline().unwrap()).unwrap();
    let committed_csv = fs::read_to_string(committed("video_092_original_3m24-4m05").join("tracks.csv")).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap().replace("\r\n", "\n"), committed_csv);
}

#[test]
fn the_092_table_differs_only_by_times_from_pts_and_areas_ending_in_a_half() {
    let recording = Recording::read(&committed("video_092_original")).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tracks.csv");
    write_tracks_csv(&path, &recording.tracks, &recording.timeline().unwrap()).unwrap();
    let (expected, written) = (&recording.rows, read(&path).unwrap());
    assert_eq!(written.len(), 14);
    assert_eq!(
        differences(expected, &written, |row| row.start.as_str()),
        STARTS_FROM_PTS
    );
    assert_eq!(differences(expected, &written, |row| row.end.as_str()), ENDS_FROM_PTS);
    let mut larger_areas = Vec::new();
    let mut faster = Vec::new();
    for (kept, found) in expected.iter().zip(&written) {
        let untimed = |row: &TrackRow| {
            (
                row.id,
                row.hits,
                row.chord_px,
                row.path_px,
                row.peak_amplitude,
                row.filled_frames,
            )
        };
        assert_eq!(untimed(found), untimed(kept));
        assert_eq!(found.duration_s, kept.duration_s);
        assert!((found.start_s - kept.start_s).abs() <= 0.025);
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
    assert_eq!(larger_areas, AREAS_ENDING_IN_A_HALF);
    assert_eq!(faster, FASTER_TRACKS);
}
