use std::fs::{self, File};
use std::path::Path;
use std::time::{Duration, SystemTime};

use super::read_verdicts_file;

fn written(root: &Path, name: &str, content: &[u8], minutes_ago: u64) {
    let path = root.join(name);
    fs::write(&path, content).unwrap();
    let moment = SystemTime::now() - Duration::from_secs(60 * minutes_ago);
    File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(moment)
        .unwrap();
}

#[test]
fn the_most_recent_verdicts_file_is_read_whatever_number_the_browser_gave_it() {
    let root = tempfile::tempdir().unwrap();
    written(root.path(), "verdicts.csv", b"old", 30);
    written(root.path(), "verdicts (1).csv", b"newest", 1);
    written(root.path(), "verdicts-2.csv", b"middle", 10);
    let file = read_verdicts_file(root.path()).unwrap();
    assert_eq!(file.name, "verdicts (1).csv");
    assert_eq!(file.bytes, b"newest");
    assert_eq!(file.candidates, 3);
}

#[test]
fn the_bytes_are_kept_as_written_whatever_their_encoding() {
    let root = tempfile::tempdir().unwrap();
    let latin = b"vid\xe9o;piste\r\nnuit;1\r\n";
    written(root.path(), "Verdicts.CSV", latin, 0);
    assert_eq!(read_verdicts_file(root.path()).unwrap().bytes, latin);
}

#[test]
fn the_date_of_the_file_is_given_for_the_page_to_tell() {
    let root = tempfile::tempdir().unwrap();
    written(root.path(), "verdicts.csv", b"", 0);
    let modified = read_verdicts_file(root.path()).unwrap().modified.unwrap();
    assert!(modified.ends_with('Z'), "{modified}");
}

#[test]
fn other_files_hidden_ones_and_folders_are_not_verdicts() {
    let root = tempfile::tempdir().unwrap();
    written(root.path(), "notes.csv", b"", 0);
    written(root.path(), "verdicts.txt", b"", 0);
    written(root.path(), ".verdicts.csv", b"", 0);
    fs::create_dir(root.path().join("verdicts-old.csv")).unwrap();
    assert_eq!(read_verdicts_file(root.path()), None);
}

#[test]
fn a_missing_folder_has_no_verdicts() {
    let root = tempfile::tempdir().unwrap();
    assert_eq!(read_verdicts_file(&root.path().join("absent")), None);
}
