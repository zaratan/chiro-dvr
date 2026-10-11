use std::io;
use std::path::Path;
use std::sync::Mutex;
use std::thread;

use engine::config::DetectConfig;
use engine::detection::detect;
use media::decode::Threads;
use reference::location::{recorded_video, recording_directories};
use reference::recording::Recording;

#[cfg(test)]
#[path = "support/agreement.rs"]
mod agreement;

use agreement::{Agreement, MIN_RATE};

const VIDEOS_AT_ONCE: usize = 6;

#[cfg(test)]
fn agreement_of(directory: &Path) -> Agreement {
    let recording = Recording::read(directory).unwrap();
    let path = recorded_video(&recording.summary.video.name).unwrap();
    let detected = detect(&path, Threads::Automatic, &DetectConfig::default()).unwrap();
    assert_eq!(detected.half_window().get(), recording.summary.background_half_window);
    Agreement::of(&detected, &recording)
}

#[test]
#[ignore = "local"]
fn none_of_the_twenty_six_videos_falls_under_98_percent_of_agreement_either_way() {
    let directories = Mutex::new(recording_directories().unwrap());
    assert_eq!(directories.lock().unwrap().len(), 26);
    let agreements = Mutex::new(Vec::new());
    thread::scope(|scope| {
        for _ in 0..VIDEOS_AT_ONCE {
            scope.spawn(|| {
                loop {
                    let next = directories.lock().unwrap().pop();
                    let Some(directory) = next else {
                        break;
                    };
                    let agreement = agreement_of(&directory);
                    agreements.lock().unwrap().push(agreement);
                }
            });
        }
    });
    let mut agreements = agreements.into_inner().unwrap();
    agreements.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out = io::stdout().lock();
    for agreement in &agreements {
        agreement.write_summary(&mut out).unwrap();
    }
    for agreement in &agreements {
        agreement.write_groups(&mut out).unwrap();
    }
    let below: Vec<&str> = agreements
        .iter()
        .filter(|agreement| agreement.reference_rate() < MIN_RATE || agreement.rust_rate() < MIN_RATE)
        .map(|agreement| agreement.name.as_str())
        .collect();
    assert_eq!(below, Vec::<&str>::new());
}
