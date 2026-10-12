use std::io;
use std::sync::OnceLock;

use compare::case::Target;
use compare::comparison::{VIDEOS_AT_ONCE, compare_all};
use compare::outcome::Outcome;
use compare::report::{write_refusals, write_report};
use compare::tracks::TrackCounts;

#[cfg(test)]
fn outcome() -> &'static Outcome {
    static OUTCOME: OnceLock<Outcome> = OnceLock::new();
    OUTCOME.get_or_init(|| {
        let cases = Target::Settings.cases().unwrap();
        let outcome = Outcome::of(&cases, compare_all(&cases, VIDEOS_AT_ONCE, None));
        let mut stdout = io::stdout().lock();
        write_report(&outcome.compared, &mut stdout).unwrap();
        write_refusals(&outcome.refused, &mut stdout).unwrap();
        outcome
    })
}

#[test]
#[ignore = "slow"]
fn each_setting_the_engine_cannot_handle_yet_is_refused_for_what_it_needs_and_none_fails() {
    let failed: Vec<String> = outcome()
        .failed
        .iter()
        .map(|(recording, error)| format!("{}: {error}", recording.display()))
        .collect();
    let refused: Vec<(String, &str)> = outcome()
        .refused
        .iter()
        .map(|refusal| {
            let name = refusal.recording.file_name().unwrap().to_string_lossy().into_owned();
            (name, refusal.reason.as_str())
        })
        .collect();
    let expected = [
        ("merge_radius_0", "merge_radius 0: only 6 is handled"),
        ("osd", "osd_regions: masks are not handled"),
        ("quick", "target_sigma 0: only 1.5"),
        ("sigma_0", "target_sigma 0: only 1.5"),
        ("sigma_3", "target_sigma 3: only 1.5"),
        ("size_720x540", "is 720×540, only 1440×1080 is handled"),
        ("step_1", "keeps 31 of 31 window frames, more than 16"),
        ("width_1440", "to 1440 pixels wide is not a reduction by 3:2"),
        ("width_1440_osd", "osd_regions: masks are not handled"),
        ("width_480", "to 480 pixels wide is not a reduction by 3:2"),
        ("width_720", "to 720 pixels wide is not a reduction by 3:2"),
        ("window_2s", "keeps 21 of 61 window frames, more than 16"),
    ];

    assert_eq!(failed, Vec::<String>::new());
    assert_eq!(
        refused.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>(),
        expected.map(|(name, _)| name)
    );
    for ((name, reason), (_, needs)) in refused.iter().zip(expected) {
        assert!(reason.contains(needs), "{name}: {reason}");
    }
}

#[test]
#[ignore = "slow"]
fn every_setting_the_engine_handles_agrees_with_the_v011_on_its_detections_and_on_every_track() {
    let found: Vec<(&str, bool, TrackCounts)> = outcome()
        .compared
        .iter()
        .map(|comparison| {
            (
                comparison.detections.name(),
                comparison.detections.holds(),
                comparison.tracks.counts(),
            )
        })
        .collect();
    let agreeing = |tracks: usize| TrackCounts {
        reference: tracks,
        paired: tracks,
        agreeing: tracks,
        lost: 0,
        added: 0,
    };

    assert_eq!(
        found,
        [
            ("max_area_400", true, agreeing(4)),
            ("min_area_9", true, agreeing(5)),
            ("noise_factor_0", true, agreeing(4)),
            ("threshold_25", true, agreeing(4)),
        ]
    );
}
