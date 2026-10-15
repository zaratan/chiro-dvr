use std::path::PathBuf;

use batdetect::jobs::Job;
use batdetect::options::Options;
use batdetect::outcome::{Failure, Panic, Rank};
use batdetect::process::{Extent, Processed};
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

fn told(result: &Result<Processed, Failure>) -> String {
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
        told(&Ok(processed)),
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
fn an_interruption_erases_the_step_on_a_terminal_then_names_the_video_reached_and_how_to_redo_the_lot() {
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
                "{step}{erased}lot interrompu à la vidéo 3/12\u{a0}; relancer la même commande refait le \
                 lot\n"
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
        "batch interrupted at video 1/2; running the same command again redoes the batch\n"
    );
}
