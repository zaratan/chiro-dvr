use std::fs;
use std::num::NonZeroUsize;
use std::path::Path;

use engine::input::Input;
use media::decode::Threads;
use types::frame::FrameIndex;

use crate::failure;
use crate::jobs::{Job, Reason, Refused};
use crate::reuse::Earlier;
use crate::space::{Need, Volume, launch_estimate, short_volumes};

#[cfg(test)]
mod tests;

const ONE_THREAD: Threads = Threads::Exactly(NonZeroUsize::MIN);

#[derive(Debug, Clone, PartialEq)]
pub struct Listed {
    pub job: Job,
    pub earlier: Option<Earlier>,
    pub length_s: f64,
}

#[derive(Debug, Default)]
pub struct Stock {
    pub listed: Vec<Listed>,
    pub refused: Vec<Refused>,
}

impl Stock {
    pub fn jobs(&self) -> Vec<Job> {
        self.listed.iter().map(|listed| listed.job.clone()).collect()
    }

    pub fn length_s(&self) -> f64 {
        self.listed.iter().map(|listed| listed.length_s).sum()
    }

    pub fn already_done(&self) -> usize {
        self.listed.iter().filter(|listed| listed.earlier.is_some()).count()
    }

    pub fn needs(&self, annotated: bool) -> Vec<Need> {
        self.listed
            .iter()
            .filter(|listed| listed.earlier.is_none())
            .map(|listed| Need {
                folder: listed.job.dest.parent().unwrap_or(&listed.job.dest).to_path_buf(),
                bytes: launch_estimate(
                    fs::metadata(&listed.job.video).map_or(0, |found| found.len()),
                    annotated,
                ),
            })
            .collect()
    }

    pub fn refuse_short_disks(&mut self, annotated: bool, volume_of: impl Fn(&Path) -> Option<Volume>) {
        let shorts = short_volumes(&self.needs(annotated), volume_of);
        self.refused.extend(shorts.into_iter().map(|short| Refused {
            path: short.folder,
            reason: Reason::DiskShort {
                needed: short.needed,
                free: short.free,
                annotated,
            },
        }));
    }

    pub fn go_ahead_despite_the_disks(&mut self) {
        self.refused
            .retain(|refusal| !matches!(refusal.reason, Reason::DiskShort { .. }));
    }

    pub fn earlier(&self, job: &Job) -> Option<Earlier> {
        self.listed
            .iter()
            .find(|listed| listed.job == *job)
            .and_then(|listed| listed.earlier)
    }
}

pub fn take_stock(jobs: Vec<Job>, earlier: impl Fn(&Job) -> Option<Earlier>) -> Stock {
    let mut stock = Stock::default();
    for job in jobs {
        if let Some(earlier) = earlier(&job) {
            stock.listed.push(Listed {
                length_s: earlier.extent.length_s,
                earlier: Some(earlier),
                job,
            });
            continue;
        }
        match Input::open(&job.video, ONE_THREAD) {
            Ok(input) => {
                let video = input.video();
                let length_s = video
                    .frame_rate()
                    .seconds(FrameIndex::new(video.announced_frames().get()));
                stock.listed.push(Listed {
                    job,
                    earlier: None,
                    length_s,
                });
            }
            Err(error) => stock.refused.push(Refused {
                path: job.video,
                reason: Reason::Unopened {
                    cause: failure::reading_cause(&error),
                    detail: failure::chain(&error),
                },
            }),
        }
    }
    stock
}
