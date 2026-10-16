use std::path::{Path, PathBuf};

use draw::canvas::{self, Canvas, PixelBox};
use draw::color::PANEL;
use engine::detection::Detected;
use engine::settings::Region;
use files::names::Names;
use media::convert::{self, Converter, SourceRange};
use summary::header::Labels;
use summary::image_file::{self, write_png};
use summary::layout::Summary;
use summary::{Painted, pages};

#[cfg(test)]
mod tests;

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
    pub(crate) regions: &'a [Region],
}

#[derive(Debug, Default)]
pub(crate) struct Drawn {
    pub(crate) pages: Vec<PathBuf>,
    pub(crate) painted: Vec<Painted>,
}

pub(crate) fn write_overview(
    overview: Overview<'_>,
    detected: &Detected,
    summary: &Summary<'_>,
) -> Result<Drawn, Error> {
    let background = detected.background().ok_or_else(|| Error::NoBackground {
        video: overview.video.to_path_buf(),
    })?;
    let (width, height) = (background.width(), background.height());
    let mut canvas = Canvas::new(width, height, PANEL).map_err(Error::Canvas)?;
    Converter::new(width, height, SourceRange::FullBlackAt16)
        .and_then(|mut converter| converter.yuv_to_rgba(background, canvas.rgba_mut()))
        .map_err(Error::Convert)?;
    let stem = overview.video.file_stem().unwrap_or_default();
    let mut drawn = Drawn::default();
    let masks = display_boxes(overview.regions, width, height);
    for page in pages(summary, canvas, &masks, overview.labels)? {
        let path = overview.names.summary_png(overview.dest, stem, page.minutes, page.part);
        write_png(&path, &page.canvas)?;
        drawn.pages.push(path);
        drawn.painted.extend(page.painted);
    }
    Ok(drawn)
}

fn display_boxes(regions: &[Region], width: u32, height: u32) -> Vec<PixelBox> {
    let (width, height) = (width as usize, height as usize);
    let bound = |position: usize| i32::try_from(position).unwrap_or(i32::MAX);
    regions
        .iter()
        .map(|region| {
            let (columns, rows) = (region.columns(width, 0), region.rows(height, 0));
            PixelBox {
                left: bound(columns.start),
                top: bound(rows.start),
                right: bound(columns.end) - 1,
                bottom: bound(rows.end) - 1,
            }
        })
        .collect()
}
