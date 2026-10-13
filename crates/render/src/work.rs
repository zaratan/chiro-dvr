use std::path::{Path, PathBuf};

use crate::windows::{Group, Window};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Work {
    Annotated(PathBuf),
    Clips(Group),
    Zoom(Window),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Paths<'a> {
    pub(crate) clips: &'a [Option<&'a Path>],
    pub(crate) zooms: &'a [Option<&'a Path>],
}

impl Work {
    pub(crate) fn start(&self) -> usize {
        match self {
            Self::Annotated(_) => 0,
            Self::Clips(group) => group.span().first().get(),
            Self::Zoom(window) => window.span.first().get(),
        }
    }

    pub(crate) fn paths(&self, paths: Paths<'_>) -> Vec<PathBuf> {
        let by_track = |tracks: Vec<usize>, table: &[Option<&Path>]| -> Vec<PathBuf> {
            tracks
                .into_iter()
                .filter_map(|track| table.get(track).copied().flatten())
                .map(Path::to_path_buf)
                .collect()
        };
        match self {
            Self::Annotated(path) => vec![path.clone()],
            Self::Clips(group) => by_track(group.members().iter().map(|member| member.track).collect(), paths.clips),
            Self::Zoom(window) => by_track(vec![window.track], paths.zooms),
        }
    }

    fn rank(&self) -> (usize, u8) {
        let kind = match self {
            Self::Annotated(_) => 0,
            Self::Clips(_) => 1,
            Self::Zoom(_) => 2,
        };
        (self.start(), kind)
    }
}

pub(crate) fn works(groups: Vec<Group>, zooms: &[Window], annotated: Option<&Path>) -> Vec<Work> {
    let mut works: Vec<Work> = annotated
        .map(|path| Work::Annotated(path.to_path_buf()))
        .into_iter()
        .chain(groups.into_iter().map(Work::Clips))
        .chain(zooms.iter().copied().map(Work::Zoom))
        .collect();
    works.sort_by_key(Work::rank);
    works
}
