use std::fs;
use std::num::NonZeroUsize;

use types::frame::FrameIndex;
use types::rational::Rational;

use super::{DECODED_FORMAT, Decoder, DecoderThreads, Losses, PROBED_FORMAT, VideoSummary};
use crate::error::Error;
use crate::location::{Reference, committed};

fn summary_092() -> VideoSummary {
    VideoSummary::read(&committed(Reference::V011, "video_092_original").join("video.json")).unwrap()
}

fn decoded(summary: VideoSummary) -> VideoSummary {
    summary.decoded(
        "v0.2.0",
        Decoder {
            error_frames: vec![FrameIndex::new(1290)],
            gap_frames: vec![FrameIndex::new(1301)],
            losses: Losses {
                before_first: 2,
                after_last: 0,
            },
            threads: DecoderThreads::Frame(NonZeroUsize::new(13).unwrap()),
        },
    )
}

fn written(summary: &VideoSummary) -> (tempfile::TempDir, std::path::PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("video.json");
    summary.write(&path).unwrap();
    (directory, path)
}

#[test]
fn the_stream_of_the_092_reads_as_exact_rationals() {
    let stream = summary_092().stream;
    assert_eq!((stream.width, stream.height), (1440, 1080));
    assert_eq!(stream.avg_frame_rate, Rational::new(90_110_000, 3_000_927).unwrap());
    assert_eq!(stream.r_frame_rate, Rational::new(10_000, 333).unwrap());
    assert_eq!(stream.time_base, Rational::new(1, 10_000).unwrap());
    assert_eq!(stream.nb_frames.as_deref(), Some("9011"));
}

#[test]
fn a_stream_has_a_size_unless_one_side_is_zero() {
    let stream = summary_092().stream;
    let size = stream.size().unwrap();
    assert_eq!((size.width().get(), size.height().get()), (1440, 1080));
    assert_eq!(
        super::Stream {
            width: 0,
            ..stream.clone()
        }
        .size(),
        None
    );
    assert_eq!(super::Stream { height: 0, ..stream }.size(), None);
}

#[test]
fn the_092_summary_reports_no_damage_and_its_counts() {
    let summary = summary_092();
    assert_eq!(summary.ffprobe.as_ref().unwrap().frame_count, 9011);
    assert!(summary.ffprobe.as_ref().unwrap().error_frames.is_empty() && summary.damaged_spans.is_empty());
    assert_eq!(summary.background_half_window, 15);
    assert_eq!((summary.counts.detections, summary.counts.tracks), (780, 14));
}

#[test]
fn sections_without_a_type_are_kept_as_written() {
    let recorded = summary_092().recorded;
    assert_eq!(recorded.len(), 3);
    assert_eq!(recorded["config"]["TrackConfig"]["max_gap"], 6);
    assert_eq!(recorded["versions"]["batdetect"], "0.1.1");
}

#[test]
fn a_summary_survives_a_round_trip() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("video.json");
    let mut summary = summary_092();
    summary.damaged_spans = vec![types::span::Span::new(FrameIndex::new(1286), FrameIndex::new(1328)).unwrap()];
    summary.stream.nb_frames = None;
    summary.write(&path).unwrap();
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.ends_with("}\n") && !text.contains("nb_frames"));
    assert_eq!(VideoSummary::read(&path).unwrap(), summary);
}

#[test]
fn another_format_version_is_refused_before_its_fields_are_read() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("video.json");
    fs::write(&path, "{\"format_version\": 3, \"frames\": []}\n").unwrap();
    let error = VideoSummary::read(&path).unwrap_err();
    assert!(matches!(error, Error::Version { found: 3, .. }));
    assert!(
        error
            .to_string()
            .ends_with("has format version 3, only versions 1 and 2 are read")
    );
}

#[test]
fn a_summary_of_the_decoder_keeps_its_losses_and_threads_and_names_no_table() {
    let summary = decoded(summary_092());
    let (_directory, path) = written(&summary);

    let json: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();

    assert_eq!(
        json["decoder"]["losses"],
        serde_json::json!({"before_first": 2, "after_last": 0})
    );
    assert_eq!(
        json["decoder"]["threads"],
        serde_json::json!({"kind": "frame", "count": 13})
    );
    assert!(json.get("ffprobe").is_none() && json["files"].get("csv").is_none());
    assert_eq!(VideoSummary::read(&path).unwrap(), summary);
}

#[test]
fn a_summary_turned_to_the_decoder_keeps_its_video_and_stream_and_drops_only_what_ffprobe_and_the_table_gave() {
    let probed = summary_092();
    assert_eq!(probed.format_version, PROBED_FORMAT);

    let summary = decoded(probed.clone());

    assert_eq!(
        (summary.format_version, summary.reference.as_str()),
        (DECODED_FORMAT, "v0.2.0")
    );
    assert!(summary.ffprobe.is_none() && summary.files.csv.is_none());
    assert_eq!(
        VideoSummary {
            format_version: PROBED_FORMAT,
            reference: probed.reference.clone(),
            ffprobe: probed.ffprobe.clone(),
            decoder: None,
            files: probed.files.clone(),
            ..summary
        },
        probed
    );
}

#[test]
fn each_kind_of_decoding_threads_reads_back_and_a_single_thread_writes_no_count() {
    let four = NonZeroUsize::new(4).unwrap();
    let written: Vec<(serde_json::Value, DecoderThreads)> = [
        DecoderThreads::Single,
        DecoderThreads::Frame(four),
        DecoderThreads::Slice(four),
    ]
    .into_iter()
    .map(|threads| {
        let mut summary = decoded(summary_092());
        if let Some(decoder) = summary.decoder.as_mut() {
            decoder.threads = threads;
        }
        let (_directory, path) = written(&summary);
        let json: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let read = VideoSummary::read(&path).unwrap().decoder.unwrap().threads;
        (json["decoder"]["threads"].clone(), read)
    })
    .collect();

    assert_eq!(
        written,
        [
            (serde_json::json!({"kind": "single"}), DecoderThreads::Single),
            (
                serde_json::json!({"kind": "frame", "count": 4}),
                DecoderThreads::Frame(four)
            ),
            (
                serde_json::json!({"kind": "slice", "count": 4}),
                DecoderThreads::Slice(four)
            ),
        ]
    );
}

#[test]
fn decoding_threads_with_no_thread_or_of_an_unknown_kind_are_refused_with_their_line() {
    let (_directory, path) = written(&decoded(summary_092()));
    let text = fs::read_to_string(&path).unwrap();
    let frame = "\"kind\": \"frame\",\n      \"count\": 13";
    assert!(text.contains(frame));

    for altered in [
        frame.replace("13", "0"),
        frame.replace("frame", "process"),
        frame.replace("\"kind\": \"frame\",", "\"kind\": \"single\","),
    ] {
        fs::write(&path, text.replace(frame, &altered)).unwrap();
        assert!(
            matches!(VideoSummary::read(&path).unwrap_err(), Error::Json { .. }),
            "{altered}"
        );
    }
}

#[test]
fn a_summary_of_the_decoder_keeps_the_detection_settings_of_the_v011() {
    let (_directory, path) = written(&decoded(summary_092()));

    let areas: Areas = VideoSummary::read(&path).unwrap().detect_settings().unwrap();

    assert_eq!(areas, summary_092().detect_settings::<Areas>().unwrap());
}

fn sections_out_of_their_version() -> [VideoSummary; 6] {
    let without_probe = VideoSummary {
        ffprobe: None,
        ..summary_092()
    };
    let probed_and_decoded = VideoSummary {
        decoder: decoded(summary_092()).decoder,
        ..summary_092()
    };
    let mut probed_without_table = summary_092();
    probed_without_table.files.csv = None;
    let decoded_and_probed = VideoSummary {
        ffprobe: summary_092().ffprobe,
        ..decoded(summary_092())
    };
    let mut decoded_with_a_table = decoded(summary_092());
    decoded_with_a_table.files.csv = Some("tracks.csv".to_owned());
    let mut without_decoder = decoded(summary_092());
    without_decoder.decoder = None;
    [
        without_probe,
        probed_and_decoded,
        probed_without_table,
        decoded_and_probed,
        decoded_with_a_table,
        without_decoder,
    ]
}

fn refusal(error: &Error) -> String {
    assert!(matches!(
        error,
        Error::MissingSection { .. } | Error::ForeignSection { .. }
    ));
    error.to_string().rsplit(": ").next().unwrap().to_owned()
}

const REFUSALS: [&str; 6] = [
    "format version 1 needs the ffprobe section",
    "format version 1 has no decoder section",
    "format version 1 needs the files.csv section",
    "format version 2 has no ffprobe section",
    "format version 2 has no files.csv section",
    "format version 2 needs the decoder section",
];

#[test]
fn each_format_version_is_read_only_with_its_own_sections() {
    let refusals: Vec<String> = sections_out_of_their_version()
        .iter()
        .map(|summary| {
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("video.json");
            fs::write(&path, serde_json::to_string(summary).unwrap()).unwrap();
            refusal(&VideoSummary::read(&path).unwrap_err())
        })
        .collect();

    assert_eq!(refusals, REFUSALS);
}

#[test]
fn a_summary_out_of_its_version_is_refused_before_anything_is_written() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("video.json");

    let refusals: Vec<String> = sections_out_of_their_version()
        .iter()
        .map(|summary| refusal(&summary.write(&path).unwrap_err()))
        .collect();

    assert_eq!(refusals, REFUSALS);
    assert!(!path.exists());
}

#[test]
fn a_malformed_summary_is_reported_with_its_line() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("video.json");
    fs::write(&path, "{\n  \"format_version\": \"one\"\n}\n").unwrap();
    assert!(matches!(
        VideoSummary::read(&path).unwrap_err(),
        Error::Json { line: 2, .. }
    ));
}

#[test]
fn a_missing_summary_or_folder_is_reported() {
    let directory = tempfile::tempdir().unwrap();
    let absent = directory.path().join("absent").join("video.json");
    assert!(matches!(VideoSummary::read(&absent).unwrap_err(), Error::Read { .. }));
    assert!(matches!(summary_092().write(&absent).unwrap_err(), Error::Write { .. }));
}

#[derive(Debug, PartialEq, serde::Deserialize)]
struct Areas {
    min_area: f64,
    max_area: f64,
}

#[test]
fn the_detection_settings_of_a_recording_read_into_the_type_the_caller_asks_for() {
    let areas: Areas = summary_092().detect_settings().unwrap();
    assert_eq!(
        areas,
        Areas {
            min_area: 4.0,
            max_area: 2700.0
        }
    );
}

#[test]
fn a_recording_without_detection_settings_says_so() {
    let mut summary = summary_092();
    summary.recorded.remove("config");
    assert!(matches!(summary.detect_settings::<Areas>(), Err(Error::NoDetectConfig)));
}

#[test]
fn detection_settings_of_another_shape_are_refused_with_their_cause() {
    let error = summary_092().detect_settings::<Vec<f64>>().unwrap_err();
    assert!(matches!(error, Error::DetectConfig(_)));
    assert!(std::error::Error::source(&error).is_some());
}
