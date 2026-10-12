#[path = "support/v011.rs"]
mod v011;

use std::path::PathBuf;

use reference::location::committed;

fn expected(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(format!("{name}.summary.json"))
}

#[test]
fn the_extract_tracks_of_the_reference_get_the_numbers_colours_and_header_of_v0_1_1() {
    let name = "video_092_original_3m24-4m05";
    let found = v011::compare(&committed(name), &expected(name));
    assert_eq!(found.tracks, 4);
    assert_eq!((found.radius, found.beside), (23.0, 0));
    assert!(found.farthest < 23.0, "{}", found.farthest);
    assert_eq!(found.differences, Vec::<String>::new());
    assert_eq!(found.misplaced, Vec::<String>::new());
}

#[test]
fn the_092_tracks_of_the_reference_get_the_numbers_colours_and_header_of_v0_1_1() {
    let name = "video_092_original";
    let found = v011::compare(&committed(name), &expected(name));
    assert_eq!(found.tracks, 14);
    assert_eq!((found.radius, found.beside), (23.0, 0));
    assert!(found.farthest < 23.0, "{}", found.farthest);
    assert_eq!(found.differences, Vec::<String>::new());
    assert_eq!(found.misplaced, Vec::<String>::new());
}
