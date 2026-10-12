use std::error::Error;
use std::io::{self, Write};
use std::path::Path;

use crate::agreement::MIN_RATE;
use crate::comparison::Comparison;
use crate::outcome::Refusal;
use crate::tracks::TrackCounts;

#[cfg(test)]
mod tests;

pub fn write_report(comparisons: &[Comparison], out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "detections, outside the ignored spans of the reference")?;
    for comparison in comparisons {
        comparison.detections.write_summary(out)?;
    }
    writeln!(out, "tracks")?;
    for comparison in comparisons {
        comparison.tracks.write_summary(out)?;
    }
    let TrackCounts {
        reference,
        paired,
        agreeing,
        lost,
        added,
    } = comparisons.iter().map(|comparison| comparison.tracks.counts()).sum();
    writeln!(
        out,
        "{} videos\ttracks {reference}\tpaired {paired} (agreeing {agreeing})\tlost {lost}\tadded {added}",
        comparisons.len()
    )?;
    writeln!(out, "detection gaps, grouped")?;
    for comparison in comparisons {
        comparison.detections.write_groups(out)?;
    }
    writeln!(out, "track gaps")?;
    for comparison in comparisons {
        comparison.tracks.write_discrepancies(out)?;
    }
    writeln!(out, "detection times, with the other videos running at the same time")?;
    for comparison in comparisons {
        writeln!(out, "{}\t{:.1} s", comparison.detections.name(), comparison.seconds)?;
    }
    let below: Vec<&str> = comparisons
        .iter()
        .filter(|comparison| !comparison.detections.holds())
        .map(|comparison| comparison.detections.name())
        .collect();
    writeln!(out, "under {} % either way: {}", percent(MIN_RATE), below.len())?;
    for name in below {
        writeln!(out, "  {name}")?;
    }
    Ok(())
}

fn percent(rate: f64) -> f64 {
    rate * 100.0
}

pub fn write_refusals(refusals: &[Refusal], out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "refused, the engine cannot handle them yet: {}", refusals.len())?;
    for Refusal { recording, reason } in refusals {
        let name = recording.file_name().unwrap_or(recording.as_os_str());
        writeln!(out, "  {}: {reason}", name.display())?;
    }
    Ok(())
}

pub fn write_failure(recording: &Path, error: &dyn Error, out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "{}: {error}", recording.display())?;
    let mut cause = error.source();
    while let Some(source) = cause {
        writeln!(out, "  {source}")?;
        cause = source.source();
    }
    Ok(())
}
