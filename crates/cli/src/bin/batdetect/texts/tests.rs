use std::collections::BTreeSet;
use std::num::NonZeroU32;

use batdetect::outcome::Rank;
use batdetect::progress::Step;
use draw::style::style_for;
use draw::text::Typeface;
use engine::settings::Bound;
use fluent_bundle::FluentResource;
use summary::header::{Band, HeaderLine, Labels};
use summary::legend::{column_width, fit_text, legend_width};
use types::language::Language;

use super::{ENGLISH_SOURCE, FRENCH_SOURCE, Fixed, Message, Shown, Texts, resolved};
use files::names::RESULTS_FOLDER;

const LANGUAGES: [Language; 2] = [Language::French, Language::English];
const VARIANTS: usize = 58;
const FIXED: [Fixed; 64] = [
    Fixed::BandNotAnalysed,
    Fixed::BandUnreadable,
    Fixed::About,
    Fixed::InputsHelp,
    Fixed::OutDirHelp,
    Fixed::ZoomHelp,
    Fixed::AnnotatedHelp,
    Fixed::MaxTracksHelp,
    Fixed::ClipMarginHelp,
    Fixed::OsdRegionHelp,
    Fixed::HelpHelp,
    Fixed::VersionHelp,
    Fixed::NoRegion,
    Fixed::BesideTheVideos,
    Fixed::ThresholdHelp,
    Fixed::MinHitsHelp,
    Fixed::MaxBlobsHelp,
    Fixed::WorkWidthHelp,
    Fixed::ValueVideo,
    Fixed::ValueFolder,
    Fixed::ValueWhich,
    Fixed::ValueCount,
    Fixed::ValueSeconds,
    Fixed::ValueLevel,
    Fixed::ValuePixels,
    Fixed::UsageHeading,
    Fixed::ArgumentsHeading,
    Fixed::OptionsHeading,
    Fixed::DetectionHeading,
    Fixed::ErrorLabel,
    Fixed::TipLabel,
    Fixed::MissingArguments,
    Fixed::InvalidUtf8,
    Fixed::ArgumentsRefused,
    Fixed::SettingsRefused,
    Fixed::SkipRefusedHelp,
    Fixed::SkipRefusedDeclined,
    Fixed::NoNotifyHelp,
    Fixed::InputsRefused,
    Fixed::TallyVideo,
    Fixed::TallyLength,
    Fixed::TallyTracks,
    Fixed::TallyNotAnalysed,
    Fixed::TallyCompute,
    Fixed::TallyFailedCell,
    Fixed::TallyTooManyHeading,
    Fixed::CommonUses,
    Fixed::UseNights,
    Fixed::UseVideo,
    Fixed::UseOutDir,
    Fixed::UseZoom,
    Fixed::UseAnnotated,
    Fixed::ExampleNight,
    Fixed::ExampleNextNight,
    Fixed::ExampleResults,
    Fixed::AllOptions,
    Fixed::FileVideo,
    Fixed::FileSummary,
    Fixed::FileTable,
    Fixed::FileClip,
    Fixed::FileZoom,
    Fixed::FileAnnotated,
    Fixed::FileParams,
    Fixed::Guide,
];

fn rank(message: &Message) -> usize {
    match message {
        Message::Fixed(_) => 0,
        Message::TallyTitle { .. } => 1,
        Message::TooManyTracks { .. } => 2,
        Message::ClipsRedone { .. } => 3,
        Message::Failure { .. } => 4,
        Message::FailureTodo { .. } => 5,
        Message::SummaryPart { .. } => 6,
        Message::SummaryTracks { .. } => 7,
        Message::SummaryNotAnalysed { .. } => 8,
        Message::SummaryMore { .. } => 9,
        Message::SummaryUnreadable { .. } => 10,
        Message::SummaryMoreUnreadable { .. } => 11,
        Message::DefaultValue { .. } => 12,
        Message::PossibleValues { .. } => 13,
        Message::UnexpectedArgument { .. } => 14,
        Message::SimilarArgument { .. } => 15,
        Message::SimilarValue { .. } => 16,
        Message::ArgumentAsValue { .. } => 17,
        Message::InvalidValue { .. } => 18,
        Message::RegionShape { .. } => 19,
        Message::ValueRequired { .. } => 20,
        Message::UsedTwice { .. } => 21,
        Message::UnexpectedValue { .. } => 22,
        Message::DecimalPoint { .. } => 23,
        Message::MoreInformation { .. } => 24,
        Message::SettingRefused { .. } => 25,
        Message::RegionRefused { .. } => 26,
        Message::ClipMarginRefused { .. } => 27,
        Message::OptionRefused { .. } => 28,
        Message::InputRefused { .. } => 29,
        Message::TallyDone { .. } => 30,
        Message::FoldersRefused { .. } => 31,
        Message::VideoStep { .. } => 32,
        Message::VideoDone { .. } => 33,
        Message::VideoFailed { .. } => 34,
        Message::FailureDetail { .. } => 35,
        Message::TallyTooManyCount { .. } => 36,
        Message::TallyFailedCount { .. } => 37,
        Message::TallyTooManyCell { .. } => 38,
        Message::TallyTooMany { .. } => 39,
        Message::TallyFailedHeading { .. } => 40,
        Message::TallyKeptHeading { .. } => 41,
        Message::TallyKept { .. } => 42,
        Message::TallyChanged { .. } => 43,
        Message::TallyRegions { .. } => 44,
        Message::TallyFailureTodo { .. } => 45,
        Message::UseFolder { .. } => 46,
        Message::FilesHeading { .. } => 47,
        Message::FilesLong { .. } => 48,
        Message::NoVideo { .. } => 49,
        Message::InputWithoutVideo { .. } => 50,
        Message::SharedFolder { .. } => 51,
        Message::FolderRefused { .. } => 52,
        Message::InputsSkipped { .. } => 53,
        Message::SkipRefusedQuestion { .. } => 54,
        Message::TallyRefusedCount { .. } => 55,
        Message::TallyRefusedHeading { .. } => 56,
        Message::Interrupted { .. } => 57,
    }
}

fn fixed_rank(fixed: Fixed) -> usize {
    match fixed {
        Fixed::BandNotAnalysed => 0,
        Fixed::BandUnreadable => 1,
        Fixed::About => 2,
        Fixed::InputsHelp => 3,
        Fixed::OutDirHelp => 4,
        Fixed::ZoomHelp => 5,
        Fixed::AnnotatedHelp => 6,
        Fixed::MaxTracksHelp => 7,
        Fixed::ClipMarginHelp => 8,
        Fixed::OsdRegionHelp => 9,
        Fixed::HelpHelp => 10,
        Fixed::VersionHelp => 11,
        Fixed::NoRegion => 12,
        Fixed::ThresholdHelp => 13,
        Fixed::MinHitsHelp => 14,
        Fixed::MaxBlobsHelp => 15,
        Fixed::WorkWidthHelp => 16,
        Fixed::ValueVideo => 17,
        Fixed::ValueFolder => 18,
        Fixed::ValueWhich => 19,
        Fixed::ValueCount => 20,
        Fixed::ValueSeconds => 21,
        Fixed::ValueLevel => 22,
        Fixed::ValuePixels => 23,
        Fixed::UsageHeading => 24,
        Fixed::ArgumentsHeading => 25,
        Fixed::OptionsHeading => 26,
        Fixed::DetectionHeading => 27,
        Fixed::ErrorLabel => 28,
        Fixed::TipLabel => 29,
        Fixed::MissingArguments => 30,
        Fixed::InvalidUtf8 => 31,
        Fixed::InputsRefused => 32,
        Fixed::BesideTheVideos => 33,
        Fixed::TallyVideo => 34,
        Fixed::TallyLength => 35,
        Fixed::TallyTracks => 36,
        Fixed::TallyNotAnalysed => 37,
        Fixed::TallyCompute => 38,
        Fixed::TallyFailedCell => 39,
        Fixed::TallyTooManyHeading => 40,
        Fixed::CommonUses => 41,
        Fixed::UseNights => 42,
        Fixed::UseVideo => 43,
        Fixed::UseOutDir => 44,
        Fixed::UseZoom => 45,
        Fixed::UseAnnotated => 46,
        Fixed::ExampleNight => 47,
        Fixed::ExampleNextNight => 48,
        Fixed::ExampleResults => 49,
        Fixed::AllOptions => 50,
        Fixed::FileVideo => 51,
        Fixed::FileSummary => 52,
        Fixed::FileTable => 53,
        Fixed::FileClip => 54,
        Fixed::FileZoom => 55,
        Fixed::FileAnnotated => 56,
        Fixed::FileParams => 57,
        Fixed::Guide => 58,
        Fixed::ArgumentsRefused => 59,
        Fixed::SettingsRefused => 60,
        Fixed::SkipRefusedHelp => 61,
        Fixed::SkipRefusedDeclined => 62,
        Fixed::NoNotifyHelp => 63,
    }
}

fn samples() -> Vec<Message> {
    FIXED
        .into_iter()
        .map(Message::Fixed)
        .chain(lines())
        .chain(command_line())
        .collect()
}

fn lines() -> Vec<Message> {
    let text = |value: &str| value.to_owned();
    vec![
        Message::TallyTitle { total: 12 },
        Message::TallyDone { count: 10 },
        Message::TallyTooManyCount { count: 1 },
        Message::TallyFailedCount { count: 1 },
        Message::TallyTooManyCell { count: 412 },
        Message::TallyTooMany {
            video: text("video_099"),
            count: 412,
            limit: 300,
        },
        Message::TallyFailedHeading { failed: 1, total: 12 },
        Message::TallyKeptHeading { count: 1 },
        Message::TallyKept {
            video: text("video_092"),
            table: text("video_092_pistes.csv"),
            written: false,
        },
        Message::TallyChanged {
            settings: text("--threshold 11"),
        },
        Message::TallyRegions { count: 2 },
        Message::TallyFailureTodo { cause: "encoding" },
        Message::UseFolder {
            folder: text("nuit-12-oct/batdetect"),
        },
        Message::FilesHeading {
            folder: text("batdetect"),
        },
        Message::FilesLong {
            period: text("_000m-010m"),
            part: text("_1_sur_3"),
        },
        Message::TooManyTracks {
            video: text("nuit/video_092.mp4"),
            count: 301,
            limit: 300,
            annotated: true,
        },
        Message::ClipsRedone {
            video: text("nuit/video_092.mp4"),
            names: text("piste_03_0m51s.mp4"),
            count: 1,
            causes: text("the encoder failed"),
        },
        Message::VideoStep {
            rank: Rank { position: 3, total: 12 },
            video: text("video_092"),
            step: Step::Summary,
        },
        Message::VideoDone {
            rank: Rank { position: 3, total: 12 },
            video: text("video_092"),
            tracks: 14,
        },
        Message::Interrupted {
            rank: Rank { position: 3, total: 12 },
        },
        Message::VideoFailed {
            rank: Rank { position: 3, total: 12 },
            video: text("video_092"),
            too_many: true,
        },
        Message::Failure {
            video: text("nuit/video_092.mp4"),
            cause: "in-use",
            place: text("out/video_092/video_092_resume.png"),
        },
        Message::FailureTodo {
            cause: "disk-full",
            folder: text("out/video_092"),
        },
        Message::FailureDetail {
            detail: text("the detection failed"),
        },
        Message::SummaryPart { number: 2, count: 8 },
        Message::SummaryTracks { count: 2 },
        Message::SummaryNotAnalysed {
            start: text("0:00"),
            end: text("0:09"),
        },
        Message::SummaryMore { count: 2 },
        Message::SummaryUnreadable {
            start: text("1:02"),
            end: text("1:05"),
        },
        Message::SummaryMoreUnreadable {
            count: 84,
            duration: text("3 min 22 s"),
            seconds: 202,
        },
    ]
}

fn command_line() -> Vec<Message> {
    let text = |value: &str| value.to_owned();
    vec![
        Message::DefaultValue { value: text("1.5") },
        Message::PossibleValues {
            values: text("auto, all, none"),
        },
        Message::UnexpectedArgument {
            argument: text("--tresh"),
        },
        Message::SimilarArgument {
            suggestion: text("--threshold"),
        },
        Message::SimilarValue {
            suggestion: text("all"),
        },
        Message::ArgumentAsValue {
            argument: text("-x"),
            escaped: text("-- -x"),
        },
        Message::InvalidValue {
            value: text("x"),
            argument: text("--zoom <WHICH>"),
        },
        Message::RegionShape { value: text("0,0,1") },
        Message::ValueRequired {
            argument: text("--max-tracks <COUNT>"),
        },
        Message::UsedTwice {
            argument: text("--out-dir <FOLDER>"),
        },
        Message::UnexpectedValue {
            value: text("oui"),
            argument: text("--annotated"),
        },
        Message::DecimalPoint { written: text("11.5") },
        Message::MoreInformation { flag: text("--help") },
        Message::SettingRefused {
            option: text("--work-width"),
            bound: Bound::AtLeast(16),
            value: Shown::Number(8.0),
        },
        Message::RegionRefused {
            index: 1,
            region: text("0.5,0,0.2,1"),
        },
        Message::ClipMarginRefused {
            value: Shown::Written(text("1,5")),
        },
        Message::OptionRefused {
            option: text("--min-hits"),
        },
        Message::InputRefused {
            path: text("absente.mp4"),
            reason: "missing",
        },
        Message::InputWithoutVideo {
            path: text("nuit-12-oct"),
            folder: true,
            listed: text("mp4, mov, avi, mkv"),
            last: text("m4v"),
        },
        Message::SharedFolder {
            path: text("out/night_x_mp4"),
            videos: text("a/night/x.mp4, b/night/x.mp4"),
        },
        Message::NoVideo {
            count: 2,
            listed: text("mp4, mov, avi, mkv"),
            last: text("m4v"),
        },
        Message::FoldersRefused { count: 2 },
        Message::FolderRefused {
            path: text("nuit/batdetect"),
            access: "read-only",
        },
        Message::InputsSkipped { count: 2 },
        Message::SkipRefusedQuestion { count: 3 },
        Message::TallyRefusedCount { count: 2 },
        Message::TallyRefusedHeading { count: 2 },
    ]
}

fn source(language: Language) -> &'static str {
    match language {
        Language::French => FRENCH_SOURCE,
        Language::English => ENGLISH_SOURCE,
    }
}

fn keys(source: &str) -> BTreeSet<&str> {
    source
        .lines()
        .filter_map(|line| line.split_once(" ="))
        .map(|(key, _)| key)
        .filter(|key| key.starts_with(|first: char| first.is_ascii_alphabetic()))
        .collect()
}

#[test]
fn the_samples_hold_one_message_of_each_rank_written_by_hand() {
    let ranks: BTreeSet<usize> = samples().iter().map(rank).collect();
    assert_eq!(ranks, (0..VARIANTS).collect());
    let fixed: BTreeSet<usize> = FIXED.into_iter().map(fixed_rank).collect();
    assert_eq!(fixed, (0..FIXED.len()).collect());
}

#[test]
fn both_catalogs_are_read_without_an_error() {
    for language in LANGUAGES {
        assert!(
            FluentResource::try_new(source(language).to_owned()).is_ok(),
            "{language:?}"
        );
    }
}

#[test]
fn both_catalogs_are_flat_messages_without_terms_attributes_or_number_functions() {
    for language in LANGUAGES {
        for line in source(language).lines() {
            assert!(!line.starts_with('-'), "{line}");
            assert!(!line.trim_start().starts_with('.'), "{line}");
            assert!(!line.contains("NUMBER("), "{line}");
        }
    }
}

#[test]
fn every_message_reads_in_both_languages_with_the_arguments_it_is_given() {
    for language in LANGUAGES {
        for message in samples() {
            assert!(resolved(language, &message).is_ok(), "{language:?} {}", message.id());
        }
    }
}

#[test]
fn every_argument_of_a_message_is_used_by_at_least_one_language() {
    for message in samples() {
        let arguments = message.arguments();
        for (name, _) in arguments.iter() {
            let without: fluent_bundle::FluentArgs<'_> = arguments
                .iter()
                .filter(|(other, _)| *other != name)
                .map(|(other, value)| (other, value.clone()))
                .collect();
            let missed = LANGUAGES.iter().any(|&language| {
                let bundle = super::bundle(language);
                let pattern = bundle
                    .get_message(message.id())
                    .and_then(|found| found.value())
                    .unwrap();
                let mut errors = Vec::new();
                bundle.format_pattern(pattern, Some(&without), &mut errors);
                !errors.is_empty()
            });
            assert!(missed, "{} {name}", message.id());
        }
    }
}

#[test]
fn every_key_of_both_catalogs_belongs_to_a_message() {
    let ids: BTreeSet<&str> = samples().iter().map(Message::id).collect();
    for language in LANGUAGES {
        assert_eq!(keys(source(language)), ids, "{language:?}");
    }
}

#[test]
fn no_isolation_mark_surrounds_the_arguments() {
    for language in LANGUAGES {
        for message in samples() {
            let text = Texts::new(language).say(&message);
            assert!(!text.contains(['\u{2068}', '\u{2069}']), "{text}");
        }
    }
}

#[test]
fn a_french_count_of_thousands_is_grouped_with_a_narrow_no_break_space_on_the_console() {
    let message = Message::TallyTooMany {
        video: "video_092".to_owned(),
        count: 1234,
        limit: 300,
    };
    assert_eq!(
        Texts::new(Language::French).say(&message),
        "video_092\u{a0}: 1\u{202f}234 pistes, au-delà de 300"
    );
    assert_eq!(
        Texts::new(Language::English).say(&message),
        "video_092: 1234 tracks, above 300"
    );
}

#[test]
fn a_french_count_of_thousands_is_grouped_with_a_no_break_space_on_the_summary_that_roboto_can_draw() {
    assert_eq!(
        Texts::new(Language::French).line(&HeaderLine::Tracks(1234)),
        "1\u{a0}234 pistes"
    );
    assert_eq!(
        Texts::new(Language::English).line(&HeaderLine::Tracks(1234)),
        "1234 tracks"
    );
}

fn texts(language: Language, lines: &[HeaderLine]) -> Vec<String> {
    lines.iter().map(|line| Texts::new(language).line(line)).collect()
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
            start: "1:02".to_owned(),
            end: "1:05".to_owned(),
        },
    ]
}

#[test]
fn the_summary_header_reads_in_french_with_the_wording_of_v0_1_1_where_it_existed() {
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
            "illisible 1:02 - 1:05",
        ]
    );
}

#[test]
fn the_summary_header_reads_in_english_with_the_wording_of_v0_1_1_where_it_existed() {
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
            "unreadable 1:02 - 1:05",
        ]
    );
}

#[test]
fn the_longest_unreadable_total_is_never_cut_on_the_summary() {
    let typeface = Typeface::roboto().unwrap();
    let longest = HeaderLine::MoreUnreadable {
        count: 597,
        total_seconds: 600,
    };
    for language in [Language::French, Language::English] {
        let header = ["video_121".to_owned(), Texts::new(language).line(&longest)];
        for width in [480, 640, 1440, 3840] {
            let style = style_for(NonZeroU32::new(width).unwrap(), "99", &typeface);
            let column = column_width(&typeface, &style, &[]);
            let room =
                legend_width(&typeface, &header, column + style.margin, column, &style) as f32 - 2.0 * style.margin;
            assert_eq!(fit_text(&typeface, &header[1], room, &style), header[1], "{width}");
        }
    }
}

#[test]
fn the_time_strip_names_its_two_bands_in_each_language() {
    let bands = |language| [Band::NotAnalysed, Band::Unreadable].map(|band| Texts::new(language).band(band));
    assert_eq!(bands(Language::French), ["hors analyse", "illisible"]);
    assert_eq!(bands(Language::English), ["not analysed", "unreadable"]);
}

#[test]
fn the_hidden_unreadable_spans_carry_the_total_unreadable_time() {
    let lines = [
        HeaderLine::MoreUnreadable {
            count: 84,
            total_seconds: 202,
        },
        HeaderLine::MoreUnreadable {
            count: 1,
            total_seconds: 1,
        },
    ];
    assert_eq!(
        texts(Language::French, &lines),
        [
            "+ 84 autres, 3 min 22 s illisibles en tout",
            "+ 1 autre, 1 s illisible en tout",
        ]
    );
    assert_eq!(
        texts(Language::English, &lines),
        [
            "+ 84 more, 3 min 22 s unreadable in all",
            "+ 1 more, 1 s unreadable in all",
        ]
    );
}

#[test]
fn a_part_of_a_crowded_summary_says_which_one_it_is_in_each_language() {
    let part = [HeaderLine::Part(files::names::Part { number: 2, count: 8 })];
    assert_eq!(texts(Language::French, &part), ["partie 2 / 8"]);
    assert_eq!(texts(Language::English, &part), ["part 2 / 8"]);
}

#[test]
fn a_message_that_misses_an_argument_still_says_what_it_can() {
    let message = Message::Failure {
        video: "nuit/video_092.mp4".to_owned(),
        cause: "forbidden",
        place: String::new(),
    };
    let said = Texts::new(Language::French).say(&message);
    assert!(
        said.starts_with("nuit/video_092.mp4\u{a0}: écriture refusée dans "),
        "{said}"
    );
}

#[test]
fn a_refused_number_of_any_kind_is_written_without_a_panic_in_both_languages() {
    for value in [5e-324, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.0, 1e300] {
        for message in [
            Message::ClipMarginRefused {
                value: Shown::Number(value),
            },
            Message::SettingRefused {
                option: "--threshold".to_owned(),
                bound: Bound::Positive,
                value: Shown::Number(value),
            },
        ] {
            for language in LANGUAGES {
                assert!(resolved(language, &message).is_ok(), "{language:?} {value}");
            }
        }
    }
}

#[test]
fn the_english_refusal_of_a_setting_reads_as_the_engine_writes_it_for_every_bound() {
    for bound in [
        Bound::Positive,
        Bound::NotNegative,
        Bound::AtLeast(16),
        Bound::UpTo(255),
    ] {
        let engine = engine::settings::Error::Setting {
            setting: engine::settings::Setting::WorkWidth,
            value: -1.5,
            bound,
        };
        let message = Message::SettingRefused {
            option: "work_width".to_owned(),
            bound,
            value: Shown::Number(-1.5),
        };
        assert_eq!(Texts::new(Language::English).say(&message), engine.to_string());
    }
}

#[test]
fn a_number_outside_fluent_takes_the_decimal_mark_of_the_language() {
    assert_eq!(Texts::new(Language::French).number(1.5), "1,5");
    assert_eq!(Texts::new(Language::English).number(1.5), "1.5");
    assert_eq!(Texts::new(Language::French).number(300.0), "300");
}

#[test]
fn every_help_text_and_every_description_of_a_file_produced_starts_with_a_capital_letter_in_both_languages() {
    for language in LANGUAGES {
        for fixed in FIXED.into_iter().filter(|fixed| {
            fixed.id().ends_with("-help") || (fixed.id().starts_with("file-") && *fixed != Fixed::FileVideo)
        }) {
            let help = Texts::new(language).text(fixed);
            assert!(
                help.starts_with(char::is_uppercase),
                "{language:?} {}: {help}",
                fixed.id()
            );
        }
    }
}

#[test]
fn the_common_uses_give_six_commands_that_each_start_with_the_name_of_the_program() {
    for language in LANGUAGES {
        let uses = Texts::new(language).common_uses("prog");
        let commands = uses.matches("\u{1b}[1m\u{1b}[36mprog ").count();
        assert_eq!(commands, 6, "{language:?}: {uses}");
        assert_eq!(
            uses.lines().filter(|line| line.starts_with("  - ")).count(),
            6,
            "{uses}"
        );
        let night = Texts::new(language).text(Fixed::ExampleNight);
        assert!(uses.contains(&format!("{night}/{RESULTS_FOLDER}/")), "{uses}");
    }
}

#[test]
fn the_files_produced_are_named_as_the_command_writes_them_in_each_language() {
    for language in LANGUAGES {
        let texts = Texts::new(language);
        let files = texts.files_produced();
        let names = files::names::Names::new(language);
        let video = texts.text(Fixed::FileVideo);
        let (here, stem) = (std::path::Path::new(""), std::ffi::OsStr::new(&video));
        let expected = [
            names.summary_png(here, stem, None, None).display().to_string(),
            names.tracks_csv(here, stem).display().to_string(),
            format!("{}/{}", names.clips_dir(here).display(), names.numbered_clip(1, 4.0, 1)),
            format!(
                "{}/{}",
                names.clips_dir(here).display(),
                names.numbered_zoomed_clip(1, 4.0, 1)
            ),
            names.annotated_video(here, stem).display().to_string(),
            "params.json".to_owned(),
        ];
        for name in expected {
            assert!(
                files.contains(&format!("\u{1b}[33m{name}\u{1b}[0m")),
                "{language:?} {name}: {files}"
            );
        }
    }
}
