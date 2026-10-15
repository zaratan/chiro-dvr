use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

use batdetect::failure::Cause;
use batdetect::jobs::Job;
use batdetect::options::Options;
use batdetect::outcome::Rank;
use batdetect::outcome::{Failure, Panic};
use batdetect::process::{self, Extent};
use batdetect::progress::Step;
use engine::detection;
use engine::input;
use engine::lost::Losses;
use types::language::Language;

use super::{failure_lines, named, redone, video_done, video_failed, video_step};
use crate::texts::{Message, Texts};

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
    assert_eq!(french[0], "nuit/video_092.mp4 : la vidéo est abîmée ou incomplète");
    assert!(french[1].starts_with("  à faire : la recopier depuis les jumelles par le câble USB"));
    assert!(
        french[2].starts_with("  détail : the detection failed: nuit/video_092.mp4: decoded 20 frames"),
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
    assert_eq!(french[0], "nuit/video_092.mp4 : le disque est plein");
    assert_eq!(
        french[1],
        "  à faire : libérer de la place sur le disque de out/video_092, puis relancer"
    );
}

#[test]
fn too_many_tracks_says_what_to_look_at_without_judging_the_tracks_and_without_detail() {
    let too_many = too_many(false);
    assert_eq!(
        lines(Language::French, &too_many),
        [
            "nuit/video_092.mp4 : 301 pistes, au-delà de 300 : extraits non écrits",
            "  à faire : regarder l'image résumé ; si ce sont de vrais passages, relancer avec un --max-tracks plus grand",
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
            "nuit/video_092.mp4 : erreur interne",
            "  à faire : le signaler avec le détail ci-dessous",
            "  détail : index out of bounds (crates/render/src/clips.rs:12:5)",
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
        let (fallback, fallback_advice) = (phrase(named(&Cause::Unexpected)), advice(named(&Cause::Unexpected)));
        for cause in &causes {
            assert_ne!(phrase(named(cause)), fallback, "{language:?} {cause:?}");
            assert_ne!(advice(named(cause)), fallback_advice, "{language:?} {cause:?}");
        }
        let too_many = named(&Cause::TooManyTracks {
            count: 2,
            limit: 1,
            annotated: false,
        });
        assert_ne!(advice(too_many), fallback_advice, "{language:?}");
    }
}

#[test]
fn too_many_tracks_names_the_video_the_count_and_the_limit_in_the_language_of_the_user() {
    assert_eq!(
        lines(Language::French, &too_many(false))[0],
        "nuit/video_092.mp4 : 301 pistes, au-delà de 300 : extraits non écrits"
    );
    assert_eq!(
        lines(Language::English, &too_many(false))[0],
        "nuit/video_092.mp4: 301 tracks, above 300: clips skipped"
    );
    assert_eq!(
        lines(Language::French, &too_many(true))[0],
        "nuit/video_092.mp4 : 301 pistes, au-delà de 300 : extraits et vidéo annotée non écrits"
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
        "nuit/video_092.mp4 : piste_03_0m51s.mp4 réécrit avec openh264 après une erreur de VideoToolbox : \
         the encoder failed: Invalid data"
    );
    assert_eq!(
        said(Language::French, &redone(&job(), &two, &cause)),
        "nuit/video_092.mp4 : piste_03_0m51s.mp4, piste_04_0m53s.mp4 réécrits avec openh264 après une erreur de \
         VideoToolbox : the encoder failed: Invalid data"
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
        said(Language::French, &video_failed(THIRD, &job())),
        "[3/12] video_092 — échec"
    );
    assert_eq!(
        said(Language::English, &video_failed(THIRD, &job())),
        "[3/12] video_092 — failed"
    );
}
