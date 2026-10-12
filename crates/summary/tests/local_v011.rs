#[path = "support/v011.rs"]
mod v011;

use reference::location::recording_directories;

const PTS_CLOCKS: [&str; 15] = [
    "video_095: track 17 start 2:52, v0.1.1 2:53",
    "video_095: track 47 start 4:13, v0.1.1 4:14",
    r#"video_096: header NotAnalysed { start: "2:23", end: "2:26" }, v0.1.1 NotAnalysed { start: "2:23", end: "2:27" }"#,
    "video_097: track 40 start 1:10, v0.1.1 1:11",
    "video_097: track 112 start 3:32, v0.1.1 3:33",
    "video_099: track 85 start 3:19, v0.1.1 3:20",
    "video_099: track 87 start 3:20, v0.1.1 3:21",
    "video_100: track 37 start 1:44, v0.1.1 1:45",
    "video_101: track 30 start 3:33, v0.1.1 3:34",
    "video_103: track 5 start 3:41, v0.1.1 3:40",
    "video_103: track 8 start 4:12, v0.1.1 4:11",
    "video_103: track 10 start 4:52, v0.1.1 4:51",
    r#"video_120: header Unreadable { duration: "1 min 14 s", spans: 44 }, v0.1.1 Unreadable { duration: "1 min 13 s", spans: 44 }"#,
    "video_125: track 11 start 0:33, v0.1.1 0:34",
    "video_126: track 10 start 4:17, v0.1.1 4:18",
];

#[test]
#[ignore = "local"]
fn the_twenty_six_reference_videos_get_the_layout_of_v0_1_1_but_for_the_pts_clocks() {
    let directories = recording_directories().unwrap();
    assert_eq!(directories.len(), 26);
    let (mut tracks, mut differences, mut misplaced) = (0, Vec::new(), Vec::new());
    for directory in directories {
        let name = directory.file_name().unwrap().to_string_lossy().into_owned();
        let expected = directory
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("summary")
            .join(format!("{name}.json"));
        let found = v011::compare(&directory, &expected);
        println!(
            "{name}\ttracks {}\tradius {}\tdifferences {}\tmisplaced {}\tbeside {}\tfarthest {:.0}",
            found.tracks,
            found.radius,
            found.differences.len(),
            found.misplaced.len(),
            found.beside,
            found.farthest
        );
        tracks += found.tracks;
        differences.extend(found.differences);
        misplaced.extend(found.misplaced);
    }
    for line in differences.iter().chain(&misplaced) {
        println!("{line}");
    }
    assert_eq!(tracks, 966);
    assert_eq!(differences, PTS_CLOCKS);
    assert_eq!(misplaced, Vec::<String>::new());
}
