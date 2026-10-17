#[cfg(test)]
mod tests;

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const PAD: char = '=';
const SIXTEEN_BITS: u32 = 16;
const EIGHT_BITS: u32 = 8;
const SIX_BITS: u32 = 6;
const LOW_SIX: u32 = 0b11_1111;

pub(crate) fn encoded(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let byte = |index: usize| u32::from(chunk.get(index).copied().unwrap_or_default());
        let word = byte(0) << SIXTEEN_BITS | byte(1) << EIGHT_BITS | byte(2);
        for index in 0..4 {
            if index <= chunk.len() {
                let shift = SIX_BITS * (3 - index as u32);
                out.push(char::from(ALPHABET[((word >> shift) & LOW_SIX) as usize]));
            } else {
                out.push(PAD);
            }
        }
    }
    out
}
