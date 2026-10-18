use std::cmp::Ordering;

use types::detection::Detection;
use types::track::Track;

use crate::error::Error;

#[cfg(test)]
mod tests;

pub(crate) fn are_twins(a: &Track, b: &Track, max_distance: f64) -> bool {
    if a.last().frame < b.first().frame || b.last().frame < a.first().frame {
        return false;
    }
    let (mut ours, mut theirs) = (a.points().iter().peekable(), b.points().iter().peekable());
    let mut shared = false;
    while let (Some(mine), Some(other)) = (ours.peek(), theirs.peek()) {
        match mine.frame.cmp(&other.frame) {
            Ordering::Less => {
                ours.next();
            }
            Ordering::Greater => {
                theirs.next();
            }
            Ordering::Equal => {
                if mine.distance(other) > max_distance {
                    return false;
                }
                shared = true;
                ours.next();
                theirs.next();
            }
        }
    }
    shared
}

pub(crate) fn combine(a: Detection, b: Detection) -> Detection {
    let left = a.left.min(b.left);
    let top = a.top.min(b.top);
    let right = (a.left + a.width).max(b.left + b.width);
    let bottom = (a.top + a.height).max(b.top + b.height);
    let area = a.area + b.area;
    Detection {
        frame: a.frame,
        x: (a.x * a.area + b.x * b.area) / area,
        y: (a.y * a.area + b.y * b.area) / area,
        left,
        top,
        width: right - left,
        height: bottom - top,
        area,
        amplitude: a.amplitude.max(b.amplitude),
        strength: a.strength.stronger(b.strength),
    }
}

pub(crate) fn merge_twins(mut tracks: Vec<Track>, max_distance: f64) -> Result<Vec<Track>, Error> {
    tracks.sort_by_key(|track| track.first().frame);
    let mut sets = DisjointSets::new(tracks.len());
    for (i, earlier) in tracks.iter().enumerate() {
        for (j, later) in tracks.iter().enumerate().skip(i + 1) {
            if later.first().frame > earlier.last().frame {
                break;
            }
            if are_twins(earlier, later, max_distance) {
                sets.join(i, j);
            }
        }
    }
    let mut groups: Vec<Vec<Track>> = Vec::new();
    let mut group_of_root: Vec<Option<usize>> = vec![None; tracks.len()];
    for (member, track) in tracks.into_iter().enumerate() {
        let root = sets.root(member);
        if let Some(group) = group_of_root[root] {
            groups[group].push(track);
        } else {
            group_of_root[root] = Some(groups.len());
            groups.push(vec![track]);
        }
    }
    groups.iter().map(|members| merged(members)).collect()
}

fn merged(members: &[Track]) -> Result<Track, Error> {
    let mut points: Vec<Detection> = members
        .iter()
        .flat_map(|track| track.points().iter().copied())
        .collect();
    points.sort_by_key(|point| point.frame);
    let combined = points
        .chunk_by(|a, b| a.frame == b.frame)
        .filter_map(|same_frame| same_frame.iter().copied().reduce(combine))
        .collect();
    Ok(Track::new(0, combined)?)
}

#[derive(Debug)]
struct DisjointSets {
    parent: Vec<usize>,
}

impl DisjointSets {
    fn new(count: usize) -> Self {
        Self {
            parent: (0..count).collect(),
        }
    }

    fn root(&mut self, mut member: usize) -> usize {
        while self.parent[member] != member {
            self.parent[member] = self.parent[self.parent[member]];
            member = self.parent[member];
        }
        member
    }

    fn join(&mut self, a: usize, b: usize) {
        let (a, b) = (self.root(a), self.root(b));
        self.parent[a.max(b)] = a.min(b);
    }
}
