use std::io;

use compare::case::Target;
use compare::comparison::{Comparison, Folders, Gaps, VIDEOS_AT_ONCE, compare_all};
use compare::report::write_report;
use reference::location::Reference;

#[test]
#[ignore = "local"]
fn the_twenty_six_videos_keep_the_gaps_frozen_against_the_v020_video_by_video_where_only_weak_detections_are_added() {
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
    let weak = Gaps::added_only;

    assert_eq!(
        found,
        [
            ("video_089.mp4", weak(22, 553, [1, 1, 0, 0])),
            ("video_090.mp4", weak(40, 48, [18, 14, 2, 2])),
            ("video_091.mp4", weak(215, 9602, [21, 21, 0, 0])),
            ("video_092.mp4", weak(19, 20, [14, 14, 0, 0])),
            ("video_093.mp4", weak(38, 253, [6, 6, 0, 1])),
            ("video_094.mp4", weak(681, 50_496, [14, 12, 0, 0])),
            ("video_095.mp4", weak(152, 167, [78, 70, 0, 2])),
            ("video_096.mp4", weak(276, 3763, [135, 122, 1, 4])),
            ("video_097.mp4", weak(303, 1621, [148, 137, 1, 5])),
            ("video_098.mp4", weak(328, 2871, [130, 114, 1, 8])),
            ("video_099.mp4", weak(235, 499, [117, 105, 0, 5])),
            ("video_100.mp4", weak(202, 325, [98, 90, 0, 4])),
            ("video_101.mp4", weak(91, 199, [31, 31, 1, 1])),
            ("video_102.mp4", weak(68, 104, [36, 32, 0, 2])),
            ("video_103.mp4", weak(3893, 155_968, [10, 9, 0, 0])),
            ("video_120.mp4", weak(10, 11, [0, 0, 0, 0])),
            ("video_121.mp4", weak(21, 24, [2, 2, 0, 1])),
            ("video_122.mp4", weak(11, 12, [0, 0, 0, 0])),
            ("video_123.mp4", weak(8, 8, [8, 8, 0, 0])),
            ("video_124.mp4", weak(4, 4, [5, 5, 0, 0])),
            ("video_125.mp4", weak(69, 70, [46, 42, 1, 2])),
            ("video_126.mp4", weak(21, 21, [13, 12, 0, 0])),
            ("video_127.mp4", weak(20, 21, [7, 5, 0, 0])),
            ("video_128.mp4", weak(17, 17, [5, 4, 0, 0])),
            ("video_129.mp4", weak(28, 28, [16, 14, 0, 0])),
            ("video_130.mp4", weak(307, 9917, [6, 6, 0, 0])),
        ]
    );
}
