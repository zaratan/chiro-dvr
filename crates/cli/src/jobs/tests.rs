use std::fs;
use std::path::{Path, PathBuf};

use super::{Error, collect_videos, plan_jobs};

fn dests(videos: &[&str], out: &Path) -> Vec<PathBuf> {
    let videos: Vec<PathBuf> = videos.iter().map(PathBuf::from).collect();
    plan_jobs(&videos, out)
        .unwrap()
        .into_iter()
        .map(|job| job.dest)
        .collect()
}

#[test]
fn a_folder_gives_its_video_files_in_name_order() {
    let folder = tempfile::tempdir().unwrap();
    for name in ["b.MP4", "a.mov", "notes.txt", "c.mkv"] {
        fs::write(folder.path().join(name), "").unwrap();
    }
    fs::create_dir(folder.path().join("sub.mp4")).unwrap();
    let names: Vec<_> = collect_videos(&[folder.path().to_path_buf()])
        .unwrap()
        .into_iter()
        .map(|path| path.file_name().unwrap().to_owned())
        .collect();
    assert_eq!(names, ["a.mov", "b.MP4", "c.mkv"]);
}

#[test]
fn a_file_given_by_name_is_kept_whatever_its_extension() {
    let videos = collect_videos(&[PathBuf::from("night/clip.dat")]).unwrap();
    assert_eq!(videos, [PathBuf::from("night/clip.dat")]);
}

#[test]
fn each_video_gets_a_folder_named_after_it() {
    let out = Path::new("out");
    assert_eq!(dests(&["night/video_092.mp4"], out), [out.join("video_092")]);
}

#[test]
fn videos_with_the_same_name_get_distinct_output_folders() {
    let out = Path::new("out");
    assert_eq!(
        dests(
            &["night1/video_001.mp4", "night2/video_001.mp4", "night2/video_002.mp4"],
            out
        ),
        [
            out.join("night1_video_001_mp4"),
            out.join("night2_video_001_mp4"),
            out.join("video_002")
        ]
    );
}

#[test]
fn the_same_name_with_another_extension_in_the_same_folder_gets_its_own_folder() {
    let out = Path::new("out");
    assert_eq!(
        dests(&["night1/x.mp4", "night1/x.MOV"], out),
        [out.join("night1_x_mp4"), out.join("night1_x_mov")]
    );
}

#[test]
fn videos_that_would_share_an_output_folder_are_refused() {
    let videos = [PathBuf::from("a/night/x.mp4"), PathBuf::from("b/night/x.mp4")];
    let error = plan_jobs(&videos, Path::new("out")).unwrap_err();
    assert!(matches!(error, Error::SharedFolder { .. }));
    assert!(error.to_string().starts_with("several videos would write to"));
}

#[cfg(unix)]
#[test]
fn a_folder_that_cannot_be_listed_is_reported_by_name() {
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::tempdir().unwrap();
    let locked = root.path().join("verrouillé");
    fs::create_dir(&locked).unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
    let outcome = collect_videos(std::slice::from_ref(&locked));
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
    let error = outcome.unwrap_err();
    assert!(matches!(error, Error::Folder { .. }));
    assert!(error.to_string().starts_with("cannot list the videos of"));
    assert!(error.to_string().contains("verrouillé"));
}
