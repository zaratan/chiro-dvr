use super::{HeaderLine, clock, duration, header_lines, whole_seconds};
use crate::periods::Period;
use crate::testing::track;

fn period(start: f64, end: f64, ignored: &[[f64; 2]], damaged: &[[f64; 2]]) -> Period<'static> {
    Period {
        start,
        end,
        tracks: Vec::new(),
        alone: true,
        ignored: ignored.to_vec(),
        damaged: damaged.to_vec(),
    }
}

fn window(start: &str, end: &str) -> HeaderLine {
    HeaderLine::Window {
        start: start.to_owned(),
        end: end.to_owned(),
    }
}

fn not_analysed(start: &str, end: &str) -> HeaderLine {
    HeaderLine::NotAnalysed {
        start: start.to_owned(),
        end: end.to_owned(),
    }
}

fn five_short_ranges() -> Vec<[f64; 2]> {
    (0..5)
        .map(|k| [10.0 * f64::from(k), 10.0 * f64::from(k) + 2.0])
        .collect()
}

#[test]
fn header_counts_tracks_not_passages_since_only_the_naturalist_confirms_one_and_shows_the_period() {
    let one = track(1, 0, &[[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]);
    let mut with_one = period(0.0, 300.0, &[], &[]);
    with_one.tracks.push(&one);
    assert_eq!(
        header_lines("v092", &with_one),
        [
            HeaderLine::Name("v092".to_owned()),
            HeaderLine::Tracks(1),
            window("0:00", "5:00")
        ]
    );
    assert_eq!(
        header_lines("v092", &period(600.0, 1200.0, &[], &[]))[1..],
        [HeaderLine::Tracks(0), window("10:00", "20:00")]
    );
}

#[test]
fn ignored_time_is_listed_rounded_outward_so_reviewing_it_covers_the_whole_gap() {
    let ignored = period(0.0, 300.0, &[[0.0, 8.59], [225.96, 251.21]], &[]);
    assert_eq!(
        header_lines("v089", &ignored)[3..],
        [not_analysed("0:00", "0:09"), not_analysed("3:45", "4:12")]
    );
}

#[test]
fn more_than_three_ignored_ranges_are_summed_up_on_a_fourth_line() {
    let ignored = period(0.0, 300.0, &five_short_ranges(), &[]);
    assert_eq!(
        header_lines("v", &ignored)[3..],
        [
            not_analysed("0:00", "0:02"),
            not_analysed("0:10", "0:12"),
            not_analysed("0:20", "0:22"),
            HeaderLine::More(2),
        ]
    );
}

#[test]
fn exactly_three_ignored_ranges_need_no_fourth_line() {
    let ignored = period(0.0, 300.0, &five_short_ranges()[..3], &[]);
    assert_eq!(header_lines("v", &ignored).len(), 6);
}

fn unreadable(start: &str, end: &str) -> HeaderLine {
    HeaderLine::Unreadable {
        start: start.to_owned(),
        end: end.to_owned(),
    }
}

#[test]
fn unreadable_spans_are_dated_after_the_unstable_ranges_so_they_can_be_reviewed_by_eye() {
    let both = period(0.0, 300.0, &[[0.0, 8.59]], &[[10.0, 10.5], [40.0, 117.3]]);
    assert_eq!(
        header_lines("v122", &both)[3..],
        [
            not_analysed("0:00", "0:09"),
            unreadable("0:10", "0:11"),
            unreadable("0:40", "1:58"),
        ]
    );
}

#[test]
fn more_than_three_unreadable_spans_are_summed_up_on_their_own_fourth_line() {
    let both = period(0.0, 300.0, &five_short_ranges(), &five_short_ranges());
    let lines = header_lines("v121", &both);
    assert_eq!(lines.len(), 3 + 4 + 4);
    assert_eq!(
        lines[7..],
        [
            unreadable("0:00", "0:02"),
            unreadable("0:10", "0:12"),
            unreadable("0:20", "0:22"),
            HeaderLine::MoreUnreadable {
                count: 2,
                total_seconds: 10
            },
        ]
    );
}

#[test]
fn period_without_damage_has_no_damage_line() {
    assert_eq!(header_lines("v092", &period(0.0, 300.0, &[], &[])).len(), 3);
}

#[test]
fn the_clock_truncates_to_the_second() {
    assert_eq!(
        (clock(59.99), clock(600.0), clock(-1.0)),
        ("0:59".into(), "10:00".into(), "0:00".into())
    );
}

#[test]
fn spans_that_touch_or_overlap_once_rounded_to_the_second_are_listed_as_one() {
    assert_eq!(
        whole_seconds(&[[4.2, 5.6], [5.9, 7.3], [7.5, 8.1], [12.0, 13.0]]),
        [[4, 9], [12, 13]]
    );
    assert_eq!(whole_seconds(&[[4.2, 4.9], [5.1, 6.0]]), [[4, 6]]);
    assert_eq!(whole_seconds(&[[4.2, 4.9], [6.1, 7.0]]), [[4, 5], [6, 7]]);
    let merged = period(0.0, 300.0, &[], &[[4.2, 5.6], [5.9, 7.3], [7.5, 8.1]]);
    assert_eq!(header_lines("v121", &merged)[3..], [unreadable("0:04", "0:09")]);
}

#[test]
fn the_unreadable_total_counts_the_exact_spans_not_the_merged_seconds() {
    let spans: Vec<[f64; 2]> = (0..5)
        .map(|k| [10.0 * f64::from(k) + 0.5, 10.0 * f64::from(k) + 1.0])
        .collect();
    let lines = header_lines("v", &period(0.0, 300.0, &[], &spans));
    assert_eq!(
        lines.last(),
        Some(&HeaderLine::MoreUnreadable {
            count: 2,
            total_seconds: 3
        })
    );
}

#[test]
fn a_duration_reads_in_minutes_and_seconds() {
    assert_eq!(
        (duration(1), duration(59), duration(202)),
        ("1 s".into(), "59 s".into(), "3 min 22 s".into())
    );
}

#[test]
fn spans_given_out_of_order_are_all_kept() {
    assert_eq!(whole_seconds(&[[10.0, 12.0], [4.0, 5.0]]), [[4, 5], [10, 12]]);
}
