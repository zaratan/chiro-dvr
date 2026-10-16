use crate::table::{Column, Headings, LANGUAGES};

#[cfg(test)]
mod tests;

const SEPARATORS: [char; 3] = [';', ',', '\t'];
const LINE_ENDS: [char; 2] = ['\r', '\n'];
const BYTE_ORDER_MARK: char = '\u{feff}';
const QUOTE: char = '"';

pub(crate) fn carries_verdicts(bytes: &[u8], headings: &dyn Headings) -> bool {
    let text = String::from_utf8_lossy(bytes);
    let mut lines = text.split(LINE_ENDS).filter(|line| !line.is_empty());
    let Some(header) = lines.next() else {
        return false;
    };
    let verdict = LANGUAGES.map(|language| headings.heading(Column::Verdict, language).to_lowercase());
    let named_verdict = |cell: &str| verdict.contains(&unquoted(cell.trim()).to_lowercase());
    let header = header.trim_start_matches(BYTE_ORDER_MARK);
    let column = SEPARATORS.into_iter().find_map(|separator| {
        header
            .split(separator)
            .position(named_verdict)
            .map(|index| (separator, index))
    });
    column.is_some_and(|(separator, index)| {
        lines.any(|line| {
            line.split(separator)
                .nth(index)
                .is_some_and(|cell| !unquoted(cell).is_empty())
        })
    })
}

fn unquoted(cell: &str) -> &str {
    cell.strip_prefix(QUOTE)
        .and_then(|inner| inner.strip_suffix(QUOTE))
        .unwrap_or(cell)
}
