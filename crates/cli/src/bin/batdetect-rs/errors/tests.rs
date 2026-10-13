use clap::error::ErrorKind;
use types::language::Language;

use super::rendered;
use crate::arguments::parse;
use crate::texts::Texts;

const ENGLISH: Texts = Texts::new(Language::English);
const FRENCH: Texts = Texts::new(Language::French);

fn refused(arguments: &[&str], texts: Texts) -> clap::Error {
    let mut all = vec!["batdetect-rs"];
    all.extend_from_slice(arguments);
    parse(all, texts).unwrap_err()
}

fn every_kind_of_mistake() -> Vec<Vec<&'static str>> {
    [
        &["a.mp4", "-o", "out", "--tresh", "3"][..],
        &["a.mp4", "-o", "out", "-x"],
        &["a.mp4", "-o", "out", "--zoom", "x"],
        &["a.mp4", "-o", "out", "--zoom", "al"],
        &["a.mp4", "-o", "out", "--zoom="],
        &["a.mp4", "--max-tracks"],
        &["a.mp4", "-o", "out", "--max-tracks", "many"],
        &["a.mp4", "-o", "out", "--annotated=oui"],
        &["a.mp4", "-o", "out", "-o", "other"],
        &["-o", "out"],
        &["a.mp4"],
    ]
    .map(<[&str]>::to_vec)
    .to_vec()
}

fn french(arguments: &[&str]) -> String {
    rendered(FRENCH, &refused(arguments, FRENCH)).to_string()
}

#[test]
fn in_english_every_mistake_reads_exactly_as_clap_writes_it_colours_included() {
    for arguments in every_kind_of_mistake() {
        let error = refused(&arguments, ENGLISH);
        assert_eq!(
            rendered(ENGLISH, &error).ansi().to_string(),
            error.render().ansi().to_string(),
            "{arguments:?}"
        );
    }
}

#[test]
fn in_french_no_mistake_keeps_an_english_word_of_clap() {
    for arguments in every_kind_of_mistake() {
        let text = french(&arguments);
        for english in [
            "error:",
            "tip:",
            "Usage",
            "For more information",
            "unexpected",
            "invalid value",
            "is required",
            "cannot be used",
            "were not provided",
            "similar argument",
            "similar value",
            "as a value",
        ] {
            assert!(!text.contains(english), "{english}: {text}");
        }
    }
}

#[test]
fn in_french_an_unknown_option_is_named_with_its_closest_option() {
    assert_eq!(
        french(&["a.mp4", "-o", "out", "--tresh", "3"]),
        "erreur\u{a0}: argument non reconnu\u{a0}: '--tresh'\n\n  conseil\u{a0}: un argument proche existe\u{a0}: \
         '--threshold'\n\nUtilisation\u{a0}: batdetect-rs --out-dir <DOSSIER> <VIDÉO>...\n\nPour en savoir plus, \
         essayez '--help'.\n"
    );
    assert!(
        french(&["a.mp4", "-o", "out", "-x"]).contains("conseil\u{a0}: pour passer '-x' comme valeur, écrire '-- -x'")
    );
}

#[test]
fn in_french_a_wrong_choice_lists_the_possible_values() {
    assert_eq!(
        french(&["a.mp4", "-o", "out", "--zoom", "x"]),
        "erreur\u{a0}: valeur invalide 'x' pour '--zoom <CHOIX>'\n  [valeurs possibles\u{a0}: auto, all, none]\n\n\
         Pour en savoir plus, essayez '--help'.\n"
    );
    assert!(french(&["a.mp4", "-o", "out", "--zoom="]).starts_with(
        "erreur\u{a0}: une valeur est attendue pour '--zoom <CHOIX>', mais aucune n'est donnée\n  [valeurs possibles"
    ));
}

#[test]
fn in_french_a_choice_close_to_a_possible_value_is_named() {
    assert!(
        french(&["a.mp4", "-o", "out", "--zoom", "al"])
            .contains("\n\n  conseil\u{a0}: une valeur proche existe\u{a0}: 'all'\n")
    );
}

#[test]
fn in_french_a_value_that_is_not_a_number_keeps_the_english_cause_after_the_french_sentence() {
    assert!(french(&["a.mp4", "-o", "out", "--max-tracks", "many"]).starts_with(
        "erreur\u{a0}: valeur invalide 'many' pour '--max-tracks <NOMBRE>'\u{a0}: invalid digit found in string\n"
    ));
}

#[test]
fn in_french_a_repeated_option_a_value_given_to_a_switch_and_a_missing_video_are_told_in_french() {
    assert!(
        french(&["a.mp4", "-o", "out", "-o", "other"])
            .starts_with("erreur\u{a0}: l'argument '--out-dir <DOSSIER>' ne peut être donné qu'une fois\n")
    );
    assert!(
        french(&["a.mp4", "-o", "out", "--annotated=oui"])
            .starts_with("erreur\u{a0}: valeur inattendue 'oui' pour '--annotated'\u{a0}: aucune n'était attendue\n")
    );
    assert!(
        french(&["-o", "out"]).starts_with("erreur\u{a0}: arguments obligatoires manquants\u{a0}:\n  <VIDÉO>...\n")
    );
}

#[test]
fn a_refusal_of_a_setting_gives_its_bound_in_each_language_with_the_usage_and_the_help_flag() {
    assert_eq!(
        french(&["a.mp4", "-o", "out", "--work-width", "8"]),
        "erreur\u{a0}: --work-width doit être un entier ≥ 16, pas 8\n\nUtilisation\u{a0}: batdetect-rs [OPTIONS] \
         --out-dir <DOSSIER> <VIDÉO>...\n\nPour en savoir plus, essayez '--help'.\n"
    );
    assert_eq!(
        rendered(
            ENGLISH,
            &refused(&["a.mp4", "-o", "out", "--threshold", "-0.5"], ENGLISH)
        )
        .to_string(),
        "error: --threshold must be a finite number > 0, not -0.5\n\nUsage: batdetect-rs [OPTIONS] --out-dir \
         <FOLDER> <VIDEO>...\n\nFor more information, try '--help'.\n"
    );
    assert!(french(&["a.mp4", "-o", "out", "--threshold", "-0.5"]).contains("pas -0,5\n"));
}

#[test]
fn a_refusal_is_a_usage_error_in_bold_red_in_each_language() {
    for (texts, label) in [(ENGLISH, "error:"), (FRENCH, "erreur\u{a0}:")] {
        let error = refused(&["a.mp4", "-o", "out", "--clip-margin", "-2"], texts);
        assert_eq!((error.kind(), error.exit_code()), (ErrorKind::ValueValidation, 2));
        let coloured = rendered(texts, &error).ansi().to_string();
        assert!(
            coloured.starts_with(&format!("\u{1b}[1m\u{1b}[31m{label}\u{1b}[0m ")),
            "{coloured:?}"
        );
    }
}

#[test]
fn an_unforeseen_kind_of_error_says_the_arguments_are_refused_and_keeps_the_english_kind() {
    let error = clap::Error::new(ErrorKind::TooFewValues);
    let english_kind = ErrorKind::TooFewValues.as_str().unwrap();
    assert_eq!(
        rendered(FRENCH, &error).to_string(),
        format!(
            "erreur\u{a0}: les arguments sont refusés\u{a0}: {english_kind}\n\nPour en savoir plus, essayez '--help'.\n"
        )
    );
}

#[cfg(unix)]
#[test]
fn a_choice_that_is_not_utf8_is_told_in_each_language() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    for (texts, sentence) in [
        (ENGLISH, "error: invalid UTF-8 was detected in one or more arguments"),
        (FRENCH, "erreur\u{a0}: UTF-8 invalide dans un ou plusieurs arguments"),
    ] {
        let arguments = [
            OsString::from("batdetect-rs"),
            OsString::from("a.mp4"),
            OsString::from("-o"),
            OsString::from("out"),
            OsString::from("--zoom"),
            OsString::from_vec(vec![0xff]),
        ];
        let error = parse(arguments, texts).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidUtf8);
        assert!(rendered(texts, &error).to_string().starts_with(sentence));
    }
}
