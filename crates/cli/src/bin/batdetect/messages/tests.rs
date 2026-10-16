use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

use batdetect::access::Access;
use batdetect::failure::Cause;
use batdetect::jobs::{self, Job, Reason, Refused};
use batdetect::options::Options;
use batdetect::outcome::Rank;
use batdetect::outcome::{Failure, Panic};
use batdetect::process::{self, Extent};
use batdetect::progress::Step;
use batdetect::writable::Unwritable;
use engine::detection;
use engine::input;
use engine::lost::Losses;
use types::language::Language;

use super::{
    detection_refused, failure_lines, folders_refused, inputs_refused, inputs_skipped, option_refused, redone,
    render_refused, tracking_refused, video_done, video_failed, video_step,
};
use crate::texts::{Fixed, Message, Texts};

const THIRD: Rank = Rank { position: 3, total: 12 };

fn said(language: Language, message: &Message) -> String {
    Texts::new(language).say(message)
}

fn job() -> Job {
    Job {
        video: PathBuf::from("nuit/video_092.mp4"),
        dest: PathBuf::from("out/video_092"),
    }
}

#[derive(Debug)]
struct Layer(&'static str, Option<Box<Layer>>);

impl fmt::Display for Layer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl Error for Layer {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.1.as_deref().map(|cause| cause as &(dyn Error + 'static))
    }
}

fn lines(language: Language, failure: &Failure) -> Vec<String> {
    failure_lines(&job(), failure, &Options::default())
        .iter()
        .map(|message| said(language, message))
        .collect()
}

fn too_many(annotated: bool) -> Failure {
    Failure::Error(process::Error::TooManyTracks {
        count: 301,
        limit: 300,
        annotated,
        extent: Extent {
            length_s: 60.0,
            not_analysed_s: 0.0,
        },
        kept: Vec::new(),
    })
}

fn damaged() -> Failure {
    Failure::Error(process::Error::Detection(detection::Error::Input(
        input::Error::FrameCount {
            path: PathBuf::from("nuit/video_092.mp4"),
            announced: 30,
            losses: Losses::default(),
            decoded: 20,
        },
    )))
}

#[test]
fn a_known_failure_says_its_cause_what_to_do_and_the_english_detail_once() {
    let english = lines(Language::English, &damaged());
    assert_eq!(english.len(), 3, "{english:?}");
    assert_eq!(english[0], "nuit/video_092.mp4: the video is damaged or incomplete");
    assert_eq!(
        english[1],
        "  what to do: copy it again from the binoculars over the USB cable, not through Stream Vision 2, then run \
         again; if it stays damaged, report it"
    );
    assert!(
        english[2].starts_with("  detail: the detection failed: nuit/video_092.mp4: decoded 20 frames"),
        "{}",
        english[2]
    );
    assert_eq!(english.concat().matches("failed").count(), 1, "{english:?}");
    let french = lines(Language::French, &damaged());
    assert_eq!(french[0], "nuit/video_092.mp4\u{a0}: la vidéo est abîmée ou incomplète");
    assert!(french[1].starts_with("  à faire\u{a0}: la recopier depuis les jumelles par le câble USB"));
    assert!(
        french[2].starts_with("  détail\u{a0}: the detection failed: nuit/video_092.mp4: decoded 20 frames"),
        "{}",
        french[2]
    );
}

#[test]
fn a_full_disk_names_the_folder_of_the_results() {
    let full = Failure::Error(process::Error::Files(files::error::Error::Write {
        path: PathBuf::from("out/video_092/video_092_pistes.csv"),
        source: io::Error::from(io::ErrorKind::StorageFull),
    }));
    let french = lines(Language::French, &full);
    assert_eq!(french[0], "nuit/video_092.mp4\u{a0}: le disque est plein");
    assert_eq!(
        french[1],
        "  à faire\u{a0}: libérer de la place sur le disque de out/video_092, puis relancer"
    );
}

#[test]
fn too_many_tracks_says_what_to_look_at_without_judging_the_tracks_and_without_detail() {
    let too_many = too_many(false);
    assert_eq!(
        lines(Language::French, &too_many),
        [
            "nuit/video_092.mp4\u{a0}: 301 pistes, au-delà de 300\u{a0}: extraits non écrits",
            "  à faire\u{a0}: regarder l'image résumé\u{a0}; si ce sont de vrais passages, relancer avec un --max-tracks plus grand",
        ]
    );
    assert_eq!(
        lines(Language::English, &too_many)[1],
        "  what to do: look at the summary image; if these are real passes, run again with a larger --max-tracks"
    );
}

#[test]
fn a_panic_is_an_internal_error_whose_detail_gives_its_message_and_its_place() {
    let panic = Failure::Panic(Panic {
        message: "index out of bounds".to_owned(),
        location: Some("crates/render/src/clips.rs:12:5".to_owned()),
    });
    assert_eq!(
        lines(Language::French, &panic),
        [
            "nuit/video_092.mp4\u{a0}: erreur interne",
            "  à faire\u{a0}: le signaler avec le détail ci-dessous",
            "  détail\u{a0}: index out of bounds (crates/render/src/clips.rs:12:5)",
        ]
    );
    assert_eq!(
        lines(Language::English, &panic)[0],
        "nuit/video_092.mp4: internal error"
    );
}

#[test]
fn every_cause_has_its_own_sentence_and_its_own_advice_in_both_languages() {
    let causes = [
        Cause::Missing,
        Cause::ReadRefused,
        Cause::NotAVideo,
        Cause::UnusableName,
        Cause::Damaged,
        Cause::Unsupported,
        Cause::Settings,
        Cause::DiskFull,
        Cause::Forbidden,
        Cause::InUse {
            file: PathBuf::from("v_resume.png"),
        },
        Cause::Encoding,
    ];
    for language in [Language::French, Language::English] {
        let phrase = |cause| {
            said(
                language,
                &Message::Failure {
                    video: "v.mp4".to_owned(),
                    cause,
                    place: "out".to_owned(),
                },
            )
        };
        let advice = |cause| {
            said(
                language,
                &Message::FailureTodo {
                    cause,
                    folder: "out".to_owned(),
                },
            )
        };
        let (fallback, fallback_advice) = (phrase(Cause::Unexpected.key()), advice(Cause::Unexpected.key()));
        for cause in &causes {
            assert_ne!(phrase(cause.key()), fallback, "{language:?} {cause:?}");
            assert_ne!(advice(cause.key()), fallback_advice, "{language:?} {cause:?}");
        }
        let too_many = Cause::TooManyTracks {
            count: 2,
            limit: 1,
            annotated: false,
        }
        .key();
        assert_ne!(advice(too_many), fallback_advice, "{language:?}");
    }
}

#[test]
fn too_many_tracks_names_the_video_the_count_and_the_limit_in_the_language_of_the_user() {
    assert_eq!(
        lines(Language::French, &too_many(false))[0],
        "nuit/video_092.mp4\u{a0}: 301 pistes, au-delà de 300\u{a0}: extraits non écrits"
    );
    assert_eq!(
        lines(Language::English, &too_many(false))[0],
        "nuit/video_092.mp4: 301 tracks, above 300: clips skipped"
    );
    assert_eq!(
        lines(Language::French, &too_many(true))[0],
        "nuit/video_092.mp4\u{a0}: 301 pistes, au-delà de 300\u{a0}: extraits et vidéo annotée non écrits"
    );
    assert_eq!(
        lines(Language::English, &too_many(true))[0],
        "nuit/video_092.mp4: 301 tracks, above 300: clips and annotated video skipped"
    );
}

#[test]
fn the_clips_written_again_are_named_with_the_cause_of_the_videotoolbox_failure() {
    let cause = Layer("the encoder failed", Some(Box::new(Layer("Invalid data", None))));
    let one = [PathBuf::from("out/video_092/extraits/piste_03_0m51s.mp4")];
    let two = [
        one[0].clone(),
        PathBuf::from("out/video_092/extraits/piste_04_0m53s.mp4"),
    ];
    assert_eq!(
        said(Language::French, &redone(&job(), &one, &cause)),
        "nuit/video_092.mp4\u{a0}: piste_03_0m51s.mp4 réécrit avec openh264 après une erreur de VideoToolbox\u{a0}: \
         the encoder failed: Invalid data"
    );
    assert_eq!(
        said(Language::French, &redone(&job(), &two, &cause)),
        "nuit/video_092.mp4\u{a0}: piste_03_0m51s.mp4, piste_04_0m53s.mp4 réécrits avec openh264 après une erreur de \
         VideoToolbox\u{a0}: the encoder failed: Invalid data"
    );
    assert_eq!(
        said(Language::English, &redone(&job(), &one, &cause)),
        "nuit/video_092.mp4: piste_03_0m51s.mp4 written again with openh264 after a VideoToolbox error: \
         the encoder failed: Invalid data"
    );
}

#[test]
fn the_line_of_a_video_names_its_rank_its_name_without_extension_and_its_step() {
    let steps: Vec<String> = Step::ALL
        .into_iter()
        .map(|step| said(Language::French, &video_step(THIRD, &job(), step)))
        .collect();
    assert_eq!(
        steps,
        [
            "[3/12] video_092 — détection",
            "[3/12] video_092 — suivi",
            "[3/12] video_092 — image résumé",
            "[3/12] video_092 — extraits",
        ]
    );
    let steps: Vec<String> = Step::ALL
        .into_iter()
        .map(|step| said(Language::English, &video_step(THIRD, &job(), step)))
        .collect();
    assert_eq!(
        steps,
        [
            "[3/12] video_092 — detection",
            "[3/12] video_092 — tracking",
            "[3/12] video_092 — summary image",
            "[3/12] video_092 — clips",
        ]
    );
}

#[test]
fn the_line_of_a_finished_video_keeps_its_track_count_or_says_it_failed() {
    assert_eq!(
        said(Language::French, &video_done(THIRD, &job(), 14)),
        "[3/12] video_092 — 14 pistes"
    );
    assert_eq!(
        said(Language::French, &video_done(THIRD, &job(), 1)),
        "[3/12] video_092 — 1 piste"
    );
    assert_eq!(
        said(Language::French, &video_done(THIRD, &job(), 0)),
        "[3/12] video_092 — 0 piste"
    );
    assert_eq!(
        said(Language::English, &video_done(THIRD, &job(), 0)),
        "[3/12] video_092 — 0 tracks"
    );
    assert_eq!(
        said(Language::English, &video_done(THIRD, &job(), 1)),
        "[3/12] video_092 — 1 track"
    );
    assert_eq!(
        said(Language::French, &video_failed(THIRD, &job(), &damaged())),
        "[3/12] video_092 — échec"
    );
    assert_eq!(
        said(Language::English, &video_failed(THIRD, &job(), &damaged())),
        "[3/12] video_092 — failed"
    );
}

#[test]
fn the_line_of_a_video_with_too_many_tracks_ends_with_the_word_of_the_summary_and_not_with_failed() {
    assert_eq!(
        said(Language::French, &video_failed(THIRD, &job(), &too_many(false))),
        "[3/12] video_092 — trop de pistes"
    );
    assert_eq!(
        said(Language::English, &video_failed(THIRD, &job(), &too_many(true))),
        "[3/12] video_092 — too many tracks"
    );
}

fn refusal(language: Language, error: &jobs::Error) -> Vec<String> {
    let texts = Texts::new(language);
    let (heading, lines) = inputs_refused(texts, error);
    [texts.say(&heading)].into_iter().chain(lines).collect()
}

#[test]
fn every_reason_to_refuse_an_input_has_its_own_words_in_both_languages() {
    let refused = |path: &str, reason: Reason| Refused {
        path: PathBuf::from(path),
        reason,
    };
    let error = jobs::Error::Refused {
        refused: vec![
            refused("absente.mp4", Reason::Missing),
            refused("README.md", Reason::NotAVideo),
            refused("vide", Reason::NoVideo),
            refused("fermee", Reason::Unreadable(Access::Refused)),
            refused("x.mp4/nuit", Reason::Unreadable(Access::NotAFolder)),
            refused("partie", Reason::Unreadable(Access::Missing)),
            refused("reseau", Reason::Unreadable(Access::Unavailable)),
            refused(
                "out/night_x_mp4",
                Reason::SharedFolder {
                    videos: vec![PathBuf::from("a/night/x.mp4"), PathBuf::from("b/night/x.mp4")],
                },
            ),
        ],
        valid: Vec::new(),
    };
    assert_eq!(
        refusal(Language::French, &error),
        [
            "aucune vidéo n'est traitée\u{a0}:",
            "absente.mp4\u{a0}: introuvable",
            "README.md\u{a0}: pas une vidéo (mp4, mov, avi, mkv ou m4v)",
            "vide\u{a0}: aucune vidéo (mp4, mov, avi, mkv ou m4v) dans ce dossier",
            "fermee\u{a0}: lecture refusée",
            "x.mp4/nuit\u{a0}: un élément du chemin n'est pas un dossier",
            "partie\u{a0}: introuvable",
            "reseau\u{a0}: inaccessible",
            "out/night_x_mp4\u{a0}: dossier de résultats commun à a/night/x.mp4, b/night/x.mp4",
        ]
    );
    assert_eq!(
        refusal(Language::English, &error),
        [
            "no video is processed:",
            "absente.mp4: not found",
            "README.md: not a video (mp4, mov, avi, mkv or m4v)",
            "vide: no video (mp4, mov, avi, mkv or m4v) in this folder",
            "fermee: reading refused",
            "x.mp4/nuit: part of the path is not a folder",
            "partie: not found",
            "reseau: unreachable",
            "out/night_x_mp4: results folder shared by a/night/x.mp4, b/night/x.mp4",
        ]
    );
}

#[test]
fn folders_without_any_video_are_listed_under_the_extensions_looked_for_in_both_languages() {
    let one = jobs::Error::NoVideo {
        folders: vec![PathBuf::from("nuit-12-oct")],
    };
    let two = jobs::Error::NoVideo {
        folders: vec![PathBuf::from("nuit-12-oct"), PathBuf::from("nuit-13-oct")],
    };
    assert_eq!(
        refusal(Language::French, &one),
        [
            "aucune vidéo (mp4, mov, avi, mkv ou m4v) trouvée dans ce dossier\u{a0}:",
            "nuit-12-oct"
        ]
    );
    assert_eq!(
        refusal(Language::French, &two),
        [
            "aucune vidéo (mp4, mov, avi, mkv ou m4v) trouvée dans ces dossiers\u{a0}:",
            "nuit-12-oct",
            "nuit-13-oct"
        ]
    );
    assert_eq!(
        refusal(Language::English, &one),
        [
            "no video (mp4, mov, avi, mkv or m4v) found in this folder:",
            "nuit-12-oct"
        ]
    );
    assert_eq!(
        refusal(Language::English, &two)[0],
        "no video (mp4, mov, avi, mkv or m4v) found in these folders:"
    );
}

fn both(message: &Message) -> [String; 2] {
    [said(Language::French, message), said(Language::English, message)]
}

#[test]
fn a_refusal_that_the_command_line_cannot_reach_names_its_option_and_asks_for_a_report() {
    assert_eq!(
        both(&option_refused("max-tracks")),
        [
            "--max-tracks ne peut pas s'appliquer\u{a0}; le signaler",
            "--max-tracks cannot be applied; please report it"
        ]
    );
    let pad = tracking::stability::StabilityConfig::new(None, -1.0).unwrap_err();
    assert_eq!(
        both(&tracking_refused("max-blobs", &pad)),
        [
            "--max-blobs ne peut pas s'appliquer\u{a0}; le signaler",
            "--max-blobs cannot be applied; please report it"
        ]
    );
}

#[test]
fn a_refused_default_without_an_option_asks_for_a_report_without_an_english_cause() {
    let refused = [
        "les réglages par défaut sont refusés\u{a0}; le signaler",
        "the default settings are refused; please report it",
    ];
    let areas = engine::settings::Error::Areas {
        min_area: 9.0,
        max_area: 4.0,
    };
    assert_eq!(both(&detection_refused(&areas)), refused);
    let trail = render::config::RenderConfig::new(12, -1.0, 1.5, 300).unwrap_err();
    assert_eq!(both(&render_refused(&trail)), refused);
}

#[test]
fn a_refused_clip_margin_is_recognised_by_its_setting_and_not_by_its_name() {
    let margin = render::config::RenderConfig::new(12, 1.0, -2.0, 300).unwrap_err();
    assert_eq!(
        both(&render_refused(&margin)),
        [
            "--clip-margin doit être un nombre fini de secondes, au moins 0, pas -2",
            "--clip-margin must be a finite number of seconds, at least 0, not -2"
        ]
    );
}

#[test]
fn every_folder_where_nothing_can_be_written_has_its_own_line_and_reason_in_both_languages() {
    let folder = |path: &str, access| Unwritable {
        folder: PathBuf::from(path),
        access,
    };
    let refused = [
        folder("a", Access::Refused),
        folder("b", Access::NotAFolder),
        folder("c", Access::Missing),
        folder("d", Access::ReadOnly),
        folder("e", Access::DiskFull),
        folder("f", Access::Unavailable),
    ];
    let lines = |language| {
        let texts = Texts::new(language);
        let (heading, lines) = folders_refused(texts, &refused);
        [texts.say(&heading)].into_iter().chain(lines).collect::<Vec<String>>()
    };
    assert_eq!(
        lines(Language::French),
        [
            "aucune vidéo n'est traitée, les résultats ne peuvent pas s'écrire dans ces dossiers\u{a0}:",
            "a\u{a0}: écriture refusée",
            "b\u{a0}: n'est pas un dossier",
            "c\u{a0}: introuvable",
            "d\u{a0}: disque en lecture seule",
            "e\u{a0}: disque plein",
            "f\u{a0}: inaccessible",
        ]
    );
    assert_eq!(
        lines(Language::English),
        [
            "no video is processed, the results cannot be written in these folders:",
            "a: writing refused",
            "b: not a folder",
            "c: not found",
            "d: read-only disk",
            "e: disk full",
            "f: unreachable",
        ]
    );
    let (heading, _) = folders_refused(Texts::new(Language::French), &refused[..1]);
    assert_eq!(
        said(Language::French, &heading),
        "aucune vidéo n'est traitée, les résultats ne peuvent pas s'écrire dans ce dossier\u{a0}:"
    );
    assert_eq!(
        said(Language::English, &heading),
        "no video is processed, the results cannot be written in this folder:"
    );
}

#[test]
fn the_refusals_before_the_question_are_counted_and_the_question_counts_the_valid_videos_in_both_languages() {
    let refused = |path: &str| Refused {
        path: PathBuf::from(path),
        reason: Reason::Missing,
    };
    let headings = |refusals: &[Refused]| {
        [Language::French, Language::English].map(|language| {
            let texts = Texts::new(language);
            let (heading, lines) = inputs_skipped(texts, refusals);
            [texts.say(&heading)].into_iter().chain(lines).collect::<Vec<String>>()
        })
    };
    assert_eq!(
        headings(&[refused("absente.mp4"), refused("autre.mp4")]),
        [
            vec![
                "2 entrées sont refusées\u{a0}:".to_owned(),
                "absente.mp4\u{a0}: introuvable".to_owned(),
                "autre.mp4\u{a0}: introuvable".to_owned()
            ],
            vec![
                "2 inputs are refused:".to_owned(),
                "absente.mp4: not found".to_owned(),
                "autre.mp4: not found".to_owned()
            ],
        ]
    );
    assert_eq!(
        headings(&[refused("absente.mp4")]).map(|lines| lines[0].clone()),
        ["une entrée est refusée\u{a0}:", "one input is refused:"]
    );
    assert_eq!(
        both(&Message::SkipRefusedQuestion { count: 1 }),
        [
            "Traiter quand même la vidéo valide\u{a0}? (O/n)",
            "Process the valid video anyway? (Y/n)"
        ]
    );
    assert_eq!(
        both(&Message::SkipRefusedQuestion { count: 3 }),
        [
            "Traiter quand même les 3 vidéos valides\u{a0}? (O/n)",
            "Process the 3 valid videos anyway? (Y/n)"
        ]
    );
    assert_eq!(
        both(&Message::Fixed(Fixed::SkipRefusedDeclined)),
        ["aucune vidéo n'est traitée", "no video is processed"]
    );
    assert_eq!(
        both(&Message::Fixed(Fixed::SkipRefusedHelp)),
        [
            "Traite les vidéos valides sans poser la question quand des entrées sont refusées",
            "Processes the valid videos without asking when some inputs are refused"
        ]
    );
}
