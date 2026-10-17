use std::fs;
use std::path::Path;

use files::params::Finish;

use super::{Listed, Stock, take_stock};
use crate::failure::Cause;
use crate::flight::write_flight;
use crate::jobs::{Job, Reason, Refused};
use crate::process::Extent;
use crate::reuse::Earlier;
use crate::space::{GIGABYTE, Need, Volume, launch_estimate};

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

#[test]
fn the_needs_leave_out_the_videos_already_done_and_count_the_others_in_their_results_folder() {
    let folder = tempfile::tempdir().unwrap();
    let (todo, done) = (job(folder.path(), "a.mp4"), job(folder.path(), "b.mp4"));
    write_flight(&todo.video);
    fs::write(&done.video, "not opened").unwrap();
    let stock = take_stock(vec![todo.clone(), done.clone()], |job| {
        (job.video == done.video).then(|| earlier(60.0))
    });
    let size = fs::metadata(&todo.video).unwrap().len();
    assert_eq!(
        stock.needs(true),
        [Need {
            folder: folder.path().join("out"),
            bytes: launch_estimate(size, true),
        }]
    );
}

fn readable(folder: &Path, names: &[&str]) -> Stock {
    let jobs: Vec<Job> = names.iter().map(|name| job(folder, name)).collect();
    for listed in &jobs {
        write_flight(&listed.video);
    }
    take_stock(jobs, |_| None)
}

fn missing() -> Refused {
    Refused {
        path: "absente.mp4".into(),
        reason: Reason::Missing,
    }
}

#[test]
fn a_disk_too_small_for_the_results_is_refused_after_the_inputs_naming_its_results_folder() {
    let folder = tempfile::tempdir().unwrap();
    let mut stock = readable(folder.path(), &["a.mp4"]);
    stock.refused.push(missing());
    stock.refuse_short_disks(true, |_| Some(Volume { device: 1, free: 0 }));
    assert_eq!(stock.refused.len(), 2);
    assert_eq!(stock.refused[0], missing());
    assert_eq!(stock.refused[1].path, folder.path().join("out"));
    assert!(matches!(
        stock.refused[1].reason,
        Reason::DiskShort {
            free: 0,
            annotated: true,
            ..
        }
    ));
}

#[test]
fn a_disk_with_room_or_whose_room_cannot_be_read_refuses_nothing() {
    let folder = tempfile::tempdir().unwrap();
    let mut stock = readable(folder.path(), &["a.mp4"]);
    stock.refuse_short_disks(false, |_| {
        Some(Volume {
            device: 1,
            free: 2 * GIGABYTE,
        })
    });
    stock.refuse_short_disks(false, |_| None);
    assert_eq!(stock.refused, []);
}

#[test]
fn going_ahead_despite_the_disks_drops_their_refusals_but_keeps_those_of_the_inputs() {
    let folder = tempfile::tempdir().unwrap();
    let mut stock = readable(folder.path(), &["a.mp4"]);
    stock.refused.push(missing());
    stock.refuse_short_disks(false, |_| Some(Volume { device: 1, free: 0 }));
    stock.go_ahead_despite_the_disks();
    assert_eq!(stock.refused, [missing()]);
}
