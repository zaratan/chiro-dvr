#[path = "support/agreement.rs"]
mod agreement;

use batdetect::process::process;
use media::decode::Threads;
use reference::location::{recorded_video, recording_directories};
use reference::recording::Recording;
use reference::track_matching::DiscrepancyKind;
use types::language::Language;
use types::rate::FrameRate;

#[test]
#[ignore = "local"]
fn the_twenty_six_videos_give_the_tracks_of_the_reference_but_for_the_known_divergences() {
    let directories = recording_directories().unwrap();
    assert_eq!(directories.len(), 26);
    let out = tempfile::tempdir().unwrap();
    let (mut reference, mut paired, mut agreeing, mut lost, mut added) = (0, 0, 0, 0, 0);
    let mut discrepancies = Vec::new();
    for directory in directories {
        let recording = Recording::read(&directory).unwrap();
        let name = recording.summary.video.name.clone();
        let rate = FrameRate::new(recording.summary.stream.avg_frame_rate).unwrap();
        let job = agreement::job(&recorded_video(&name).unwrap(), &out.path().join(&name));
        let processed = process(
            &job,
            Language::English,
            &agreement::DebugLabels,
            Threads::Automatic,
            agreement::VERSIONS,
        )
        .unwrap();
        let found = agreement::agreement(&processed, &recording, rate);
        println!(
            "{name}\treference {}\ttracked {}\tpaired {} (agreeing {})\tlost {}\tadded {}",
            found.reference,
            processed.tracks.len(),
            found.paired,
            found.agreeing,
            found.lost,
            found.added
        );
        reference += found.reference;
        paired += found.paired;
        agreeing += found.agreeing;
        lost += found.lost;
        added += found.added;
        discrepancies.extend(found.discrepancies);
    }
    println!("total\treference {reference}\tpaired {paired} (agreeing {agreeing})\tlost {lost}\tadded {added}");
    for discrepancy in &discrepancies {
        println!("{discrepancy:?}");
    }
    assert_eq!(reference, 966);
    assert_eq!((paired, agreeing, lost, added), (963, 958, 3, 9));
    let found: Vec<(&str, usize, DiscrepancyKind)> = discrepancies
        .iter()
        .map(|discrepancy| (discrepancy.video.as_str(), discrepancy.frame.get(), discrepancy.kind))
        .collect();
    assert_eq!(found, known_divergences());
}

#[cfg(test)]
fn known_divergences() -> Vec<(&'static str, usize, DiscrepancyKind)> {
    let points = |id, reference_points, candidate_points| DiscrepancyKind::Points {
        reference_id: id,
        candidate_id: id,
        reference_points,
        candidate_points,
    };
    let missed = |reference_id| DiscrepancyKind::Missed { reference_id };
    let extra = |candidate_id| DiscrepancyKind::Extra { candidate_id };
    vec![
        ("video_094.mp4", 3069, missed(3)),
        ("video_094.mp4", 3095, extra(3)),
        ("video_097.mp4", 612, missed(8)),
        ("video_097.mp4", 619, extra(8)),
        ("video_097.mp4", 4872, points(88, 86, 24)),
        ("video_097.mp4", 4906, extra(90)),
        ("video_098.mp4", 176, extra(4)),
        ("video_098.mp4", 1245, missed(20)),
        ("video_099.mp4", 429, points(14, 75, 119)),
        ("video_099.mp4", 509, points(16, 60, 16)),
        ("video_099.mp4", 538, points(17, 127, 50)),
        ("video_099.mp4", 583, extra(19)),
        ("video_099.mp4", 838, extra(30)),
        ("video_100.mp4", 2890, extra(34)),
        ("video_100.mp4", 2894, extra(36)),
        ("video_126.mp4", 2754, points(8, 32, 27)),
        ("video_126.mp4", 2776, extra(9)),
    ]
}
