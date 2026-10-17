use std::io::{self, Write};
use std::num::NonZeroU32;
use std::path::Path;
use std::time::Duration;

use batdetect::options::{Change, Options};
use batdetect::outcome::{Failure, Row, State};
use batdetect::process::Extent;
use batdetect::reuse::Earlier;
use batdetect::tally::Tally;
use engine::settings::Setting;
use files::names::composed;
use files::params::Finish;

use crate::arguments::{MAX_BLOBS, MIN_HITS, OSD_REGION};
use crate::clock::clock;
use crate::messages;
use crate::texts::{Fixed, Message, Texts};

#[cfg(test)]
mod tests;

const INDENT: &str = "  ";
const GAP: &str = "   ";
const UNKNOWN: &str = "—";
const LIST_SEPARATOR: &str = ", ";

pub(crate) fn write(out: &mut impl Write, tally: &Tally, options: &Options, texts: Texts) -> io::Result<()> {
    writeln!(out, "{}", title(tally, texts))?;
    for root in tally.roots() {
        writeln!(out)?;
        writeln!(out, "{}", root.display())?;
        for line in table(tally, root, texts) {
            writeln!(out, "{line}")?;
        }
    }
    too_many(out, tally, texts)?;
    failed(out, tally, options, texts)?;
    refused(out, tally, texts)?;
    left(out, tally, texts)?;
    unwritten(out, tally, texts)?;
    let changed = options.changed();
    if !changed.is_empty() {
        let settings: Vec<String> = changed.iter().map(|change| setting(*change, texts)).collect();
        let settings = settings.join(LIST_SEPARATOR);
        writeln!(out)?;
        writeln!(out, "{}", texts.say(&Message::TallyChanged { settings }))?;
    }
    Ok(())
}

pub(crate) fn title(tally: &Tally, texts: Texts) -> String {
    let (done, too_many, failed) = (tally.done().count(), tally.too_many().count(), tally.failed().count());
    let already = tally.already_done().count();
    let counted = [
        (done, Message::TallyDone { count: done }),
        (already, Message::TallyAlreadyDone { count: already }),
        (too_many, Message::TallyTooManyCount { count: too_many }),
        (failed, Message::TallyFailedCount { count: failed }),
        (
            tally.refused.len(),
            Message::TallyRefusedCount {
                count: tally.refused.len(),
            },
        ),
    ];
    let total = texts.say(&Message::TallyTitle {
        total: tally.rows.len(),
    });
    let parts = counted
        .iter()
        .filter(|(count, _)| *count > 0)
        .map(|(_, message)| texts.say(message));
    std::iter::once(total)
        .chain(parts)
        .collect::<Vec<_>>()
        .join(LIST_SEPARATOR)
}

fn table(tally: &Tally, root: &Path, texts: Texts) -> Vec<String> {
    let header = [
        Fixed::TallyVideo,
        Fixed::TallyLength,
        Fixed::TallyTracks,
        Fixed::TallyNotAnalysed,
        Fixed::TallyCompute,
    ]
    .map(|fixed| texts.text(fixed));
    let mut cells = vec![header];
    cells.extend(tally.rows_in(root).map(|row| cells_of(row, texts)));
    let widths: [usize; 5] =
        std::array::from_fn(|column| cells.iter().map(|row| width(&row[column])).max().unwrap_or(0));
    cells
        .iter()
        .map(|row| {
            let cells: Vec<String> = row
                .iter()
                .zip(widths)
                .enumerate()
                .map(|(column, (cell, width))| {
                    if column == 0 {
                        format!("{cell:<width$}")
                    } else {
                        format!("{cell:>width$}")
                    }
                })
                .collect();
            format!("{INDENT}{}", cells.join(GAP))
        })
        .collect()
}

fn cells_of(row: &Row, texts: Texts) -> [String; 5] {
    let compute = clock(row.elapsed.as_secs());
    match &row.state {
        State::Done { tracks, extent, .. } => {
            let (length, not_analysed) = measured(*extent);
            [name(row), length, texts.count(*tracks), not_analysed, compute]
        }
        State::TooManyTracks { count, extent, .. } => {
            let (length, not_analysed) = measured(*extent);
            let tracks = texts.say(&Message::TallyTooManyCell { count: *count });
            [name(row), length, tracks, not_analysed, compute]
        }
        State::Skipped(Earlier {
            finish, tracks, extent, ..
        }) => {
            let (length, not_analysed) = measured(*extent);
            let tracks = match finish {
                Finish::Done => texts.count(*tracks),
                Finish::TooManyTracks => texts.say(&Message::TallyTooManyCell { count: *tracks }),
            };
            [name(row), length, tracks, not_analysed, UNKNOWN.to_owned()]
        }
        State::Failed(_) => [
            name(row),
            UNKNOWN.to_owned(),
            texts.text(Fixed::TallyFailedCell),
            UNKNOWN.to_owned(),
            compute,
        ],
    }
}

fn measured(extent: Extent) -> (String, String) {
    let length = Duration::try_from_secs_f64(extent.length_s)
        .map_or_else(|_| UNKNOWN.to_owned(), |length| clock(length.as_secs()));
    let lost = Duration::try_from_secs_f64(extent.not_analysed_s).map_or_else(
        |_| UNKNOWN.to_owned(),
        |lost| clock(lost.as_secs() + u64::from(lost.subsec_nanos() > 0)),
    );
    (length, lost)
}

fn name(row: &Row) -> String {
    composed(&row.job.dest.file_name().unwrap_or_default().to_string_lossy())
}

fn width(cell: &str) -> usize {
    cell.chars().count()
}

fn too_many(out: &mut impl Write, tally: &Tally, texts: Texts) -> io::Result<()> {
    let rows: Vec<(&Row, usize, usize)> = tally.too_many().collect();
    if rows.is_empty() {
        return Ok(());
    }
    writeln!(out)?;
    writeln!(out, "{}", texts.text(Fixed::TallyTooManyHeading))?;
    for (row, count, limit) in rows {
        let phrase = Message::TallyTooMany {
            video: name(row),
            count,
            limit,
        };
        writeln!(out, "{INDENT}{}", texts.say(&phrase))?;
        writeln!(out, "{INDENT}{}", texts.say(&messages::too_many_todo(&row.job)))?;
    }
    Ok(())
}

fn failed(out: &mut impl Write, tally: &Tally, options: &Options, texts: Texts) -> io::Result<()> {
    let rows: Vec<(&Row, &Failure)> = tally.failed().collect();
    if rows.is_empty() {
        return Ok(());
    }
    writeln!(out)?;
    let heading = Message::TallyFailedHeading {
        failed: rows.len(),
        total: tally.rows.len(),
    };
    writeln!(out, "{}", texts.say(&heading))?;
    for (row, failure) in rows {
        for message in messages::failure_reason(name(row), &row.job, failure, options) {
            writeln!(out, "{INDENT}{}", texts.say(&messages::without_detail(message)))?;
        }
    }
    Ok(())
}

fn refused(out: &mut impl Write, tally: &Tally, texts: Texts) -> io::Result<()> {
    if tally.refused.is_empty() {
        return Ok(());
    }
    writeln!(out)?;
    let heading = Message::TallyRefusedHeading {
        count: tally.refused.len(),
    };
    writeln!(out, "{}", texts.say(&heading))?;
    for line in messages::refused_lines(texts, &tally.refused) {
        writeln!(out, "{INDENT}{line}")?;
    }
    Ok(())
}

fn left(out: &mut impl Write, tally: &Tally, texts: Texts) -> io::Result<()> {
    let left: Vec<_> = tally
        .kept()
        .map(|(row, table)| (row, &table.path, table.ours))
        .collect();
    if left.is_empty() {
        return Ok(());
    }
    writeln!(out)?;
    let heading = Message::TallyKeptHeading { count: left.len() };
    writeln!(out, "{}", texts.say(&heading))?;
    for (row, path, ours) in left {
        let message = Message::TallyKept {
            video: name(row),
            table: file_name(path),
            written: !ours,
        };
        writeln!(out, "{INDENT}{}", texts.say(&message))?;
    }
    Ok(())
}

fn unwritten(out: &mut impl Write, tally: &Tally, texts: Texts) -> io::Result<()> {
    let errors = tally.unwritten_files();
    if errors.is_empty() {
        return Ok(());
    }
    writeln!(out)?;
    let heading = Message::TallyUnwrittenHeading { count: errors.len() };
    writeln!(out, "{}", texts.say(&heading))?;
    for error in errors {
        let message = Message::TallyUnwritten {
            path: error.path().display().to_string(),
            taken: matches!(error, files::error::Error::Taken { .. }),
            detail: messages::causes(error),
        };
        writeln!(out, "{INDENT}{}", texts.say(&message))?;
    }
    Ok(())
}

fn file_name(path: &Path) -> String {
    path.file_name().unwrap_or_default().to_string_lossy().into_owned()
}

fn setting(change: Change, texts: Texts) -> String {
    match change {
        Change::Threshold(value) => format!("--{} {value}", Setting::Threshold.flag()),
        Change::MinHits(value) => format!("--{MIN_HITS} {value}"),
        Change::MaxBlobs(value) => format!("--{MAX_BLOBS} {}", value.map_or(0, NonZeroU32::get)),
        Change::WorkWidth(value) => format!("--{} {value}", Setting::WorkWidth.flag()),
        Change::OsdRegions(count) => format!("--{OSD_REGION} ({})", texts.say(&Message::TallyRegions { count })),
    }
}
