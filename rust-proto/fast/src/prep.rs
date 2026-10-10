//! Full-resolution luma plane to the blurred work frame, following batdetect's own roundings:
//! full-range gray of each source pixel (the table OpenCV's decoding gives on neutral pixels),
//! area reduction by 3:2 rounded to 8 bits, then OpenCV's Gaussian of sigma 1 work pixel
//! quantised to 4096 on each axis. One work unit is 1/64 of a gray level.

pub const UNITS_PER_LEVEL: u32 = 64;

const GRAY: [u8; 256] = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,2,3,4,5,6,8,9,10,11,12,13,15,16,17,18,19,20,22,23,24,25,26,27,28,30,31,32,33,34,35,37,38,39,40,41,42,44,45,46,47,48,49,51,52,53,54,55,56,58,59,60,61,62,63,65,66,67,68,69,70,72,73,74,75,76,77,79,80,81,82,83,84,86,87,88,89,90,91,93,94,95,96,97,98,100,101,102,103,104,105,107,108,109,110,111,112,113,115,116,117,118,119,120,122,123,124,125,126,127,129,130,131,132,133,134,136,137,138,139,140,141,143,144,145,146,147,148,150,151,152,153,154,155,157,158,159,160,161,162,164,165,166,167,168,169,171,172,173,174,175,176,178,179,180,181,182,183,185,186,187,188,189,190,192,193,194,195,196,197,198,200,201,202,203,204,205,207,208,209,210,211,212,214,215,216,217,218,219,221,222,223,224,225,226,228,229,230,231,232,233,235,236,237,238,239,240,242,243,244,245,246,247,249,250,251,252,253,254,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255];
const KERNEL: [u32; 9] = [1, 18, 221, 991, 1634, 991, 221, 18, 1];
const TAPS: usize = KERNEL.len();
const HALF: usize = TAPS / 2;
const SHIFT: u32 = 18;

pub fn work_size(width: usize, height: usize) -> Option<(usize, usize)> {
    (width % 3 == 0 && height % 3 == 0).then(|| (width / 3 * 2, height / 3 * 2))
}

fn area_row(src: &[u8], out: &mut [u16]) {
    for (o, s) in out.chunks_exact_mut(2).zip(src.chunks_exact(3)) {
        let (a, b, c) = (u16::from(GRAY[s[0] as usize]), u16::from(GRAY[s[1] as usize]), u16::from(GRAY[s[2] as usize]));
        o[0] = 2 * a + b;
        o[1] = b + 2 * c;
    }
}

fn reflect(i: isize, n: usize) -> usize {
    let n = n as isize;
    let i = if i < 0 { -i } else { i };
    (if i >= n { 2 * n - 2 - i } else { i }) as usize
}

fn blur_row(src: &[u8], out: &mut [u32]) {
    let w = src.len();
    for x in HALF..w - HALF {
        let s = &src[x - HALF..x + HALF + 1];
        let mut v = 0u32;
        for t in 0..TAPS {
            v += KERNEL[t] * u32::from(s[t]);
        }
        out[x] = v;
    }
    for x in (0..HALF).chain(w - HALF..w) {
        out[x] = (0..TAPS).map(|t| KERNEL[t] * u32::from(src[reflect(x as isize + t as isize - HALF as isize, w)])).sum();
    }
}

fn rounded_ninth(v: u16) -> u8 {
    ((v + 4) / 9) as u8
}

/// Full-resolution luma to the 8-bit work gray image, as batdetect's reading and reduction give it.
pub fn luma_to_gray(luma: &[u8], stride: usize, width: usize, height: usize, gray: &mut Vec<u8>) {
    let (dw, dh) = work_size(width, height).expect("frame size divisible by 3");
    gray.clear();
    gray.resize(dw * dh, 0);
    let (mut h0, mut h1, mut h2) = (vec![0u16; dw], vec![0u16; dw], vec![0u16; dw]);
    for k in 0..height / 3 {
        let line = |r: usize| &luma[r * stride..r * stride + width];
        area_row(line(3 * k), &mut h0);
        area_row(line(3 * k + 1), &mut h1);
        area_row(line(3 * k + 2), &mut h2);
        let (top, bottom) = gray[2 * k * dw..(2 * k + 2) * dw].split_at_mut(dw);
        for x in 0..dw {
            top[x] = rounded_ninth(2 * h0[x] + h1[x]);
            bottom[x] = rounded_ninth(h1[x] + 2 * h2[x]);
        }
    }
}

/// Work gray image to the blurred work frame; returns the sum of its values.
pub fn blur_gray(gray: &[u8], dw: usize, dh: usize, out: &mut Vec<u16>) -> u64 {
    let mut rows = vec![0u32; dw * dh];
    for y in 0..dh {
        blur_row(&gray[y * dw..(y + 1) * dw], &mut rows[y * dw..(y + 1) * dw]);
    }
    out.clear();
    out.resize(dw * dh, 0);
    let mut sum = 0u64;
    for y in 0..dh {
        let taps: [&[u32]; TAPS] =
            std::array::from_fn(|t| &rows[reflect(y as isize + t as isize - HALF as isize, dh) * dw..][..dw]);
        let o = &mut out[y * dw..(y + 1) * dw];
        let mut row_sum = 0u64;
        for x in 0..dw {
            let mut v = 0u32;
            for t in 0..TAPS {
                v += KERNEL[t] * taps[t][x];
            }
            let v = (v + (1 << (SHIFT - 1))) >> SHIFT;
            o[x] = v as u16;
            row_sum += u64::from(v);
        }
        sum += row_sum;
    }
    sum
}

/// Returns the sum of the work frame written to `out`.
pub fn prepare(luma: &[u8], stride: usize, width: usize, height: usize, out: &mut Vec<u16>) -> u64 {
    let (dw, dh) = work_size(width, height).expect("frame size divisible by 3");
    let mut gray = Vec::new();
    luma_to_gray(luma, stride, width, height, &mut gray);
    blur_gray(&gray, dw, dh, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_luma_stays_flat_at_64_units_per_gray_level() {
        let (w, h) = (24, 18);
        let mut out = Vec::new();
        let sum = prepare(&vec![126u8; w * h], w, w, h, &mut out);
        let level = u16::from(GRAY[126]) * 64;
        assert!(out.iter().all(|&v| v == level));
        assert_eq!(sum, u64::from(level) * 16 * 12);
    }

    #[test]
    fn brightest_luma_fits_in_sixteen_bits_without_overflow() {
        let (w, h) = (24, 18);
        let mut out = Vec::new();
        prepare(&vec![255u8; w * h], w, w, h, &mut out);
        assert!(out.iter().all(|&v| v == 255 * 64));
    }

    #[test]
    fn kernel_sums_to_one_in_fixed_point() {
        assert_eq!(KERNEL.iter().sum::<u32>(), 4096);
    }
}

/// OpenCV `resize(INTER_AREA)` of a BGR image by exactly 2/3, same float operations in the same
/// order as its generic area path (weights 2/3 and 1/3, row sums in float32, round half to even).
pub fn area_bgr_two_thirds(src: &[u8], width: usize, height: usize, out: &mut Vec<u8>) {
    let (dw, dh) = work_size(width, height).expect("frame size divisible by 3");
    let full = (1.0f64 / 1.5) as f32;
    let part = (0.5f64 / 1.5) as f32;
    let row_len = dw * 3;
    let horizontal = |row: &[u8], buf: &mut [f32]| {
        for (o, s) in buf.chunks_exact_mut(6).zip(row.chunks_exact(9)) {
            for c in 0..3 {
                let (a, b, cc) = (f32::from(s[c]), f32::from(s[3 + c]), f32::from(s[6 + c]));
                o[c] = 0.0 + a * full + b * part;
                o[3 + c] = 0.0 + b * part + cc * full;
            }
        }
    };
    out.clear();
    out.resize(row_len * dh, 0);
    let (mut b0, mut b1, mut b2) = (vec![0f32; row_len], vec![0f32; row_len], vec![0f32; row_len]);
    for k in 0..height / 3 {
        let line = |r: usize| &src[r * width * 3..(r + 1) * width * 3];
        horizontal(line(3 * k), &mut b0);
        horizontal(line(3 * k + 1), &mut b1);
        horizontal(line(3 * k + 2), &mut b2);
        let (top, bottom) = out[2 * k * row_len..(2 * k + 2) * row_len].split_at_mut(row_len);
        for x in 0..row_len {
            top[x] = (full * b0[x] + part * b1[x]).round_ties_even().clamp(0.0, 255.0) as u8;
            bottom[x] = (part * b1[x] + full * b2[x]).round_ties_even().clamp(0.0, 255.0) as u8;
        }
    }
}
