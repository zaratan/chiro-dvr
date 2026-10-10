#[derive(Clone, Copy)]
struct Tap {
    di: usize,
    si: usize,
    alpha: f32,
}

fn area_taps(ssize: usize, dsize: usize, cn: usize, scale: f64) -> Vec<Tap> {
    let mut tab = Vec::new();
    for dx in 0..dsize {
        let fsx1 = dx as f64 * scale;
        let fsx2 = fsx1 + scale;
        let cell = scale.min(ssize as f64 - fsx1);
        let mut sx1 = fsx1.ceil() as i64;
        let mut sx2 = fsx2.floor() as i64;
        sx2 = sx2.min(ssize as i64 - 1);
        sx1 = sx1.min(sx2);
        if sx1 as f64 - fsx1 > 1e-3 {
            tab.push(Tap { di: dx * cn, si: (sx1 - 1) as usize * cn, alpha: ((sx1 as f64 - fsx1) / cell) as f32 });
        }
        for sx in sx1..sx2 {
            tab.push(Tap { di: dx * cn, si: sx as usize * cn, alpha: (1.0 / cell) as f32 });
        }
        if fsx2 - sx2 as f64 > 1e-3 {
            tab.push(Tap {
                di: dx * cn,
                si: sx2 as usize * cn,
                alpha: ((fsx2 - sx2 as f64).min(1.0).min(cell) / cell) as f32,
            });
        }
    }
    tab
}

fn madd(a: f32, b: f32, c: f32, fused: bool) -> f32 {
    if fused { a.mul_add(b, c) } else { a * b + c }
}

fn round_u8(v: f32) -> u8 {
    v.round_ties_even().clamp(0.0, 255.0) as u8
}

/// OpenCV `resize(..., INTER_AREA)` for a non-integer shrink of a 3-channel 8-bit image.
pub fn resize_area_bgr(src: &[u8], sw: usize, sh: usize, dw: usize, dh: usize, fused: bool) -> Vec<u8> {
    let cn = 3;
    let scale_x = 1.0 / (dw as f64 / sw as f64);
    let scale_y = 1.0 / (dh as f64 / sh as f64);
    let xtab = area_taps(sw, dw, cn, scale_x);
    let ytab = area_taps(sh, dh, 1, scale_y);
    let width = dw * cn;
    let mut dst = vec![0u8; width * dh];
    let mut buf = vec![0f32; width];
    let mut sum = vec![0f32; width];
    let mut prev_dy = ytab[0].di;
    for t in &ytab {
        let (beta, dy, sy) = (t.alpha, t.di, t.si);
        let row = &src[sy * sw * cn..(sy + 1) * sw * cn];
        buf.iter_mut().for_each(|b| *b = 0.0);
        for k in &xtab {
            for c in 0..cn {
                buf[k.di + c] = madd(f32::from(row[k.si + c]), k.alpha, buf[k.di + c], fused);
            }
        }
        if dy != prev_dy {
            let out = &mut dst[prev_dy * width..(prev_dy + 1) * width];
            for dx in 0..width {
                out[dx] = round_u8(sum[dx]);
                sum[dx] = beta * buf[dx];
            }
            prev_dy = dy;
        } else {
            for dx in 0..width {
                sum[dx] = madd(beta, buf[dx], sum[dx], fused);
            }
        }
    }
    let out = &mut dst[prev_dy * width..(prev_dy + 1) * width];
    for dx in 0..width {
        out[dx] = round_u8(sum[dx]);
    }
    dst
}

/// OpenCV 5.0 `cvtColor(..., COLOR_BGR2GRAY)` for 8-bit images: `BY15/GY15/RY15` of `color_rgb.simd.hpp`, 15 bits.
pub fn bgr_to_gray(src: &[u8]) -> Vec<u8> {
    const B: u32 = 3735;
    const G: u32 = 19235;
    const R: u32 = 9798;
    src.chunks_exact(3)
        .map(|p| ((u32::from(p[0]) * B + u32::from(p[1]) * G + u32::from(p[2]) * R + (1 << 14)) >> 15) as u8)
        .collect()
}

fn reflect101(i: isize, n: isize) -> usize {
    if n == 1 {
        return 0;
    }
    let mut i = i;
    loop {
        if i < 0 {
            i = -i;
        } else if i >= n {
            i = 2 * n - 2 - i;
        } else {
            return i as usize;
        }
    }
}

/// OpenCV separable float filter with a symmetric kernel, BORDER_REFLECT_101, as its NEON path does it:
/// row pass `x0*k0` then fused multiply-adds, column pass `fma(below + above, k, acc)` from the centre.
pub fn gaussian_blur_f32(src: &[u8], w: usize, h: usize, kernel: &[f32]) -> Vec<f32> {
    let ksize = kernel.len();
    let half = ksize / 2;
    let mut rows = vec![0f32; w * h];
    let mut padded = vec![0f32; w + 2 * half];
    for y in 0..h {
        let line = &src[y * w..(y + 1) * w];
        for (i, p) in padded.iter_mut().enumerate() {
            *p = f32::from(line[reflect101(i as isize - half as isize, w as isize)]);
        }
        let out = &mut rows[y * w..(y + 1) * w];
        for x in 0..w {
            let mut s = padded[x] * kernel[0];
            for k in 1..ksize {
                s = padded[x + k].mul_add(kernel[k], s);
            }
            out[x] = s;
        }
    }
    let mut dst = vec![0f32; w * h];
    let centre = &kernel[half..];
    for y in 0..h {
        let row = |d: isize| reflect101(y as isize + d, h as isize) * w;
        let mid = row(0);
        let out = &mut dst[y * w..(y + 1) * w];
        for x in 0..w {
            out[x] = rows[mid + x].mul_add(centre[0], 0.0);
        }
        for k in 1..=half {
            let (below, above) = (row(k as isize), row(-(k as isize)));
            for x in 0..w {
                out[x] = (rows[below + x] + rows[above + x]).mul_add(centre[k], out[x]);
            }
        }
    }
    dst
}

const PW_BLOCKSIZE: usize = 128;

/// numpy's pairwise summation for float32 (`FLOAT_pairwise_sum`).
pub fn pairwise_sum_f32(a: &[f32]) -> f32 {
    let n = a.len();
    if n < 8 {
        let mut res = -0.0f32;
        for v in a {
            res += v;
        }
        res
    } else if n <= PW_BLOCKSIZE {
        let mut r = [0f32; 8];
        r.copy_from_slice(&a[..8]);
        let mut i = 8;
        while i < n - (n % 8) {
            for j in 0..8 {
                r[j] += a[i + j];
            }
            i += 8;
        }
        let mut res = ((r[0] + r[1]) + (r[2] + r[3])) + ((r[4] + r[5]) + (r[6] + r[7]));
        while i < n {
            res += a[i];
            i += 1;
        }
        res
    } else {
        let mut n2 = n / 2;
        n2 -= n2 % 8;
        pairwise_sum_f32(&a[..n2]) + pairwise_sum_f32(&a[n2..])
    }
}

pub fn mean_f32(a: &[f32]) -> f64 {
    f64::from((-0.0f32 + pairwise_sum_f32(a)) / a.len() as f32)
}

/// OpenCV `getStructuringElement(MORPH_ELLIPSE, (size, size))` as (dy, dx) offsets from the centre.
pub fn ellipse_offsets(size: usize) -> Vec<(isize, isize)> {
    let r = (size / 2) as isize;
    let c = r;
    let inv_r2 = if r > 0 { 1.0 / (r * r) as f64 } else { 0.0 };
    let mut offsets = Vec::new();
    for i in 0..size as isize {
        let dy = i - c;
        if dy.abs() > r {
            continue;
        }
        let dx = (c as f64 * (((r * r - dy * dy) as f64) * inv_r2).sqrt()).round_ties_even() as isize;
        let j1 = (c - dx).max(0);
        let j2 = (c + dx + 1).min(size as isize);
        for j in j1..j2 {
            offsets.push((dy, j - c));
        }
    }
    offsets
}

fn morph(src: &[u8], w: usize, h: usize, offsets: &[(isize, isize)], dilate: bool) -> Vec<u8> {
    let mut dst = vec![0u8; w * h];
    for y in 0..h as isize {
        for x in 0..w as isize {
            let mut v = if dilate { 0u8 } else { u8::MAX };
            for &(dy, dx) in offsets {
                let (yy, xx) = (y + dy, x + dx);
                let s = if yy < 0 || xx < 0 || yy >= h as isize || xx >= w as isize {
                    if dilate { 0 } else { u8::MAX }
                } else {
                    src[yy as usize * w + xx as usize]
                };
                v = if dilate { v.max(s) } else { v.min(s) };
            }
            dst[y as usize * w + x as usize] = v;
        }
    }
    dst
}

pub fn morph_close_dense(src: &[u8], w: usize, h: usize, size: usize) -> Vec<u8> {
    let offsets = ellipse_offsets(size);
    morph(&morph(src, w, h, &offsets, true), w, h, &offsets, false)
}

/// Same result as `morph_close_dense`, in time proportional to the set pixels: a binary close only
/// keeps pixels of the dilation, so the erosion is evaluated there alone.
pub fn morph_close(src: &[u8], w: usize, h: usize, size: usize) -> Vec<u8> {
    let offsets = ellipse_offsets(size);
    let (wi, hi) = (w as isize, h as isize);
    let mut dilated = vec![0u8; w * h];
    let mut touched = Vec::new();
    for (p, _) in src.iter().enumerate().filter(|(_, v)| **v != 0) {
        let (y, x) = ((p / w) as isize, (p % w) as isize);
        for &(dy, dx) in &offsets {
            let (yy, xx) = (y + dy, x + dx);
            if yy >= 0 && xx >= 0 && yy < hi && xx < wi {
                let q = yy as usize * w + xx as usize;
                if dilated[q] == 0 {
                    dilated[q] = 1;
                    touched.push(q);
                }
            }
        }
    }
    let mut closed = vec![0u8; w * h];
    for &q in &touched {
        let (y, x) = ((q / w) as isize, (q % w) as isize);
        let kept = offsets.iter().all(|&(dy, dx)| {
            let (yy, xx) = (y + dy, x + dx);
            yy < 0 || xx < 0 || yy >= hi || xx >= wi || dilated[yy as usize * w + xx as usize] != 0
        });
        closed[q] = u8::from(kept);
    }
    closed
}

pub struct Components {
    pub labels: Vec<i32>,
    pub count: usize,
    pub left: Vec<i64>,
    pub top: Vec<i64>,
    pub width: Vec<i64>,
    pub height: Vec<i64>,
    pub area: Vec<i64>,
    pub cx: Vec<f64>,
    pub cy: Vec<f64>,
}

fn find(parent: &mut [usize], mut i: usize) -> usize {
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i];
    }
    i
}

/// 8-connected components numbered like OpenCV's Spaghetti labelling: by the first 2×2 block, in block raster order.
pub fn components(src: &[u8], w: usize, h: usize) -> Components {
    let mut provisional = vec![0usize; w * h];
    let mut parent = vec![0usize];
    for y in 0..h {
        for x in 0..w {
            if src[y * w + x] == 0 {
                continue;
            }
            let mut found = 0usize;
            let neighbours = [(-1isize, -1isize), (-1, 0), (-1, 1), (0, -1)];
            for (dy, dx) in neighbours {
                let (yy, xx) = (y as isize + dy, x as isize + dx);
                if yy < 0 || xx < 0 || xx >= w as isize {
                    continue;
                }
                let l = provisional[yy as usize * w + xx as usize];
                if l == 0 {
                    continue;
                }
                if found == 0 {
                    found = l;
                } else {
                    let (a, b) = (find(&mut parent, found), find(&mut parent, l));
                    if a != b {
                        parent[a.max(b)] = a.min(b);
                    }
                }
            }
            if found == 0 {
                found = parent.len();
                parent.push(found);
            }
            provisional[y * w + x] = found;
        }
    }
    let roots = parent.len();
    let blocks_w = w.div_ceil(2);
    let mut first_block = vec![usize::MAX; roots];
    for y in 0..h {
        for x in 0..w {
            let l = provisional[y * w + x];
            if l != 0 {
                let r = find(&mut parent, l);
                let block = (y / 2) * blocks_w + x / 2;
                first_block[r] = first_block[r].min(block);
            }
        }
    }
    let mut order: Vec<usize> = (1..roots).filter(|&r| find(&mut parent, r) == r).collect();
    order.sort_by_key(|&r| first_block[r]);
    let mut final_label = vec![0i32; roots];
    for (k, &r) in order.iter().enumerate() {
        final_label[r] = k as i32 + 1;
    }
    let count = order.len() + 1;
    let mut c = Components {
        labels: vec![0; w * h],
        count,
        left: vec![i64::MAX; count],
        top: vec![i64::MAX; count],
        width: vec![0; count],
        height: vec![0; count],
        area: vec![0; count],
        cx: vec![0.0; count],
        cy: vec![0.0; count],
    };
    let (mut right, mut bottom) = (vec![i64::MIN; count], vec![i64::MIN; count]);
    let (mut sx, mut sy) = (vec![0u64; count], vec![0u64; count]);
    for y in 0..h {
        for x in 0..w {
            let l = provisional[y * w + x];
            let label = if l == 0 { 0 } else { final_label[find(&mut parent, l)] as usize };
            c.labels[y * w + x] = label as i32;
            c.left[label] = c.left[label].min(x as i64);
            c.top[label] = c.top[label].min(y as i64);
            right[label] = right[label].max(x as i64);
            bottom[label] = bottom[label].max(y as i64);
            c.area[label] += 1;
            sx[label] += x as u64;
            sy[label] += y as u64;
        }
    }
    for l in 0..count {
        c.width[l] = right[l] - c.left[l] + 1;
        c.height[l] = bottom[l] - c.top[l] + 1;
        c.cx[l] = sx[l] as f64 / c.area[l] as f64;
        c.cy[l] = sy[l] as f64 / c.area[l] as f64;
    }
    c
}

#[derive(Debug, Clone, PartialEq)]
pub struct Blob {
    pub x: f64,
    pub y: f64,
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    pub area: f64,
    pub amplitude: f64,
}

pub struct BlobConfig {
    pub min_area: f64,
    pub max_area: f64,
    pub merge_radius: f64,
}

/// `detect.find_blobs` with a per-pixel threshold map.
pub fn find_blobs(residual: &[f64], thresholds: &[f64], w: usize, h: usize, cfg: &BlobConfig, scale: f64) -> Vec<Blob> {
    let magnitude: Vec<f64> = residual.iter().map(|r| r.abs()).collect();
    let above: Vec<u8> = magnitude.iter().zip(thresholds).map(|(m, t)| u8::from(m > t)).collect();
    let merge = (cfg.merge_radius / scale).round_ties_even() as usize;
    let merged = if merge > 0 { morph_close(&above, w, h, 2 * merge + 1) } else { above.clone() };
    let comp = components(&merged, w, h);
    let mut counts = vec![0u64; comp.count];
    let mut peaks = vec![0f64; comp.count];
    for p in 0..w * h {
        if above[p] != 0 {
            let l = comp.labels[p] as usize;
            counts[l] += 1;
            peaks[l] = peaks[l].max(magnitude[p]);
        }
    }
    let mut found = Vec::new();
    for i in 1..comp.count {
        let area = counts[i] as f64 * scale.powi(2);
        if !(cfg.min_area <= area && area <= cfg.max_area) {
            continue;
        }
        found.push(Blob {
            x: (comp.cx[i] + 0.5) * scale - 0.5,
            y: (comp.cy[i] + 0.5) * scale - 0.5,
            left: comp.left[i] as f64 * scale,
            top: comp.top[i] as f64 * scale,
            width: comp.width[i] as f64 * scale,
            height: comp.height[i] as f64 * scale,
            area,
            amplitude: peaks[i],
        });
    }
    found
}

/// `resize_area_bgr` computed destination row by destination row, rows in parallel. Same arithmetic per pixel.
pub fn resize_area_bgr_par(src: &[u8], sw: usize, sh: usize, dw: usize, dh: usize, fused: bool) -> Vec<u8> {
    use rayon::prelude::*;
    let cn = 3;
    let xtab = area_taps(sw, dw, cn, 1.0 / (dw as f64 / sw as f64));
    let ytab = area_taps(sh, dh, 1, 1.0 / (dh as f64 / sh as f64));
    let width = dw * cn;
    let mut rows: Vec<Vec<(usize, f32)>> = vec![Vec::new(); dh];
    for t in &ytab {
        rows[t.di].push((t.si, t.alpha));
    }
    let mut dst = vec![0u8; width * dh];
    dst.par_chunks_mut(width).zip(rows.par_iter()).for_each(|(out, taps)| {
        let (mut buf, mut sum) = (vec![0f32; width], vec![0f32; width]);
        for (j, &(sy, beta)) in taps.iter().enumerate() {
            let row = &src[sy * sw * cn..(sy + 1) * sw * cn];
            buf.iter_mut().for_each(|b| *b = 0.0);
            for k in &xtab {
                for c in 0..cn {
                    buf[k.di + c] = madd(f32::from(row[k.si + c]), k.alpha, buf[k.di + c], fused);
                }
            }
            if j == 0 {
                for (s, b) in sum.iter_mut().zip(&buf) {
                    *s = beta * b;
                }
            } else {
                for (s, b) in sum.iter_mut().zip(&buf) {
                    *s = madd(beta, *b, *s, fused);
                }
            }
        }
        for (o, s) in out.iter_mut().zip(&sum) {
            *o = round_u8(*s);
        }
    });
    dst
}

/// `gaussian_blur_f32` with the taps in the outer loop so that each pass vectorises, rows in parallel.
pub fn gaussian_blur_f32_par(src: &[u8], w: usize, h: usize, kernel: &[f32]) -> Vec<f32> {
    use rayon::prelude::*;
    let ksize = kernel.len();
    let half = ksize / 2;
    let mut rows = vec![0f32; w * h];
    rows.par_chunks_mut(w).enumerate().for_each(|(y, out)| {
        let line = &src[y * w..(y + 1) * w];
        let padded: Vec<f32> =
            (0..w + 2 * half).map(|i| f32::from(line[reflect101(i as isize - half as isize, w as isize)])).collect();
        for (o, p) in out.iter_mut().zip(&padded) {
            *o = p * kernel[0];
        }
        for k in 1..ksize {
            for (o, p) in out.iter_mut().zip(&padded[k..]) {
                *o = p.mul_add(kernel[k], *o);
            }
        }
    });
    let centre = &kernel[half..];
    let mut dst = vec![0f32; w * h];
    dst.par_chunks_mut(w).enumerate().for_each(|(y, out)| {
        let row = |d: isize| &rows[reflect101(y as isize + d, h as isize) * w..][..w];
        for (o, m) in out.iter_mut().zip(row(0)) {
            *o = m.mul_add(centre[0], 0.0);
        }
        for k in 1..=half {
            let (below, above) = (row(k as isize), row(-(k as isize)));
            for ((o, b), a) in out.iter_mut().zip(below).zip(above) {
                *o = (b + a).mul_add(centre[k], *o);
            }
        }
    });
    dst
}

/// `components` restricted to the set pixels, listed in raster order: same numbering and statistics
/// for every label except the background, which `find_blobs` never reads.
pub fn components_sparse(set: &[usize], w: usize, h: usize, labels: &mut [u32]) -> Components {
    let mut parent: Vec<usize> = vec![0];
    for &p in set {
        let (y, x) = (p / w, p % w);
        let mut found = 0usize;
        let look = |q: usize, found: &mut usize, parent: &mut Vec<usize>| {
            let l = labels[q] as usize;
            if l == 0 {
                return;
            }
            if *found == 0 {
                *found = l;
            } else {
                let (a, b) = (find(parent, *found), find(parent, l));
                if a != b {
                    parent[a.max(b)] = a.min(b);
                }
            }
        };
        if y > 0 {
            if x > 0 {
                look(p - w - 1, &mut found, &mut parent);
            }
            look(p - w, &mut found, &mut parent);
            if x + 1 < w {
                look(p - w + 1, &mut found, &mut parent);
            }
        }
        if x > 0 {
            look(p - 1, &mut found, &mut parent);
        }
        if found == 0 {
            found = parent.len();
            parent.push(found);
        }
        labels[p] = found as u32;
    }
    let roots = parent.len();
    let blocks_w = w.div_ceil(2);
    let mut first_block = vec![usize::MAX; roots];
    for &p in set {
        let r = find(&mut parent, labels[p] as usize);
        first_block[r] = first_block[r].min((p / w / 2) * blocks_w + (p % w) / 2);
    }
    let mut order: Vec<usize> = (1..roots).filter(|&r| find(&mut parent, r) == r).collect();
    order.sort_by_key(|&r| first_block[r]);
    let mut final_label = vec![0u32; roots];
    for (k, &r) in order.iter().enumerate() {
        final_label[r] = k as u32 + 1;
    }
    let count = order.len() + 1;
    let mut c = Components {
        labels: Vec::new(),
        count,
        left: vec![i64::MAX; count],
        top: vec![i64::MAX; count],
        width: vec![0; count],
        height: vec![0; count],
        area: vec![0; count],
        cx: vec![0.0; count],
        cy: vec![0.0; count],
    };
    let (mut right, mut bottom) = (vec![i64::MIN; count], vec![i64::MIN; count]);
    let (mut sx, mut sy) = (vec![0u64; count], vec![0u64; count]);
    for &p in set {
        let label = final_label[find(&mut parent, labels[p] as usize)] as usize;
        labels[p] = label as u32;
        let (x, y) = ((p % w) as i64, (p / w) as i64);
        c.left[label] = c.left[label].min(x);
        c.top[label] = c.top[label].min(y);
        right[label] = right[label].max(x);
        bottom[label] = bottom[label].max(y);
        c.area[label] += 1;
        sx[label] += x as u64;
        sy[label] += y as u64;
    }
    for l in 1..count {
        c.width[l] = right[l] - c.left[l] + 1;
        c.height[l] = bottom[l] - c.top[l] + 1;
        c.cx[l] = sx[l] as f64 / c.area[l] as f64;
        c.cy[l] = sy[l] as f64 / c.area[l] as f64;
    }
    let _ = h;
    c
}

/// `find_blobs` touching only the pixels above the threshold and their closing.
pub fn find_blobs_sparse(
    residual: &[f64],
    thresholds: &[f64],
    w: usize,
    h: usize,
    cfg: &BlobConfig,
    scale: f64,
) -> Vec<Blob> {
    let above: Vec<u8> = residual.iter().zip(thresholds).map(|(r, t)| u8::from(r.abs() > *t)).collect();
    let merge = (cfg.merge_radius / scale).round_ties_even() as usize;
    let merged = if merge > 0 { morph_close(&above, w, h, 2 * merge + 1) } else { above.clone() };
    let set: Vec<usize> = merged.iter().enumerate().filter(|(_, v)| **v != 0).map(|(p, _)| p).collect();
    let mut labels = vec![0u32; w * h];
    let comp = components_sparse(&set, w, h, &mut labels);
    let mut counts = vec![0u64; comp.count];
    let mut peaks = vec![0f64; comp.count];
    for (p, a) in above.iter().enumerate() {
        if *a != 0 {
            let l = labels[p] as usize;
            counts[l] += 1;
            peaks[l] = peaks[l].max(residual[p].abs());
        }
    }
    let mut found = Vec::new();
    for i in 1..comp.count {
        let area = counts[i] as f64 * scale.powi(2);
        if !(cfg.min_area <= area && area <= cfg.max_area) {
            continue;
        }
        found.push(Blob {
            x: (comp.cx[i] + 0.5) * scale - 0.5,
            y: (comp.cy[i] + 0.5) * scale - 0.5,
            left: comp.left[i] as f64 * scale,
            top: comp.top[i] as f64 * scale,
            width: comp.width[i] as f64 * scale,
            height: comp.height[i] as f64 * scale,
            area,
            amplitude: peaks[i],
        });
    }
    found
}
