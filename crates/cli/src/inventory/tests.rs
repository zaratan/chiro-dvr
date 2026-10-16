use std::fs;
use std::path::Path;

use files::params::Finish;

use super::{Listed, take_stock};
use crate::failure::Cause;
use crate::flight::write_flight;
use crate::jobs::{Job, Reason};
use crate::process::Extent;
use crate::reuse::Earlier;

fn job(folder: &Path, name: &str) -> Job {
    Job {
        video: folder.join(name),
        dest: folder.join("out").join(name),
    }
}

fn earlier(length_s: f64) -> Earlier {
    Earlier {
        finish: Finish::Done,
        tracks: 2,
        limit: 300,
        extent: Extent {
            length_s,
            not_analysed_s: 0.0,
        },
    }
}

#[test]
fn a_readable_video_is_listed_with_the_length_its_announced_frames_give() {
    let folder = tempfile::tempdir().unwrap();
    let flight = job(folder.path(), "flight.mp4");
    write_flight(&flight.video);
    let stock = take_stock(vec![flight.clone()], |_| None);
    assert!(stock.refused.is_empty(), "{:?}", stock.refused);
    assert_eq!(
        stock.listed,
        [Listed {
            job: flight,
            earlier: None,
            length_s: 2.0,
        }]
    );
}

#[test]
fn a_file_named_like_a_video_that_cannot_be_opened_is_refused_with_the_cause_its_failure_would_give() {
    let folder = tempfile::tempdir().unwrap();
    let fake = job(folder.path(), "faux.mp4");
    fs::write(&fake.video, "not a video").unwrap();
    let stock = take_stock(vec![fake.clone()], |_| None);
    assert_eq!(stock.listed, []);
    assert_eq!(stock.refused.len(), 1);
    assert_eq!(stock.refused[0].path, fake.video);
    let Reason::Unopened { cause, detail } = &stock.refused[0].reason else {
        panic!("{:?}", stock.refused[0].reason);
    };
    assert_eq!(*cause, Cause::NotAVideo);
    assert!(detail.starts_with("cannot decode "), "{detail}");
}

#[test]
fn a_video_already_done_is_not_opened_and_counts_the_length_of_its_earlier_run() {
    let folder = tempfile::tempdir().unwrap();
    let done = job(folder.path(), "faite.mp4");
    fs::write(&done.video, "not opened").unwrap();
    let stock = take_stock(vec![done.clone()], |_| Some(earlier(41.5)));
    assert!(stock.refused.is_empty(), "{:?}", stock.refused);
    assert_eq!(
        stock.listed,
        [Listed {
            job: done,
            earlier: Some(earlier(41.5)),
            length_s: 41.5,
        }]
    );
}

#[test]
fn the_stock_keeps_the_order_given_and_counts_the_length_and_the_videos_already_done() {
    let folder = tempfile::tempdir().unwrap();
    let (first, done, fake, last) = (
        job(folder.path(), "a.mp4"),
        job(folder.path(), "b.mp4"),
        job(folder.path(), "c.mp4"),
        job(folder.path(), "d.mp4"),
    );
    write_flight(&first.video);
    write_flight(&last.video);
    fs::write(&fake.video, "not a video").unwrap();
    let stock = take_stock(vec![first.clone(), done.clone(), fake, last.clone()], |job| {
        (job.video == done.video).then(|| earlier(60.0))
    });
    assert_eq!(stock.jobs(), [first.clone(), done.clone(), last]);
    assert!((stock.length_s() - 64.0).abs() < 1e-9, "{}", stock.length_s());
    assert_eq!(stock.already_done(), 1);
    assert_eq!(stock.earlier(&done), Some(earlier(60.0)));
    assert_eq!(stock.earlier(&first), None);
    assert_eq!(stock.refused.len(), 1);
}
