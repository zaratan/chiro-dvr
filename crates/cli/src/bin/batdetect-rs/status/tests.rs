use std::path::PathBuf;

use batdetect::jobs::Job;
use batdetect::outcome::{Failure, Rank};
use batdetect::process::Processed;
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
    let status = Status::new(TEXTS, terminal);
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

fn told(result: &Result<Processed, Failure>) -> (String, String) {
    let status = Status::new(TEXTS, true);
    let job = Job {
        video: PathBuf::from("nuit/flight.mp4"),
        dest: PathBuf::from("out/flight"),
    };
    let video = Video {
        status: &status,
        rank: Rank { position: 1, total: 1 },
        job: &job,
    };
    let (mut err, mut out) = (Vec::new(), Vec::new());
    status.show(&mut err, "[1/1] flight — extraits").unwrap();
    video.tell(&mut err, &mut out, result).unwrap();
    (String::from_utf8(err).unwrap(), String::from_utf8(out).unwrap())
}

#[test]
fn on_a_terminal_the_step_is_erased_before_the_events_and_the_last_line_of_the_video() {
    let processed = Processed {
        tracks: Vec::new(),
        csv: None,
        kept: vec![Kept {
            path: PathBuf::from("out/flight/flight_pistes.csv"),
            ours: true,
        }],
        summaries: Vec::new(),
        rendered: Rendered::default(),
    };
    let (err, out) = told(&Ok(processed));
    let erased = format!("[1/1] flight — extraits\r{}\r", " ".repeat(23));
    let rest = err.strip_prefix(&erased).unwrap_or_else(|| panic!("{err:?}"));
    let lines: Vec<&str> = rest.lines().collect();
    assert_eq!(lines.len(), 2, "{rest:?}");
    assert!(lines[0].starts_with("nuit/flight.mp4 : flight_pistes.csv laissé tel quel"));
    assert_eq!(lines[1], "[1/1] flight — 0 piste");
    assert_eq!(out, "flight.mp4 : 0 piste -> out/flight\n");
}

#[test]
fn on_a_terminal_a_failure_erases_the_step_then_gives_its_cause_above_its_last_line() {
    let (err, out) = told(&Err(Failure::Panic("boom".to_owned())));
    assert_eq!(
        err,
        format!(
            "[1/1] flight — extraits\r{}\rnuit/flight.mp4 : erreur interne : boom\n[1/1] flight — échec\n",
            " ".repeat(23)
        )
    );
    assert_eq!(out, "");
}
