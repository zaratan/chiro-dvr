#[path = "support/listing.rs"]
mod listing;

use std::fs;

use listing::left_in;

#[test]
fn the_listing_leaves_out_the_apple_double_files_macos_writes_on_exfat_so_a_forgotten_partial_still_shows() {
    let dest = tempfile::tempdir().unwrap();
    for name in ["v_tracks.csv", "._v_tracks.csv", ".v_tracks.csv.a1b2c3.partial"] {
        fs::write(dest.path().join(name), "").unwrap();
    }
    assert_eq!(left_in(dest.path()), [".v_tracks.csv.a1b2c3.partial", "v_tracks.csv"]);
}
