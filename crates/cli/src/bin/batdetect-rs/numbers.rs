#[cfg(test)]
mod tests;

pub(crate) const NARROW_NO_BREAK_SPACE: char = '\u{202f}';
pub(crate) const NO_BREAK_SPACE: char = '\u{a0}';
const GROUP: usize = 3;

pub(crate) fn english(text: &str) -> String {
    written(text, '.', None)
}

pub(crate) fn french(text: &str) -> String {
    written(text, ',', Some(NARROW_NO_BREAK_SPACE))
}

fn written(text: &str, decimal: char, thousands: Option<char>) -> String {
    let (sign, unsigned) = text.strip_prefix('-').map_or(("", text), |rest| ("-", rest));
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    if whole.is_empty() || !whole.chars().all(|digit| digit.is_ascii_digit()) {
        return text.to_owned();
    }
    let zero = whole.chars().all(|digit| digit == '0') && fraction.chars().all(|digit| digit == '0');
    let mut out = String::with_capacity(text.len() + whole.len() / GROUP * NARROW_NO_BREAK_SPACE.len_utf8());
    out.push_str(if zero { "" } else { sign });
    for (index, digit) in whole.chars().enumerate() {
        if index > 0 && (whole.len() - index).is_multiple_of(GROUP) {
            out.extend(thousands);
        }
        out.push(digit);
    }
    if !fraction.is_empty() {
        out.push(decimal);
        out.push_str(fraction);
    }
    out
}
