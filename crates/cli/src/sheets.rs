use std::path::Path;

use files::params::SummaryEntry;
use files::time::hundredths;

use crate::listing::relative;
use crate::overview::Sheet;

#[cfg(test)]
mod tests;

pub(crate) fn summaries(dest: &Path, sheets: &[Sheet]) -> Vec<SummaryEntry> {
    sheets
        .iter()
        .filter_map(|sheet| {
            Some(SummaryEntry {
                file: relative(dest, &sheet.page)?,
                window_s: sheet.window.map(hundredths),
                part: sheet.part.map(|part| [part.number, part.count]),
                ignored_s: ranges(&sheet.ignored),
                damaged_s: ranges(&sheet.damaged),
                other_starts_s: sheet.other_starts.iter().copied().map(hundredths).collect(),
                radius: sheet.radius,
            })
        })
        .collect()
}

fn ranges(ranges: &[[f64; 2]]) -> Vec<[f64; 2]> {
    ranges.iter().map(|range| range.map(hundredths)).collect()
}
