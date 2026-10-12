use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use batdetect::jobs::Job;
use types::language::Language;

use super::{failed, processed};

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
        processed(Language::French, &job(), 0),
        "video_092.mp4 : 0 piste -> out/video_092"
    );
    assert_eq!(
        processed(Language::French, &job(), 1),
        "video_092.mp4 : 1 piste -> out/video_092"
    );
    assert_eq!(
        processed(Language::French, &job(), 14),
        "video_092.mp4 : 14 pistes -> out/video_092"
    );
}

#[test]
fn english_counts_only_one_track_in_the_singular() {
    assert_eq!(
        processed(Language::English, &job(), 0),
        "video_092.mp4: 0 tracks -> out/video_092"
    );
    assert_eq!(
        processed(Language::English, &job(), 1),
        "video_092.mp4: 1 track -> out/video_092"
    );
    assert_eq!(
        processed(Language::English, &job(), 3),
        "video_092.mp4: 3 tracks -> out/video_092"
    );
}

#[test]
fn a_failure_names_the_video_then_every_cause() {
    let error = Layer("the detection failed", Some(Box::new(Layer("cannot open", None))));
    assert_eq!(
        failed(&job(), &error),
        "nuit/video_092.mp4: the detection failed: cannot open"
    );
}
