use std::error::Error;
use std::io::{self, Write};
use std::path::Path;

use crate::agreement::DetectionCounts;
use crate::comparison::Comparison;
use crate::sameness::{Sameness, Verdict};
use crate::tracks::TrackCounts;

#[cfg(test)]
mod tests;

pub fn write_report(comparisons: &[Comparison], out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "inputs, the same frames, timestamps and key frames on both sides")?;
    for comparison in comparisons {
        write_inputs(comparison, out)?;
    }
    writeln!(out, "detections, outside the damaged spans of both readings")?;
    for comparison in comparisons {
        comparison.detections.write_summary(out)?;
    }
    let DetectionCounts {
        differing_frames,
        lost,
        added,
        ..
    } = comparisons
        .iter()
        .map(|comparison| comparison.detections.counts())
        .sum();
    writeln!(
        out,
        "{} videos\tdiffering frames {differing_frames}\tlost {lost}\tadded {added}",
        comparisons.len()
    )?;
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
    let differing: Vec<&str> = comparisons
        .iter()
        .filter(|comparison| comparison.differs())
        .map(|comparison| comparison.detections.name())
        .collect();
    writeln!(out, "differing from the reference: {}", differing.len())?;
    for name in differing {
        writeln!(out, "  {name}")?;
    }
    Ok(())
}

fn write_inputs(comparison: &Comparison, out: &mut impl Write) -> io::Result<()> {
    let errors = comparison.decode_errors;
    writeln!(
        out,
        "{}\tdecode errors: reference {}, current {}, differing {}\tdamaged spans {} ({} frames)",
        comparison.detections.name(),
        errors.reference,
        errors.current,
        errors.differing,
        comparison.ignored.len(),
        comparison.ignored.iter().map(|span| span.frame_count()).sum::<usize>()
    )
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

pub fn write_failure(recording: &Path, error: &dyn Error, out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "{}: {error}", recording.display())?;
    let mut cause = error.source();
    while let Some(source) = cause {
        writeln!(out, "  {source}")?;
        cause = source.source();
    }
    Ok(())
}
