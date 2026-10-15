use std::fs;
use std::path::{Path, PathBuf};

use super::{Error, collect_videos, plan_jobs};

fn dests(videos: &[&str], out: Option<&Path>) -> Vec<PathBuf> {
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
    assert_eq!(dests(&["night/video_092.mp4"], Some(out)), [out.join("video_092")]);
}

#[test]
fn videos_with_the_same_name_get_distinct_output_folders() {
    let out = Path::new("out");
    assert_eq!(
        dests(
            &["night1/video_001.mp4", "night2/video_001.mp4", "night2/video_002.mp4"],
            Some(out)
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
        dests(&["night1/x.mp4", "night1/x.MOV"], Some(out)),
        [out.join("night1_x_mp4"), out.join("night1_x_mov")]
    );
}

#[test]
fn videos_that_would_share_an_output_folder_are_refused() {
    let videos = [PathBuf::from("a/night/x.mp4"), PathBuf::from("b/night/x.mp4")];
    let error = plan_jobs(&videos, Some(Path::new("out"))).unwrap_err();
    assert!(matches!(error, Error::SharedFolder { .. }));
    assert!(error.to_string().starts_with("several videos would write to"));
}

#[test]
fn without_an_output_folder_the_results_go_to_a_batdetect_folder_beside_each_video() {
    assert_eq!(
        dests(&["night/video_092.mp4", "video_093.mp4"], None),
        [
            PathBuf::from("night/batdetect/video_092"),
            PathBuf::from("batdetect/video_093")
        ]
    );
}

#[test]
fn without_an_output_folder_the_same_name_in_two_folders_needs_no_other_name() {
    assert_eq!(
        dests(&["night1/v.mp4", "night2/v.mp4"], None),
        [PathBuf::from("night1/batdetect/v"), PathBuf::from("night2/batdetect/v")]
    );
}

#[test]
fn without_an_output_folder_the_same_name_in_one_folder_takes_its_extension_but_not_its_folder() {
    assert_eq!(
        dests(&["night1/x.mp4", "night1/x.MOV"], None),
        [
            PathBuf::from("night1/batdetect/x_mp4"),
            PathBuf::from("night1/batdetect/x_mov")
        ]
    );
}

#[test]
fn names_that_differ_only_by_case_are_homonyms_as_on_apfs_and_ntfs() {
    let out = Path::new("out");
    assert_eq!(
        dests(&["a/Vol.mp4", "b/vol.mp4"], Some(out)),
        [out.join("a_Vol_mp4"), out.join("b_vol_mp4")]
    );
    assert_eq!(
        dests(&["night/V.mp4", "night/v.mov"], None),
        [
            PathBuf::from("night/batdetect/V_mp4"),
            PathBuf::from("night/batdetect/v_mov")
        ]
    );
}

#[test]
fn a_name_decomposed_and_the_same_name_composed_are_homonyms() {
    let out = Path::new("out");
    assert_eq!(
        dests(&["a/e\u{301}te\u{301}.mp4", "b/\u{e9}t\u{e9}.mp4"], Some(out)),
        [out.join("a_e\u{301}te\u{301}_mp4"), out.join("b_\u{e9}t\u{e9}_mp4")]
    );
}

#[cfg(unix)]
#[test]
fn names_that_are_not_utf8_and_differ_only_by_their_invalid_bytes_are_not_homonyms() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let videos: Vec<PathBuf> = [b"night/\xff.mp4".as_slice(), b"night/\xfe.mp4"]
        .into_iter()
        .map(|bytes| PathBuf::from(OsStr::from_bytes(bytes)))
        .collect();
    let jobs = plan_jobs(&videos, None).unwrap();
    assert_eq!(
        jobs[0].dest,
        Path::new("night/batdetect").join(OsStr::from_bytes(b"\xff"))
    );
    assert_eq!(
        jobs[1].dest,
        Path::new("night/batdetect").join(OsStr::from_bytes(b"\xfe"))
    );
}

#[test]
fn folders_that_differ_only_by_case_are_refused_as_one_shared_folder() {
    let videos = [PathBuf::from("a/Night/x.mp4"), PathBuf::from("b/night/x.mp4")];
    let error = plan_jobs(&videos, Some(Path::new("out"))).unwrap_err();
    assert!(matches!(error, Error::SharedFolder { .. }), "{error}");
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

fn a_night_with(names: &[&str]) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    for name in names {
        fs::write(root.path().join(name), "").unwrap();
    }
    root
}

#[test]
fn a_video_given_both_by_its_folder_and_by_its_name_is_kept_once() {
    let night = a_night_with(&["video.mp4"]);
    let videos = collect_videos(&[night.path().to_path_buf(), night.path().join("video.mp4")]).unwrap();
    assert_eq!(videos, [night.path().join("video.mp4")]);
}

#[test]
fn a_video_written_two_ways_is_kept_once_under_its_first_writing() {
    let night = a_night_with(&["video.mp4"]);
    let written = night.path().join(".").join("video.mp4");
    let videos = collect_videos(&[night.path().to_path_buf(), written]).unwrap();
    assert_eq!(videos, [night.path().join("video.mp4")]);
}

#[test]
fn two_videos_of_one_folder_written_two_ways_share_one_results_folder() {
    let night = a_night_with(&["a.mp4", "b.mp4"]);
    let videos = [night.path().join("a.mp4"), night.path().join(".").join("b.mp4")];
    let jobs = plan_jobs(&videos, None).unwrap();
    let roots: Vec<&Path> = jobs.iter().map(|job| job.dest.parent().unwrap()).collect();
    assert_eq!(roots, [night.path().join("batdetect"), night.path().join("batdetect")]);
}

#[test]
fn two_videos_of_one_name_in_one_folder_written_two_ways_are_still_told_apart() {
    let night = a_night_with(&["x.mp4", "x.mov"]);
    let videos = [night.path().join("x.mp4"), night.path().join(".").join("x.mov")];
    let jobs = plan_jobs(&videos, None).unwrap();
    let names: Vec<&std::ffi::OsStr> = jobs.iter().map(|job| job.dest.file_name().unwrap()).collect();
    assert_eq!(names, ["x_mp4", "x_mov"]);
}

#[test]
fn a_missing_video_written_two_ways_is_kept_once_as_first_written_so_that_its_failure_names_it() {
    let night = a_night_with(&[]);
    let missing = night.path().join("absente.mp4");
    let again = night.path().join(".").join("absente.mp4");
    let videos = collect_videos(&[missing.clone(), again, missing.clone()]).unwrap();
    assert_eq!(videos, [missing]);
}

#[cfg(unix)]
#[test]
fn a_link_to_a_video_given_with_the_video_itself_is_kept_once() {
    let night = a_night_with(&["video.mp4"]);
    std::os::unix::fs::symlink(night.path().join("video.mp4"), night.path().join("lien.mp4")).unwrap();
    let videos = collect_videos(&[night.path().join("video.mp4"), night.path().join("lien.mp4")]).unwrap();
    assert_eq!(videos, [night.path().join("video.mp4")]);
}
