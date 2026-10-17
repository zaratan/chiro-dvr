use std::io;

use compare::case::Target;
use compare::comparison::{Comparison, Folders, Gaps, VIDEOS_AT_ONCE, compare_all};
use compare::report::write_report;
use reference::location::Reference;

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
            ("video_093.mp4", agreeing(6)),
            ("video_094.mp4", agreeing(14)),
            ("video_095.mp4", agreeing(78)),
            ("video_096.mp4", agreeing(136)),
            ("video_097.mp4", agreeing(149)),
            ("video_098.mp4", agreeing(131)),
            ("video_099.mp4", agreeing(117)),
            ("video_100.mp4", agreeing(98)),
            ("video_101.mp4", agreeing(32)),
            ("video_102.mp4", agreeing(36)),
            ("video_103.mp4", agreeing(10)),
            ("video_120.mp4", agreeing(0)),
            ("video_121.mp4", agreeing(2)),
            ("video_122.mp4", agreeing(0)),
            ("video_123.mp4", agreeing(8)),
            ("video_124.mp4", agreeing(5)),
            ("video_125.mp4", agreeing(47)),
            ("video_126.mp4", agreeing(13)),
            ("video_127.mp4", agreeing(7)),
            ("video_128.mp4", agreeing(5)),
            ("video_129.mp4", agreeing(16)),
            ("video_130.mp4", agreeing(6)),
        ]
    );
}
