use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use batdetect::jobs::Job;
use files::previous::Kept;
use types::language::Language;

use super::{failed, internal_error, processed, redone, table_kept, too_many_tracks};
use crate::texts::{Message, Texts};

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

#[test]
fn french_counts_zero_and_one_track_in_the_singular() {
    assert_eq!(
        said(Language::French, &processed(&job(), 0)),
        "video_092.mp4 : 0 piste -> out/video_092"
    );
    assert_eq!(
        said(Language::French, &processed(&job(), 1)),
        "video_092.mp4 : 1 piste -> out/video_092"
    );
    assert_eq!(
        said(Language::French, &processed(&job(), 14)),
        "video_092.mp4 : 14 pistes -> out/video_092"
    );
}

#[test]
fn english_counts_only_one_track_in_the_singular() {
    assert_eq!(
        said(Language::English, &processed(&job(), 0)),
        "video_092.mp4: 0 tracks -> out/video_092"
    );
    assert_eq!(
        said(Language::English, &processed(&job(), 1)),
        "video_092.mp4: 1 track -> out/video_092"
    );
    assert_eq!(
        said(Language::English, &processed(&job(), 3)),
        "video_092.mp4: 3 tracks -> out/video_092"
    );
}

#[test]
fn a_failure_names_the_video_then_says_it_failed_before_every_cause_left_in_english() {
    let error = Layer("the detection failed", Some(Box::new(Layer("cannot open", None))));
    assert_eq!(
        said(Language::English, &failed(&job(), &error)),
        "nuit/video_092.mp4: failed: the detection failed: cannot open"
    );
    assert_eq!(
        said(Language::French, &failed(&job(), &error)),
        "nuit/video_092.mp4 : échec : the detection failed: cannot open"
    );
}

#[test]
fn too_many_tracks_names_the_video_the_count_and_the_limit_in_the_language_of_the_user() {
    assert_eq!(
        said(Language::French, &too_many_tracks(&job(), 301, 300, false)),
        "nuit/video_092.mp4 : 301 pistes, au-delà de 300 : extraits non écrits"
    );
    assert_eq!(
        said(Language::English, &too_many_tracks(&job(), 301, 300, false)),
        "nuit/video_092.mp4: 301 tracks, above 300: clips skipped"
    );
    assert_eq!(
        said(Language::French, &too_many_tracks(&job(), 301, 300, true)),
        "nuit/video_092.mp4 : 301 pistes, au-delà de 300 : extraits et vidéo annotée non écrits"
    );
    assert_eq!(
        said(Language::English, &too_many_tracks(&job(), 301, 300, true)),
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
fn a_panic_is_told_as_an_internal_error_of_the_video_with_its_message() {
    assert_eq!(
        said(Language::English, &internal_error(&job(), "index out of bounds")),
        "nuit/video_092.mp4: internal error: index out of bounds"
    );
    assert_eq!(
        said(Language::French, &internal_error(&job(), "index out of bounds")),
        "nuit/video_092.mp4 : erreur interne : index out of bounds"
    );
}

#[test]
fn a_table_left_as_it_is_is_named_and_says_when_the_new_table_was_not_written() {
    let kept = |ours| Kept {
        path: PathBuf::from("out/video_092/video_092_pistes.csv"),
        ours,
    };
    assert_eq!(
        said(Language::French, &table_kept(&job(), &kept(false))),
        "nuit/video_092.mp4 : video_092_pistes.csv laissé tel quel, il n'a pas la forme d'un tableau des pistes"
    );
    assert_eq!(
        said(Language::French, &table_kept(&job(), &kept(true))),
        "nuit/video_092.mp4 : video_092_pistes.csv laissé tel quel, il n'a pas la forme d'un tableau des pistes ; \
         le nouveau tableau n'est pas écrit"
    );
    assert_eq!(
        said(Language::English, &table_kept(&job(), &kept(false))),
        "nuit/video_092.mp4: video_092_pistes.csv left as it is, it does not have the form of a tracks table"
    );
    assert_eq!(
        said(Language::English, &table_kept(&job(), &kept(true))),
        "nuit/video_092.mp4: video_092_pistes.csv left as it is, it does not have the form of a tracks table; \
         the new table is not written"
    );
}
