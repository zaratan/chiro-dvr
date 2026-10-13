use std::path::{Path, PathBuf};

use types::frame::FrameIndex;
use types::span::Span;

use super::{Paths, Work, works};
use crate::windows::{Window, groups};

fn window(track: usize, first: usize, last: usize) -> Window {
    Window {
        track,
        span: Span::single(FrameIndex::new(first)).hull(Span::single(FrameIndex::new(last))),
    }
}

#[test]
fn works_start_by_their_first_frame_and_a_group_goes_before_the_zoom_that_starts_with_it() {
    let groups = groups(&[window(0, 10, 20), window(1, 40, 50)]);
    let starts: Vec<(usize, bool)> = works(groups, &[window(1, 40, 50), window(2, 5, 8)], None)
        .iter()
        .map(|work| (work.start(), matches!(work, Work::Zoom(_))))
        .collect();
    assert_eq!(starts, [(5, true), (10, false), (40, false), (40, true)]);
}

#[test]
fn without_zooms_the_works_are_the_groups() {
    let windows = [window(0, 10, 20), window(1, 15, 30)];
    let found = works(groups(&windows), &[], None);
    assert_eq!(found, groups(&windows).into_iter().map(Work::Clips).collect::<Vec<_>>());
}

#[test]
fn a_work_names_the_files_it_writes() {
    let (clip, zoom) = (Path::new("clip.mp4"), Path::new("clip_zoom.mp4"));
    let clips = [Some(clip)];
    let zooms = [Some(zoom)];
    let annotated = Path::new("video_annotated.mp4");
    let all = works(groups(&[window(0, 10, 20)]), &[window(0, 10, 20)], Some(annotated));
    let table = Paths {
        clips: &clips,
        zooms: &zooms,
    };
    let paths: Vec<Vec<PathBuf>> = all.iter().map(|work| work.paths(table)).collect();
    assert_eq!(
        paths,
        [
            vec![annotated.to_path_buf()],
            vec![clip.to_path_buf()],
            vec![zoom.to_path_buf()]
        ]
    );
}

#[test]
fn the_annotated_video_starts_first_even_beside_a_group_that_opens_the_video() {
    let all = works(groups(&[window(0, 0, 20)]), &[], Some(Path::new("annotated.mp4")));
    assert!(matches!(all[0], Work::Annotated(_)));
    assert!(matches!(all[1], Work::Clips(_)));
}
