//! Pixels above threshold to detections: binary closing with an ellipse, 8-connected
//! components, area and peak per component. Work proportional to the pixels involved.

use std::cell::RefCell;

use crate::kernel::Above;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blob {
    pub x: f64,
    pub y: f64,
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    pub area: f64,
    pub peak: f32,
}

pub struct BlobConfig {
    pub min_area: f64,
    pub max_area: f64,
    pub merge_radius: f64,
}

pub fn ellipse_offsets(size: usize) -> Vec<(isize, isize)> {
    let r = (size / 2) as isize;
    let inv_r2 = if r > 0 { 1.0 / (r * r) as f64 } else { 0.0 };
    let mut offsets = Vec::new();
    for dy in -r..=r {
        let dx = (r as f64 * (((r * r - dy * dy) as f64) * inv_r2).sqrt()).round_ties_even() as isize;
        for x in -dx..=dx {
            offsets.push((dy, x));
        }
    }
    offsets
}

struct Scratch {
    marks: Vec<u8>,
    labels: Vec<u32>,
}

thread_local! {
    static SCRATCH: RefCell<Scratch> = const { RefCell::new(Scratch { marks: Vec::new(), labels: Vec::new() }) };
}

fn find(parent: &mut [u32], mut i: u32) -> u32 {
    while parent[i as usize] != i {
        parent[i as usize] = parent[parent[i as usize] as usize];
        i = parent[i as usize];
    }
    i
}

pub fn find_blobs(above: &[Above], w: usize, h: usize, cfg: &BlobConfig, scale: f64) -> Vec<Blob> {
    if above.is_empty() {
        return Vec::new();
    }
    SCRATCH.with(|cell| {
        let mut scratch = cell.borrow_mut();
        let Scratch { marks, labels } = &mut *scratch;
        if marks.len() != w * h {
            *marks = vec![0; w * h];
            *labels = vec![0; w * h];
        }
        let merge = (cfg.merge_radius / scale).round_ties_even() as usize;
        let offsets = ellipse_offsets(2 * merge + 1);
        let (wi, hi) = (w as isize, h as isize);
        let mut touched: Vec<u32> = Vec::with_capacity(above.len() * offsets.len());
        for a in above {
            let p = a.pixel as usize;
            let (y, x) = ((p / w) as isize, (p % w) as isize);
            for &(dy, dx) in &offsets {
                let (yy, xx) = (y + dy, x + dx);
                if yy >= 0 && xx >= 0 && yy < hi && xx < wi {
                    let q = yy as usize * w + xx as usize;
                    if marks[q] == 0 {
                        marks[q] = 1;
                        touched.push(q as u32);
                    }
                }
            }
        }
        let mut closed: Vec<u32> = touched
            .iter()
            .copied()
            .filter(|&q| {
                let q = q as usize;
                let (y, x) = ((q / w) as isize, (q % w) as isize);
                offsets.iter().all(|&(dy, dx)| {
                    let (yy, xx) = (y + dy, x + dx);
                    yy < 0 || xx < 0 || yy >= hi || xx >= wi || marks[yy as usize * w + xx as usize] != 0
                })
            })
            .collect();
        closed.sort_unstable();
        let mut parent: Vec<u32> = vec![0];
        for &q in &closed {
            let p = q as usize;
            let (y, x) = (p / w, p % w);
            let mut found = 0u32;
            let mut look = |r: usize, found: &mut u32| {
                let l = labels[r];
                if l == 0 {
                    return;
                }
                if *found == 0 {
                    *found = l;
                } else {
                    let (a, b) = (find(&mut parent, *found), find(&mut parent, l));
                    if a != b {
                        parent[a.max(b) as usize] = a.min(b);
                    }
                }
            };
            if y > 0 {
                if x > 0 {
                    look(p - w - 1, &mut found);
                }
                look(p - w, &mut found);
                if x + 1 < w {
                    look(p - w + 1, &mut found);
                }
            }
            if x > 0 {
                look(p - 1, &mut found);
            }
            if found == 0 {
                found = parent.len() as u32;
                parent.push(found);
            }
            labels[p] = found;
        }
        let roots = parent.len();
        let mut index = vec![u32::MAX; roots];
        let mut count = 0usize;
        for &q in &closed {
            let r = find(&mut parent, labels[q as usize]) as usize;
            if index[r] == u32::MAX {
                index[r] = count as u32;
                count += 1;
            }
        }
        let mut stats = vec![(i64::MAX, i64::MAX, i64::MIN, i64::MIN, 0u64, 0u64, 0u64); count];
        for &q in &closed {
            let p = q as usize;
            let c = index[find(&mut parent, labels[p]) as usize] as usize;
            labels[p] = c as u32 + 1;
            let (x, y) = ((p % w) as i64, (p / w) as i64);
            let s = &mut stats[c];
            s.0 = s.0.min(x);
            s.1 = s.1.min(y);
            s.2 = s.2.max(x);
            s.3 = s.3.max(y);
            s.4 += 1;
            s.5 += x as u64;
            s.6 += y as u64;
        }
        let mut hits = vec![0u64; count];
        let mut peaks = vec![0f32; count];
        for a in above {
            let l = labels[a.pixel as usize];
            if l != 0 {
                let c = l as usize - 1;
                hits[c] += 1;
                peaks[c] = peaks[c].max(a.magnitude);
            }
        }
        for &q in &touched {
            marks[q as usize] = 0;
            labels[q as usize] = 0;
        }
        let mut found = Vec::new();
        for (c, s) in stats.iter().enumerate() {
            let area = hits[c] as f64 * scale * scale;
            if !(cfg.min_area <= area && area <= cfg.max_area) {
                continue;
            }
            let (cx, cy) = (s.5 as f64 / s.4 as f64, s.6 as f64 / s.4 as f64);
            found.push(Blob {
                x: (cx + 0.5) * scale - 0.5,
                y: (cy + 0.5) * scale - 0.5,
                left: s.0 as f64 * scale,
                top: s.1 as f64 * scale,
                width: (s.2 - s.0 + 1) as f64 * scale,
                height: (s.3 - s.1 + 1) as f64 * scale,
                area,
                peak: peaks[c],
            });
        }
        found
    })
}
