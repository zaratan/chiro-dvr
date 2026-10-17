use std::path::PathBuf;

use batdetect::access::Access;
use batdetect::checkup::{Checkup, Encoding, Space};
use media::encode::Codec;
use types::language::Language;

use super::report_of;
use crate::texts::Texts;

fn ready(encoding: Encoding) -> Checkup {
    Checkup {
        folder: PathBuf::from("/Users/manon/Soirées"),
        encoding,
        space: Space::Enough(412_300_000_000),
        refused_writing: None,
    }
}

fn french(found: &Checkup) -> String {
    report_of(found, "0.2.1", "7.1.1", Texts::new(Language::French))
}

#[test]
fn a_mac_ready_for_an_evening_says_so_line_by_line() {
    assert_eq!(
        french(&ready(Encoding::Preferred(Codec::VideoToolbox))),
        "batdetect 0.2.1 — FFmpeg 7.1.1 inclus\n\
         ✓ encodeur\u{a0}: VideoToolbox (matériel)\n\
         ✓ place libre\u{a0}: 412\u{a0}Go dans /Users/manon/Soirées\n\
         ✓ écriture\u{a0}: permise dans /Users/manon/Soirées\n\
         Tout est prêt."
    );
}

#[test]
fn openh264_alone_where_it_is_the_only_encoder_is_ready() {
    let report = french(&ready(Encoding::Preferred(Codec::OpenH264)));
    assert!(report.contains("✓ encodeur\u{a0}: openh264 (logiciel)\n"), "{report}");
    assert!(report.ends_with("Tout est prêt."), "{report}");
}

#[test]
fn videotoolbox_that_does_not_respond_is_a_warning_with_something_to_do_and_everything_stays_ready() {
    let report = french(&ready(Encoding::Fallback));
    assert!(
        report.contains(
            "⚠ encodeur\u{a0}: openh264 seulement, VideoToolbox ne répond pas\n  à faire\u{a0}: rien d'obligatoire"
        ),
        "{report}"
    );
    assert!(report.ends_with("Tout est prêt."), "{report}");
}

#[test]
fn every_problem_has_its_line_and_its_thing_to_do_and_they_are_counted() {
    let report = french(&Checkup {
        folder: PathBuf::from("/Volumes/CLE"),
        encoding: Encoding::Broken,
        space: Space::Short(3_240_000_000),
        refused_writing: Some(Access::ReadOnly),
    });
    assert_eq!(
        report,
        "batdetect 0.2.1 — FFmpeg 7.1.1 inclus\n\
         ✗ encodeur\u{a0}: aucun ne fonctionne\n\
         \u{20}\u{20}à faire\u{a0}: réinstaller batdetect\n\
         ✗ place libre\u{a0}: 3,2\u{a0}Go dans /Volumes/CLE, moins de 20\u{a0}Go\n\
         \u{20}\u{20}à faire\u{a0}: libérer de la place, ou choisir un autre dossier avec -o\n\
         ✗ écriture\u{a0}: refusée dans /Volumes/CLE (lecture seule)\n\
         \u{20}\u{20}à faire\u{a0}: choisir un dossier où l'écriture est permise, avec -o\n\
         3 points à corriger."
    );
}

#[test]
fn a_space_just_under_the_threshold_is_rounded_down_so_it_never_reads_as_the_threshold() {
    let report = french(&Checkup {
        space: Space::Short(19_960_000_000),
        ..ready(Encoding::Preferred(Codec::OpenH264))
    });
    assert!(
        report.contains("19,9\u{a0}Go dans /Users/manon/Soirées, moins de 20\u{a0}Go"),
        "{report}"
    );
}

#[test]
fn an_encoder_that_could_not_be_tried_points_to_the_system_disk_not_to_a_reinstall() {
    let report = french(&ready(Encoding::Untried));
    assert!(
        report.contains("✗ encodeur\u{a0}: non essayé, le dossier temporaire du système refuse l'écriture\n  à faire\u{a0}: libérer de la place sur le disque du système"),
        "{report}"
    );
    assert!(report.ends_with("1 point à corriger."), "{report}");
}

#[test]
fn a_free_space_that_cannot_be_measured_is_a_warning_without_anything_to_do() {
    let report = french(&Checkup {
        space: Space::Unknown,
        ..ready(Encoding::Preferred(Codec::OpenH264))
    });
    assert!(
        report.contains("⚠ place libre\u{a0}: non mesurée dans /Users/manon/Soirées\n✓"),
        "{report}"
    );
}

#[test]
fn the_english_report_counts_one_problem_in_the_singular() {
    let report = report_of(
        &Checkup {
            refused_writing: Some(Access::Refused),
            ..ready(Encoding::Preferred(Codec::OpenH264))
        },
        "0.2.1",
        "7.1.1",
        Texts::new(Language::English),
    );
    assert!(
        report.contains("✗ writing: refused in /Users/manon/Soirées (not allowed)\n"),
        "{report}"
    );
    assert!(report.ends_with("1 point to fix."), "{report}");
}
