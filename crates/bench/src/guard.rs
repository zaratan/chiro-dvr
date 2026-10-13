use reference::bench::{BenchReport, ReportedTarget};
use types::frame::FrameIndex;

use crate::metrics::{Evaluation, TargetResult};
use crate::stats::wilson;
use crate::synthetic::{self, Class, Target, UnknownMotion};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Change {
    pub id: usize,
    pub start: FrameIndex,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassGuard {
    pub class: Class,
    pub n: usize,
    pub v011_found: usize,
    pub found: usize,
    pub lowest: f64,
    pub lost: Vec<Change>,
    pub gained: Vec<Change>,
}

impl ClassGuard {
    pub fn failed(&self) -> bool {
        self.n > 0 && (self.found as f64 / self.n as f64) < self.lowest
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Guard {
    pub classes: Vec<ClassGuard>,
    pub v011_only: Vec<Change>,
    pub rust_only: Vec<Change>,
    pub contrast_differs: Vec<Change>,
    pub v011_false_tracks: usize,
    pub false_tracks: usize,
    pub v011_reference_tracks: usize,
    pub reference_tracks: usize,
}

impl Guard {
    pub fn failed(&self) -> bool {
        self.classes.iter().any(ClassGuard::failed)
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("target {id} of the recorded report has an unknown motion")]
    Motion {
        id: usize,
        #[source]
        source: UnknownMotion,
    },
    #[error("target {id} of the recorded report has an invalid class")]
    Class {
        id: usize,
        #[source]
        source: synthetic::Error,
    },
    #[error("target {id} of the recorded report is not among the replayed targets")]
    Unknown { id: usize },
}

pub fn compare(
    judged: &Evaluation,
    measured: &Evaluation,
    recorded: &BenchReport,
    targets: &[Target],
) -> Result<Guard, Error> {
    let change = |id: usize| {
        targets
            .iter()
            .find(|target| target.id() == id)
            .map(|target| Change {
                id,
                start: target.start(),
            })
            .ok_or(Error::Unknown { id })
    };
    if let Some(unknown) = recorded
        .bats
        .iter()
        .find(|reported| targets.iter().all(|target| target.id() != reported.id))
    {
        return Err(Error::Unknown { id: unknown.id });
    }
    let rust = |id: usize| judged.targets.iter().find(|result| result.id == id);
    let mut classes: Vec<ClassGuard> = Vec::new();
    let mut v011_only = Vec::new();
    for reported in &recorded.bats {
        let Some(result) = rust(reported.id) else {
            v011_only.push(change(reported.id)?);
            continue;
        };
        let class = recorded_class(reported)?;
        let guarded = class_entry(&mut classes, class);
        guarded.n += 1;
        guarded.v011_found += usize::from(reported.found);
        guarded.found += usize::from(result.found);
        match (reported.found, result.found) {
            (true, false) => guarded.lost.push(change(reported.id)?),
            (false, true) => guarded.gained.push(change(reported.id)?),
            _ => {}
        }
    }
    for guarded in &mut classes {
        guarded.lowest = wilson(guarded.v011_found, guarded.n).0;
    }
    classes.sort_by(|left, right| left.class.order(right.class));
    let rust_only = judged
        .targets
        .iter()
        .filter(|result| recorded.bats.iter().all(|reported| reported.id != result.id))
        .map(|result| change(result.id))
        .collect::<Result<_, _>>()?;
    let contrast_differs = targets
        .iter()
        .filter(|target| visible(judged, target.id()) != visible(measured, target.id()))
        .map(|target| change(target.id()))
        .collect::<Result<_, _>>()?;
    Ok(Guard {
        classes,
        v011_only,
        rust_only,
        contrast_differs,
        v011_false_tracks: recorded.false_tracks,
        false_tracks: judged.false_tracks,
        v011_reference_tracks: recorded.reference_tracks,
        reference_tracks: judged.reference_tracks,
    })
}

fn class_entry(classes: &mut Vec<ClassGuard>, class: Class) -> &mut ClassGuard {
    let index = classes
        .iter()
        .position(|guarded| guarded.class == class)
        .unwrap_or_else(|| {
            classes.push(ClassGuard {
                class,
                n: 0,
                v011_found: 0,
                found: 0,
                lowest: 0.0,
                lost: Vec::new(),
                gained: Vec::new(),
            });
            classes.len() - 1
        });
    &mut classes[index]
}

fn visible(evaluation: &Evaluation, id: usize) -> bool {
    evaluation.targets.iter().any(|result: &TargetResult| result.id == id)
}

fn recorded_class(reported: &ReportedTarget) -> Result<Class, Error> {
    let id = reported.id;
    let motion = reported.motion.parse().map_err(|source| Error::Motion { id, source })?;
    Class::new(reported.amplitude, reported.sigma, motion).map_err(|source| Error::Class { id, source })
}
