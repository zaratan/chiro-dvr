use std::io;

use compare::case::Target;
use compare::comparison::{Comparison, Folders, Gaps, VIDEOS_AT_ONCE, compare_all};
use compare::report::write_report;
use reference::location::Reference;

fn rejoined(reference: usize, joins: usize, tracks_joined: usize) -> Gaps {
    let paired = reference - joins;
    Gaps {
        paired_tracks: paired,
        agreeing_tracks: paired - tracks_joined,
        lost_tracks: joins,
        ..Gaps::agreeing(0)
    }
}

#[test]
#[ignore = "local"]
fn the_twenty_six_videos_keep_the_gaps_frozen_against_the_v020_video_by_video() {
    let cases = Target::Recorded.cases(Reference::V020).unwrap();
    let comparisons: Vec<Comparison> = compare_all(&cases, VIDEOS_AT_ONCE, &Folders::default())
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();

    write_report(&comparisons, &mut io::stdout().lock()).unwrap();
    let found: Vec<(&str, Gaps)> = comparisons
        .iter()
        .map(|comparison| (comparison.detections.name(), comparison.gaps()))
        .collect();
    let agreeing = Gaps::agreeing;

    assert_eq!(
        found,
        [
            ("video_089.mp4", agreeing(1)),
            ("video_090.mp4", agreeing(20)),
            ("video_091.mp4", agreeing(21)),
            ("video_092.mp4", agreeing(14)),
            ("video_093.mp4", rejoined(6, 2, 2)),
            ("video_094.mp4", rejoined(14, 1, 1)),
            ("video_095.mp4", rejoined(78, 11, 11)),
            ("video_096.mp4", rejoined(136, 13, 12)),
            ("video_097.mp4", rejoined(149, 15, 11)),
            ("video_098.mp4", rejoined(131, 8, 7)),
            ("video_099.mp4", rejoined(117, 7, 7)),
            ("video_100.mp4", rejoined(98, 5, 5)),
            ("video_101.mp4", rejoined(32, 1, 1)),
            ("video_102.mp4", rejoined(36, 1, 1)),
            ("video_103.mp4", agreeing(10)),
            ("video_120.mp4", agreeing(0)),
            ("video_121.mp4", agreeing(2)),
            ("video_122.mp4", agreeing(0)),
            ("video_123.mp4", agreeing(8)),
            ("video_124.mp4", agreeing(5)),
            ("video_125.mp4", rejoined(47, 3, 3)),
            ("video_126.mp4", agreeing(13)),
            ("video_127.mp4", agreeing(7)),
            ("video_128.mp4", agreeing(5)),
            ("video_129.mp4", agreeing(16)),
            ("video_130.mp4", agreeing(6)),
        ]
    );
}
