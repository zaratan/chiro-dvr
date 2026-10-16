use std::error::Error;
use std::io::{self, Write};
use std::path::Path;

use crate::agreement::MIN_RATE;
use crate::comparison::Comparison;
use crate::sameness::{Sameness, Verdict};
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

pub fn write_same_as(previous: &Path, comparisons: &[Comparison], out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "same as {}, outside the damaged spans", previous.display())?;
    let compared: Vec<&Sameness> = comparisons
        .iter()
        .filter_map(|comparison| comparison.same_as.as_ref())
        .collect();
    for sameness in &compared {
        sameness.write_line(out)?;
    }
    let identical = compared.iter().filter(|sameness| sameness.holds()).count();
    let inputs = compared
        .iter()
        .filter(|sameness| matches!(sameness.verdict(), Verdict::InputsDiffer(_)))
        .count();
    writeln!(
        out,
        "{} videos: identical {identical}, differing {}, inputs differ {inputs}",
        compared.len(),
        compared.len() - identical - inputs
    )
}

fn percent(rate: f64) -> f64 {
    rate * 100.0
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
