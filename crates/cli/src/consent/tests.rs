use std::io::{self, BufRead, Cursor, Read};

use super::{Asking, Choice};

const QUESTION: &str = "Traiter quand même la vidéo valide\u{a0}? (O/n)";
const AT_THE_TERMINAL: Asking = Asking {
    skip_refused: false,
    terminal: true,
};

fn answered(answers: &str) -> (Choice, String) {
    let mut output = Vec::new();
    let choice = AT_THE_TERMINAL
        .choose(QUESTION, &mut Cursor::new(answers), &mut output)
        .unwrap();
    (choice, String::from_utf8(output).unwrap())
}

#[test]
fn enter_and_every_yes_letter_in_either_language_process_the_valid_videos() {
    for answer in ["\n", "o\n", "O\n", "y\n", "Y\n", "  o \n", "Y"] {
        assert_eq!(
            answered(answer),
            (Choice::Process, format!("{QUESTION} ")),
            "{answer:?}"
        );
    }
}

#[test]
fn n_in_either_case_refuses_after_a_single_question() {
    for answer in ["n\n", "N\n"] {
        assert_eq!(answered(answer), (Choice::Refuse, format!("{QUESTION} ")), "{answer:?}");
    }
}

#[test]
fn any_other_answer_asks_the_same_question_again() {
    assert_eq!(
        answered("oui\nnon\n\n"),
        (Choice::Process, format!("{QUESTION} {QUESTION} {QUESTION} "))
    );
}

#[test]
fn the_end_of_the_input_refuses_and_ends_the_question_line_instead_of_asking_forever() {
    assert_eq!(answered(""), (Choice::Refuse, format!("{QUESTION} \n")));
    assert_eq!(
        answered("peut-être\n"),
        (Choice::Refuse, format!("{QUESTION} {QUESTION} \n"))
    );
}

#[derive(Debug)]
struct Broken;

impl Read for Broken {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::from(io::ErrorKind::BrokenPipe))
    }
}

impl BufRead for Broken {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Err(io::Error::from(io::ErrorKind::BrokenPipe))
    }

    fn consume(&mut self, _: usize) {}
}

#[test]
fn an_input_that_cannot_be_read_is_an_error_for_the_command_to_take_as_a_no() {
    let error = AT_THE_TERMINAL
        .choose(QUESTION, &mut Broken, &mut Vec::new())
        .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
}

#[test]
fn without_a_terminal_nothing_is_asked_and_the_refusal_stands() {
    let asking = Asking {
        skip_refused: false,
        terminal: false,
    };
    let mut output = Vec::new();
    assert_eq!(
        asking.choose(QUESTION, &mut Cursor::new("o\n"), &mut output).unwrap(),
        Choice::Refuse
    );
    assert_eq!(output, Vec::<u8>::new());
    assert!(!asking.asks());
    assert!(!asking.shows_the_refusals_first());
}

#[test]
fn skip_refused_processes_the_valid_videos_without_asking_with_or_without_a_terminal() {
    for terminal in [true, false] {
        let asking = Asking {
            skip_refused: true,
            terminal,
        };
        let mut output = Vec::new();
        assert_eq!(
            asking.choose(QUESTION, &mut Cursor::new("n\n"), &mut output).unwrap(),
            Choice::Process
        );
        assert_eq!(output, Vec::<u8>::new());
        assert!(!asking.asks());
        assert!(asking.shows_the_refusals_first());
    }
    assert!(AT_THE_TERMINAL.asks());
}
