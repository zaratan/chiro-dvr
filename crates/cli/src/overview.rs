use std::path::{Path, PathBuf};

use draw::canvas::{self, Canvas, PixelBox};
use draw::color::{PANEL, Rgb};
use engine::detection::Detected;
use engine::settings::Region;
use files::names::{Names, Part};
use media::convert::{self, Converter, SourceRange};
use summary::header::Labels;
use summary::image_file::{self, write_png};
use summary::layout::Summary;
use summary::{Painted, pages};
use types::track::Track;

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

#[derive(Debug)]
pub(crate) struct Drawn {
    pub(crate) background: PathBuf,
    pub(crate) sheets: Vec<Sheet>,
    pub(crate) painted: Vec<Drawing>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Sheet {
    pub(crate) page: PathBuf,
    pub(crate) part: Option<Part>,
    pub(crate) window: [f64; 2],
    pub(crate) ignored: Vec<[f64; 2]>,
    pub(crate) damaged: Vec<[f64; 2]>,
    pub(crate) other_starts: Vec<f64>,
    pub(crate) radius: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Drawing {
    pub(crate) page: PathBuf,
    pub(crate) painted: Painted,
}

impl Drawn {
    pub(crate) fn colors(&self, tracks: &[Track]) -> Vec<Option<Rgb>> {
        tracks
            .iter()
            .map(|track| {
                self.painted
                    .iter()
                    .find(|drawing| drawing.painted.track == track.id())
                    .map(|drawing| drawing.painted.color)
            })
            .collect()
    }
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
    let masks = display_boxes(overview.regions, width, height);
    let summaries = pages(summary, canvas, &masks, overview.labels)?;
    let mut drawn = Drawn {
        background: overview.names.background_png(overview.dest, stem),
        sheets: Vec::new(),
        painted: Vec::new(),
    };
    write_png(&drawn.background, &summaries.backdrop)?;
    for page in summaries.pages {
        let path = overview
            .names
            .summary_png(overview.dest, stem, page.minutes.clone(), page.part);
        write_png(&path, &page.canvas)?;
        drawn.painted.extend(page.painted.into_iter().map(|painted| Drawing {
            page: path.clone(),
            painted,
        }));
        drawn.sheets.push(Sheet {
            page: path,
            part: page.part,
            window: page.window,
            ignored: page.ignored,
            damaged: page.damaged,
            other_starts: page.other_starts,
            radius: page.radius,
        });
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
