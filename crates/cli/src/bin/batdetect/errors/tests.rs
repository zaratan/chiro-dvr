use clap::error::ErrorKind;
use types::language::Language;

use super::rendered;
use crate::arguments::parse;
use crate::texts::Texts;

const ENGLISH: Texts = Texts::new(Language::English);
const FRENCH: Texts = Texts::new(Language::French);

fn refused(arguments: &[&str], texts: Texts) -> clap::Error {
    let mut all = vec!["batdetect"];
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
        &["a.mp4", "-o", "out", "--annotated=oui"],
        &["a.mp4", "-o", "out", "-o", "other"],
        &["-o", "out"],
    ]
    .map(<[&str]>::to_vec)
    .to_vec()
}

fn french(arguments: &[&str]) -> String {
    rendered(FRENCH, &refused(arguments, FRENCH)).to_string()
}

#[test]
fn in_english_every_mistake_of_syntax_reads_as_clap_writes_it_but_its_help_flag_names_the_program() {
    for arguments in every_kind_of_mistake() {
        let error = refused(&arguments, ENGLISH);
        assert_eq!(
            rendered(ENGLISH, &error).ansi().to_string(),
            error
                .render()
                .ansi()
                .to_string()
                .replace("--help\u{1b}[0m'.", "batdetect --help\u{1b}[0m'."),
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
         '--threshold'\n\nUtilisation\u{a0}: batdetect --out-dir <DOSSIER> <VIDÉO>...\n\nPour en savoir plus, \
         essayez 'batdetect --help'.\n"
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
         Pour en savoir plus, essayez 'batdetect --help'.\n"
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

fn without_usage(sentence: &str, footer: &str) -> String {
    format!("{sentence}\n\n{footer}\n")
}

#[test]
fn a_value_that_does_not_read_names_its_option_and_its_range_in_each_language_without_the_usage() {
    let footers = [
        "Pour en savoir plus, essayez 'batdetect --help'.",
        "For more information, try 'batdetect --help'.",
    ];
    for (option, value, sentences) in [
        (
            "--max-tracks",
            "many",
            [
                "erreur\u{a0}: --max-tracks doit être un entier ≥ 0, pas many",
                "error: --max-tracks must be an integer ≥ 0, not many",
            ],
        ),
        (
            "--min-hits",
            "0",
            [
                "erreur\u{a0}: --min-hits doit être un entier ≥ 1, pas 0",
                "error: --min-hits must be an integer ≥ 1, not 0",
            ],
        ),
        (
            "--max-blobs",
            "-3",
            [
                "erreur\u{a0}: --max-blobs doit être un entier ≥ 0, pas -3",
                "error: --max-blobs must be an integer ≥ 0, not -3",
            ],
        ),
        (
            "--work-width",
            "x",
            [
                "erreur\u{a0}: --work-width doit être un entier ≥ 16, pas x",
                "error: --work-width must be an integer ≥ 16, not x",
            ],
        ),
        (
            "--threshold",
            "abc",
            [
                "erreur\u{a0}: --threshold doit être un nombre fini > 0, pas abc",
                "error: --threshold must be a finite number > 0, not abc",
            ],
        ),
        (
            "--clip-margin",
            "abc",
            [
                "erreur\u{a0}: --clip-margin doit être un nombre fini de secondes, au moins 0, pas abc",
                "error: --clip-margin must be a finite number of seconds, at least 0, not abc",
            ],
        ),
        (
            "--osd-region",
            "0,0,1",
            [
                "erreur\u{a0}: --osd-region doit être quatre fractions X0,Y0,X1,Y1 séparées par des virgules, pas 0,0,1",
                "error: --osd-region must be four fractions X0,Y0,X1,Y1 separated by commas, not 0,0,1",
            ],
        ),
    ] {
        for ((texts, sentence), footer) in [FRENCH, ENGLISH].into_iter().zip(sentences).zip(footers) {
            let error = refused(&["a.mp4", option, value], texts);
            assert_eq!(error.exit_code(), 2);
            assert_eq!(rendered(texts, &error).to_string(), without_usage(sentence, footer));
        }
    }
}

#[test]
fn a_decimal_comma_is_answered_with_the_point_for_the_options_that_take_one() {
    for (option, refusal, written) in [
        (
            "--threshold",
            "--threshold doit être un nombre fini > 0, pas 11,5",
            "11.5",
        ),
        (
            "--clip-margin",
            "--clip-margin doit être un nombre fini de secondes, au moins 0, pas 1,5",
            "1.5",
        ),
    ] {
        let value = written.replace('.', ",");
        assert_eq!(
            french(&["a.mp4", option, &value]),
            format!(
                "erreur\u{a0}: {refusal}\n\n  conseil\u{a0}: écrire {written}, avec un point\n\nPour en savoir plus, \
                 essayez 'batdetect --help'.\n"
            )
        );
    }
    assert_eq!(
        rendered(ENGLISH, &refused(&["a.mp4", "--threshold", "11,5"], ENGLISH)).to_string(),
        "error: --threshold must be a finite number > 0, not 11,5\n\n  tip: write 11.5, with a point\n\nFor more \
         information, try 'batdetect --help'.\n"
    );
    assert!(!french(&["a.mp4", "--max-tracks", "1,5"]).contains("conseil"));
}

#[test]
fn an_empty_number_is_told_as_a_missing_value() {
    assert!(
        french(&["a.mp4", "--threshold="]).starts_with(
            "erreur\u{a0}: une valeur est attendue pour '--threshold <NIVEAU>', mais aucune n'est donnée\n"
        )
    );
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
fn a_refusal_of_a_setting_gives_its_bound_in_each_language_with_the_help_flag_of_the_program_but_no_usage() {
    assert_eq!(
        french(&["a.mp4", "-o", "out", "--work-width", "8"]),
        "erreur\u{a0}: --work-width doit être un entier ≥ 16, pas 8\n\nPour en savoir plus, essayez 'batdetect \
         --help'.\n"
    );
    assert_eq!(
        rendered(
            ENGLISH,
            &refused(&["a.mp4", "-o", "out", "--threshold", "-0.5"], ENGLISH)
        )
        .to_string(),
        "error: --threshold must be a finite number > 0, not -0.5\n\nFor more information, try 'batdetect \
         --help'.\n"
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
fn an_unforeseen_kind_of_error_says_the_arguments_are_refused_without_an_english_word() {
    let error = clap::Error::new(ErrorKind::TooFewValues);
    assert_eq!(
        rendered(FRENCH, &error).to_string(),
        "erreur\u{a0}: les arguments sont refusés\n\nPour en savoir plus, essayez 'batdetect --help'.\n"
    );
    assert_eq!(
        rendered(ENGLISH, &error).to_string(),
        "error: the arguments are refused\n\nFor more information, try 'batdetect --help'.\n"
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
            OsString::from("batdetect"),
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

#[test]
fn a_display_region_that_starts_with_a_minus_sign_is_refused_by_the_region_sentence_in_each_language() {
    for (texts, sentence, footer) in [
        (
            FRENCH,
            "erreur\u{a0}: --osd-region 1 ne vérifie pas 0 ≤ x0 < x1 ≤ 1 et 0 ≤ y0 < y1 ≤ 1\u{a0}: -0.1,0,1,1",
            "Pour en savoir plus, essayez 'batdetect --help'.",
        ),
        (
            ENGLISH,
            "error: --osd-region 1 does not satisfy 0 ≤ x0 < x1 ≤ 1 and 0 ≤ y0 < y1 ≤ 1: -0.1,0,1,1",
            "For more information, try 'batdetect --help'.",
        ),
    ] {
        let error = refused(&["a.mp4", "--osd-region", "-0.1,0,1,1"], texts);
        assert_eq!(rendered(texts, &error).to_string(), without_usage(sentence, footer));
    }
}

#[test]
fn an_option_given_in_place_of_a_display_region_is_read_as_that_region_and_refused() {
    assert_eq!(
        french(&["a.mp4", "--osd-region", "--threshold"]),
        without_usage(
            "erreur\u{a0}: --osd-region doit être quatre fractions X0,Y0,X1,Y1 séparées par des virgules, pas \
             --threshold",
            "Pour en savoir plus, essayez 'batdetect --help'."
        )
    );
}
