use std::io::{self, Write};

use bench::against::against;
use bench::guard::{Change, Guard};
use bench::metrics::Judging;
use engine::config::DetectConfig;
use media::decode::Threads;
use reference::fixture::fixture;
use reference::location::{Reference, recorded_bench};

#[cfg(test)]
fn listed(changes: &[Change]) -> String {
    let listed: Vec<String> = changes
        .iter()
        .map(|change| format!("{} ({})", change.id, change.start.get()))
        .collect();
    listed.join(", ")
}

#[cfg(test)]
fn write_guard(name: &str, guard: &Guard) {
    let mut out = io::stdout().lock();
    writeln!(out, "{name}: failed {}", guard.failed()).unwrap();
    for class in &guard.classes {
        let class_name = class.class;
        writeln!(
            out,
            "  {} {} σ {}: n {}, v0.1.1 {}, Rust {}, bound {:.3}, lost [{}], gained [{}]",
            class_name.motion().name(),
            class_name.amplitude(),
            class_name.sigma(),
            class.n,
            class.v011_found,
            class.found,
            class.lowest,
            listed(&class.lost),
            listed(&class.gained),
        )
        .unwrap();
    }
    writeln!(
        out,
        "  v0.1.1 only [{}], Rust only [{}], contrast differs [{}], false tracks {} / {}, reference tracks {} / {}",
        listed(&guard.v011_only),
        listed(&guard.rust_only),
        listed(&guard.contrast_differs),
        guard.v011_false_tracks,
        guard.false_tracks,
        guard.v011_reference_tracks,
        guard.reference_tracks
    )
    .unwrap();
}

#[test]
#[ignore = "local"]
fn the_two_benches_of_the_v011_replayed_on_the_092_fall_below_no_lower_wilson_bound() {
    let video = fixture("video_092_original.mp4").unwrap();
    for name in ["transit", "hunt"] {
        let checked = against(
            &video,
            Threads::Automatic,
            &DetectConfig::default(),
            Judging::default(),
            &recorded_bench(Reference::V011, name).unwrap(),
        )
        .unwrap();

        write_guard(name, &checked.guard);
        assert!(!checked.guard.failed(), "{name}");
    }
}
