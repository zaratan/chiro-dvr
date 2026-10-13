use reference::bench::TargetLine;
use types::frame::FrameIndex;

use crate::synthetic::injection::{Injected, Observation};
use crate::synthetic::{self, Class, Point, Target, UnknownMotion};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("recorded target {id} has an unknown motion")]
    Motion {
        id: usize,
        #[source]
        source: UnknownMotion,
    },
    #[error("recorded target {id} is not a valid target")]
    Target {
        id: usize,
        #[source]
        source: synthetic::Error,
    },
    #[error("recorded target {id} has a contrast at frame {frame}, where it has no position")]
    Observation { id: usize, frame: usize },
    #[error("recorded target {id} has its contrast at frame {frame} out of frame order")]
    Order { id: usize, frame: usize },
    #[error("recorded target {id} appears twice")]
    Duplicate { id: usize },
}

pub fn target(line: &TargetLine) -> Result<Target, Error> {
    let id = line.id;
    let motion = line.motion.parse().map_err(|source| Error::Motion { id, source })?;
    let invalid = |source| Error::Target { id, source };
    let class = Class::new(line.amplitude, line.sigma, motion).map_err(invalid)?;
    let positions = line.positions.iter().map(|&[x, y]| Point { x, y }).collect();
    Target::new(
        id,
        FrameIndex::new(line.start_frame),
        positions,
        line.amplitudes.clone(),
        class,
    )
    .map_err(invalid)
}

pub fn injected(line: &TargetLine) -> Result<Injected, Error> {
    let target = target(line)?;
    if let Some(pair) = line.effective.windows(2).find(|pair| pair[1].0 <= pair[0].0) {
        return Err(Error::Order {
            id: line.id,
            frame: pair[1].0,
        });
    }
    let observations = line
        .effective
        .iter()
        .map(|&(frame, effective)| {
            let frame = FrameIndex::new(frame);
            let at = target.position(frame).ok_or(Error::Observation {
                id: line.id,
                frame: frame.get(),
            })?;
            Ok(Observation { frame, at, effective })
        })
        .collect::<Result<_, Error>>()?;
    Ok(Injected { target, observations })
}

pub fn targets(lines: &[TargetLine]) -> Result<Vec<Target>, Error> {
    unique(lines)?;
    lines.iter().map(target).collect()
}

pub fn all_injected(lines: &[TargetLine]) -> Result<Vec<Injected>, Error> {
    unique(lines)?;
    lines.iter().map(injected).collect()
}

fn unique(lines: &[TargetLine]) -> Result<(), Error> {
    let mut ids: Vec<usize> = lines.iter().map(|line| line.id).collect();
    ids.sort_unstable();
    match ids.windows(2).find(|pair| pair[0] == pair[1]) {
        Some(pair) => Err(Error::Duplicate { id: pair[0] }),
        None => Ok(()),
    }
}

pub fn line(injected: &Injected) -> TargetLine {
    let target = &injected.target;
    let class = target.class();
    TargetLine {
        id: target.id(),
        motion: class.motion().name().to_owned(),
        amplitude: class.amplitude(),
        sigma: class.sigma(),
        start_frame: target.start().get(),
        positions: target.positions().iter().map(|point| [point.x, point.y]).collect(),
        amplitudes: target.amplitudes().to_vec(),
        effective: injected
            .observations
            .iter()
            .map(|observation| (observation.frame.get(), observation.effective))
            .collect(),
        patch: None,
    }
}
