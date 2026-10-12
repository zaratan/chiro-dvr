use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use std::num::NonZeroU32;

use batdetect::jobs::Job;
use draw::style::style_for;
use draw::text::Typeface;
use summary::header::{HeaderLine, Labels};
use summary::legend::{MAX_HEADER_COLUMNS, column_width, fit_text};
use types::language::Language;

use super::{SummaryLabels, failed, processed};

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

fn texts(language: Language, lines: &[HeaderLine]) -> Vec<String> {
    lines.iter().map(|line| SummaryLabels(language).line(line)).collect()
}

fn clocks(start: &str, end: &str) -> (String, String) {
    (start.to_owned(), end.to_owned())
}

fn every_kind_of_line() -> Vec<HeaderLine> {
    let (start, end) = clocks("0:00", "5:00");
    let (from, to) = clocks("0:00", "0:09");
    vec![
        HeaderLine::Name("vidéo_092".to_owned()),
        HeaderLine::Tracks(0),
        HeaderLine::Tracks(1),
        HeaderLine::Tracks(2),
        HeaderLine::Window { start, end },
        HeaderLine::NotAnalysed { start: from, end: to },
        HeaderLine::More(1),
        HeaderLine::More(2),
        HeaderLine::Unreadable {
            duration: "1 s".to_owned(),
            spans: 1,
        },
        HeaderLine::Unreadable {
            duration: "1 min 18 s".to_owned(),
            spans: 2,
        },
    ]
}

#[test]
fn the_summary_header_keeps_the_french_wording_of_v0_1_1() {
    assert_eq!(
        texts(Language::French, &every_kind_of_line()),
        [
            "vidéo_092",
            "0 piste",
            "1 piste",
            "2 pistes",
            "0:00 - 5:00",
            "hors analyse 0:00 - 0:09",
            "+ 1 autre",
            "+ 2 autres",
            "illisible 1 s (1 plage)",
            "illisible 1 min 18 s (2 plages)",
        ]
    );
}

#[test]
fn the_summary_header_keeps_the_english_wording_of_v0_1_1() {
    assert_eq!(
        texts(Language::English, &every_kind_of_line()),
        [
            "vidéo_092",
            "0 tracks",
            "1 track",
            "2 tracks",
            "0:00 - 5:00",
            "not analysed 0:00 - 0:09",
            "+ 1 more",
            "+ 2 more",
            "unreadable 1 s (1 span)",
            "unreadable 1 min 18 s (2 spans)",
        ]
    );
}

#[test]
fn the_longest_damage_line_is_never_cut_on_the_summary() {
    let typeface = Typeface::roboto().unwrap();
    let longest = HeaderLine::Unreadable {
        duration: "10 min 0 s".to_owned(),
        spans: 600,
    };
    for language in [Language::French, Language::English] {
        let line = SummaryLabels(language).line(&longest);
        for width in [480, 640, 1440, 3840] {
            let style = style_for(NonZeroU32::new(width).unwrap(), "99", &typeface);
            let room = MAX_HEADER_COLUMNS * column_width(&typeface, &style, &[]) - 2.0 * style.margin;
            assert_eq!(fit_text(&typeface, &line, room, &style), line, "{width}");
        }
    }
}
