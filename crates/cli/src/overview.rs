use std::path::{Path, PathBuf};

use draw::canvas::{self, Canvas};
use draw::color::PANEL;
use engine::detection::Detected;
use files::names::Names;
use media::convert::{self, Converter, SourceRange};
use summary::header::Labels;
use summary::image_file::{self, write_png};
use summary::layout::Summary;
use summary::pages;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no frame of {} was kept for the summary background", video.display())]
    NoBackground { video: PathBuf },
    #[error("cannot convert the summary background")]
    Convert(#[source] convert::Error),
    #[error("cannot draw the summary")]
    Canvas(#[source] canvas::Error),
    #[error(transparent)]
    Summary(#[from] summary::Error),
    #[error(transparent)]
    Image(#[from] image_file::Error),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Overview<'a> {
    pub(crate) video: &'a Path,
    pub(crate) dest: &'a Path,
    pub(crate) names: Names,
    pub(crate) labels: &'a dyn Labels,
}

pub(crate) fn write_overview(
    overview: Overview<'_>,
    detected: &Detected,
    summary: &Summary<'_>,
) -> Result<Vec<PathBuf>, Error> {
    let background = detected.background().ok_or_else(|| Error::NoBackground {
        video: overview.video.to_path_buf(),
    })?;
    let (width, height) = (background.width(), background.height());
    let mut canvas = Canvas::new(width, height, PANEL).map_err(Error::Canvas)?;
    Converter::new(width, height, SourceRange::FullBlackAt16)
        .and_then(|mut converter| converter.yuv_to_rgba(background, canvas.rgba_mut()))
        .map_err(Error::Convert)?;
    let stem = overview.video.file_stem().unwrap_or_default();
    let mut written = Vec::new();
    for page in pages(summary, canvas, &[], overview.labels)? {
        let path = overview.names.summary_png(overview.dest, stem, page.minutes);
        write_png(&path, &page.canvas)?;
        written.push(path);
    }
    Ok(written)
}
