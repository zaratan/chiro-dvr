use types::language::Language;

#[cfg(test)]
mod tests;

const BYTE_ORDER_MARK: &str = "\u{feff}";
const QUOTE: char = '"';
const POINT: char = '.';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Dialect {
    start: &'static str,
    separator: char,
    decimal_mark: char,
}

const FRENCH: Dialect = Dialect {
    start: BYTE_ORDER_MARK,
    separator: ';',
    decimal_mark: ',',
};

const ENGLISH: Dialect = Dialect {
    start: "",
    separator: ',',
    decimal_mark: POINT,
};

impl Dialect {
    pub(crate) const fn of(language: Language) -> Self {
        match language {
            Language::French => FRENCH,
            Language::English => ENGLISH,
        }
    }

    pub(crate) const fn separator(self) -> char {
        self.separator
    }

    pub(crate) fn header(self, cells: impl IntoIterator<Item = String>) -> String {
        format!("{}{}", self.start, self.row(cells))
    }

    pub(crate) fn row(self, cells: impl IntoIterator<Item = String>) -> String {
        cells
            .into_iter()
            .map(|cell| self.quoted(cell))
            .collect::<Vec<_>>()
            .join(&self.separator.to_string())
    }

    pub(crate) fn decimal(self, value: f64, places: i32) -> String {
        let scale = 10_f64.powi(places);
        let rounded = (value * scale).round() / scale + 0.0;
        let text = rounded.to_string();
        let text = if rounded.is_finite() && !text.contains(POINT) {
            text + ".0"
        } else {
            text
        };
        text.replace(POINT, &self.decimal_mark.to_string())
    }

    fn quoted(self, cell: String) -> String {
        if cell.contains([self.separator, QUOTE, '\r', '\n']) {
            format!("{QUOTE}{}{QUOTE}", cell.replace(QUOTE, "\"\""))
        } else {
            cell
        }
    }
}
