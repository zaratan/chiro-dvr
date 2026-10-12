use super::{HeaderLine, clock, duration, header_lines};
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

#[test]
fn damaged_time_is_summed_on_its_own_line_after_the_unstable_ranges() {
    let both = period(0.0, 300.0, &[[0.0, 8.59]], &[[10.0, 10.5], [40.0, 117.3]]);
    assert_eq!(
        header_lines("v122", &both)[3..],
        [
            not_analysed("0:00", "0:09"),
            HeaderLine::Unreadable {
                duration: "1 min 18 s".to_owned(),
                spans: 2,
            },
        ]
    );
}

#[test]
fn period_without_damage_has_no_damage_line() {
    assert_eq!(header_lines("v092", &period(0.0, 300.0, &[], &[])).len(), 3);
}

#[test]
fn the_clock_truncates_to_the_second_and_the_duration_rounds_up() {
    assert_eq!(
        (clock(59.99), clock(600.0), clock(-1.0)),
        ("0:59".into(), "10:00".into(), "0:00".into())
    );
    assert_eq!(
        (duration(0.2), duration(59.4), duration(202.0)),
        ("1 s".into(), "1 min 0 s".into(), "3 min 22 s".into())
    );
}

#[test]
fn the_longest_damage_line_counts_six_hundred_spans_over_ten_minutes() {
    let spans: Vec<[f64; 2]> = (0..600).map(|k| [f64::from(k), f64::from(k) + 0.999]).collect();
    let line = header_lines("v", &period(0.0, 600.0, &[], &spans)).pop();
    assert_eq!(
        line,
        Some(HeaderLine::Unreadable {
            duration: "10 min 0 s".to_owned(),
            spans: 600,
        })
    );
}
