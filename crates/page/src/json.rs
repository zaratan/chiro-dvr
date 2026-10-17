#[cfg(test)]
mod tests;

const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";
const NIBBLE_SHIFTS: [u32; 4] = [12, 8, 4, 0];
const LOW_NIBBLE: u32 = 0xf;

pub(crate) fn string(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() + 2);
    out.push('"');
    for character in raw.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            control if control.is_control() => {
                let code = u32::from(control);
                out.push_str("\\u");
                for shift in NIBBLE_SHIFTS {
                    out.push(char::from(HEX_DIGITS[((code >> shift) & LOW_NIBBLE) as usize]));
                }
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

pub(crate) fn object<'a>(pairs: impl IntoIterator<Item = (&'a str, String)>) -> String {
    let members: Vec<String> = pairs
        .into_iter()
        .map(|(key, value)| format!("{}:{}", string(key), string(&value)))
        .collect();
    format!("{{{}}}", members.join(","))
}
