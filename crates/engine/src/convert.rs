#[cfg(test)]
mod tests;

pub(crate) fn numbered(pixel: usize) -> u32 {
    pixel as u32
}

pub(crate) fn real(count: usize) -> f64 {
    count as f64
}

pub(crate) fn wide(value: i128) -> f64 {
    value as f64
}

pub(crate) fn narrowed(value: f64) -> f32 {
    value as f32
}

pub(crate) fn position(value: f64) -> usize {
    value as usize
}
