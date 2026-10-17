use std::path::PathBuf;

use batdetect::eta::Evening;
use batdetect::jobs::Job;
use batdetect::options::Options;
use batdetect::outcome::{Failure, Panic, Ran, Rank};
use batdetect::process::{Extent, Processed};
use batdetect::progress::Step;
use batdetect::reuse::Earlier;
use files::params::Finish;
use files::previous::Kept;
use render::clips::Rendered;
use types::language::Language;

use super::{Status, Video};
use crate::texts::Texts;

const TEXTS: Texts = Texts::new(Language::French);

#[derive(Debug, Clone, Copy)]
enum Action<'a> {
    Show(&'a str),
    Clear,
}

fn written(terminal: bool, actions: &[Action<'_>]) -> String {
    let status = Status::new(TEXTS, terminal, 1);
    let mut out = Vec::new();
    for action in actions {
        match action {
            Action::Show(line) => status.show(&mut out, line).unwrap(),
            Action::Clear => status.clear(&mut out).unwrap(),
        }
    }
    String::from_utf8(out).unwrap()
}

#[test]
fn redirected_each_step_is_a_line_of_its_own_and_nothing_is_ever_erased() {
    let actions = [
        Action::Show("[1/2] v — détection"),
        Action::Clear,
        Action::Show("[1/2] v — suivi"),
        Action::Clear,
    ];
    assert_eq!(written(false, &actions), "[1/2] v — détection\n[1/2] v — suivi\n");
}

#[test]
fn on_a_terminal_a_step_is_drawn_over_the_previous_one_with_spaces_and_no_escape_sequence() {
    let actions = [
        Action::Show("[1/2] vidéo — détection"),
        Action::Show("[1/2] vidéo — suivi"),
    ];
    assert_eq!(
        written(true, &actions),
        "[1/2] vidéo — détection\r                       \r[1/2] vidéo — suivi"
    );
}

#[test]
fn on_a_terminal_clearing_erases_the_drawn_step_once_and_then_does_nothing() {
    let actions = [Action::Show("abc"), Action::Clear, Action::Clear];
    assert_eq!(written(true, &actions), "abc\r   \r");
}

#[test]
fn on_a_terminal_the_erased_width_counts_characters_and_not_bytes() {
    assert_eq!(written(true, &[Action::Show("é"), Action::Clear]), "é\r \r");
}

fn told(result: &Result<Ran, Failure>) -> String {
    let status = Status::new(TEXTS, true, 1);
    let job = Job {
        video: PathBuf::from("nuit/flight.mp4"),
        dest: PathBuf::from("out/flight"),
    };
    let video = Video {
        status: &status,
        rank: Rank { position: 1, total: 1 },
        job: &job,
    };
    let mut err = Vec::new();
    status.show(&mut err, "[1/1] flight — extraits").unwrap();
    video.tell(&mut err, result, &Options::default()).unwrap();
    String::from_utf8(err).unwrap()
}

#[test]
fn on_a_terminal_the_step_is_erased_before_the_last_line_of_the_video() {
    let processed = Processed {
        tracks: Vec::new(),
        extent: Extent {
            length_s: 2.0,
            not_analysed_s: 0.0,
        },
        csv: None,
        kept: vec![Kept {
            path: PathBuf::from("out/flight/flight_pistes.csv"),
            ours: true,
        }],
        summaries: Vec::new(),
        rendered: Rendered::default(),
    };
    assert_eq!(
        told(&Ok(Ran::Processed(processed))),
        format!("[1/1] flight — extraits\r{}\r[1/1] flight — 0 piste\n", " ".repeat(23))
    );
}

#[test]
fn on_a_terminal_a_failure_erases_the_step_then_gives_its_cause_above_its_last_line() {
    let panic = Panic {
        message: "boom".to_owned(),
        location: None,
    };
    assert_eq!(
        told(&Err(Failure::Panic(panic))),
        format!(
            "[1/1] flight — extraits\r{}\rnuit/flight.mp4\u{a0}: erreur interne\n  à faire\u{a0}: le signaler avec le détail \
             ci-dessous\n  détail\u{a0}: boom\n[1/1] flight — échec\n",
            " ".repeat(23)
        )
    );
}

#[test]
fn an_interruption_erases_the_step_on_a_terminal_then_names_the_video_reached_and_how_to_resume_the_lot() {
    let step = "[3/12] flight — détection";
    let width = step.chars().count();
    for (terminal, erased) in [(true, format!("\r{}\r", " ".repeat(width))), (false, "\n".to_owned())] {
        let status = Status::new(TEXTS, terminal, 12);
        status.enter(Rank { position: 3, total: 12 });
        let mut err = Vec::new();
        status.show(&mut err, step).unwrap();
        status.interrupted(&mut err).unwrap();
        assert_eq!(
            String::from_utf8(err).unwrap(),
            format!(
                "{step}{erased}lot interrompu à la vidéo 3/12\u{a0}; relancer la même commande reprend là où \
                 le lot s'est arrêté\n"
            )
        );
    }
}

#[test]
fn an_interruption_in_english_says_the_same() {
    let status = Status::new(Texts::new(Language::English), false, 2);
    let mut err = Vec::new();
    status.interrupted(&mut err).unwrap();
    assert_eq!(
        String::from_utf8(err).unwrap(),
        "batch interrupted at video 1/2; running the same command again resumes where the batch stopped\n"
    );
}

#[test]
fn a_video_already_done_announces_no_step_and_says_so_with_its_earlier_track_count() {
    let status = Status::new(TEXTS, true, 1);
    let job = Job {
        video: PathBuf::from("nuit/flight.mp4"),
        dest: PathBuf::from("out/flight"),
    };
    let video = Video {
        status: &status,
        rank: Rank { position: 1, total: 1 },
        job: &job,
    };
    let earlier = Earlier {
        finish: Finish::Done,
        tracks: 4,
        limit: 300,
        extent: Extent {
            length_s: 40.0,
            not_analysed_s: 0.0,
        },
    };
    let mut err = Vec::new();
    video
        .tell(&mut err, &Ok(Ran::Skipped(earlier)), &Options::default())
        .unwrap();
    assert_eq!(String::from_utf8(err).unwrap(), "[1/1] flight — déjà faite, 4 pistes\n");
}

#[test]
fn an_interruption_under_force_says_that_the_same_command_redoes_the_whole_lot() {
    for (language, said) in [
        (
            Language::French,
            "lot interrompu à la vidéo 1/2\u{a0}; relancer la même commande refait le lot\n",
        ),
        (
            Language::English,
            "batch interrupted at video 1/2; running the same command again redoes the batch\n",
        ),
    ] {
        let status = Status::new(Texts::new(language), false, 2).redoing_all(true);
        let mut err = Vec::new();
        status.interrupted(&mut err).unwrap();
        assert_eq!(String::from_utf8(err).unwrap(), said);
    }
}

#[test]
fn on_a_terminal_a_line_wider_than_the_terminal_is_cut_one_column_short_and_erased_as_cut() {
    let status = Status::new(TEXTS, true, 1).fitted_to(|| Some(10));
    let mut out = Vec::new();
    status.show(&mut out, "[1/2] vidéo — détection").unwrap();
    status.clear(&mut out).unwrap();
    assert_eq!(String::from_utf8(out).unwrap(), "[1/2] vid\r         \r");
}

#[test]
fn redirected_a_line_is_never_cut_whatever_the_width() {
    let status = Status::new(TEXTS, false, 1).fitted_to(|| Some(10));
    let mut out = Vec::new();
    status.show(&mut out, "[1/2] vidéo — détection").unwrap();
    assert_eq!(String::from_utf8(out).unwrap(), "[1/2] vidéo — détection\n");
}

fn evening_said(terminal: bool) -> Vec<(String, String)> {
    let status = Status::new(TEXTS, terminal, 3).timed(Evening::new(vec![20, 20, 20]));
    let mut said = Vec::new();
    for (index, name) in ["a", "b", "c"].into_iter().enumerate() {
        let job = Job {
            video: PathBuf::from(format!("nuit/{name}.mp4")),
            dest: PathBuf::from(format!("out/{name}")),
        };
        let rank = Rank {
            position: index + 1,
            total: 3,
        };
        status.enter(rank);
        let video = Video {
            status: &status,
            rank,
            job: &job,
        };
        let mut step = Vec::new();
        video.step_on(&mut step, Step::Detection).unwrap();
        status.computed(rank);
        let processed = Processed {
            tracks: Vec::new(),
            extent: Extent {
                length_s: 2.0,
                not_analysed_s: 0.0,
            },
            csv: None,
            kept: Vec::new(),
            summaries: Vec::new(),
            rendered: Rendered::default(),
        };
        let mut last = Vec::new();
        video
            .tell(&mut last, &Ok(Ran::Processed(processed)), &Options::default())
            .unwrap();
        said.push((String::from_utf8(step).unwrap(), String::from_utf8(last).unwrap()));
    }
    said
}

fn finish_of(line: &str) -> Option<&str> {
    let (_, time) = line.trim_end().split_once(" · fin vers ")?;
    Some(time)
}

fn is_clock_time(time: Option<&str>) -> bool {
    time.is_some_and(|time| {
        time.len() == 5
            && time
                .chars()
                .enumerate()
                .all(|(index, char)| (index == 2) == (char == ':'))
    })
}

#[test]
fn on_a_terminal_the_finish_joins_the_step_once_a_first_video_is_computed_and_never_the_last_line() {
    let said = evening_said(true);
    assert_eq!(said[0].0, "[1/3] a — détection");
    assert!(is_clock_time(finish_of(&said[1].0)), "{:?}", said[1].0);
    assert!(
        said[1].0.starts_with("[2/3] b — détection · fin vers "),
        "{:?}",
        said[1].0
    );
    assert!(is_clock_time(finish_of(&said[2].0)), "{:?}", said[2].0);
    for (_, last) in &said {
        assert_eq!(finish_of(last), None, "{last:?}");
    }
}

#[test]
fn redirected_the_finish_joins_the_last_line_of_a_video_while_videos_are_left_and_never_a_step() {
    let said = evening_said(false);
    for (step, _) in &said {
        assert_eq!(finish_of(step), None, "{step:?}");
    }
    assert!(is_clock_time(finish_of(&said[0].1)), "{:?}", said[0].1);
    assert!(
        said[0].1.starts_with("[1/3] a — 0 piste · fin vers "),
        "{:?}",
        said[0].1
    );
    assert!(is_clock_time(finish_of(&said[1].1)), "{:?}", said[1].1);
    assert_eq!(said[2].1, "[3/3] c — 0 piste\n");
}
