use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::time::Duration;

use batdetect::jobs::{Job, Reason, Refused};
use batdetect::options::Options;
use batdetect::outcome::{Failure, Panic, Row, State};
use batdetect::process::{self, Extent};
use batdetect::reuse::Earlier;
use batdetect::tally::Tally;
use engine::config::DetectConfig;
use engine::detection;
use engine::input;
use engine::lost::Losses;
use engine::settings::DetectSettings;
use files::params::Finish;
use files::previous::Kept;
use tracking::stability::StabilityConfig;
use tracking::track::TrackConfig;
use types::language::Language;

use super::write;
use crate::texts::Texts;

fn row(dest: &str, seconds: u64, state: State) -> Row {
    let dest = PathBuf::from(dest);
    Row {
        job: Job {
            video: PathBuf::from(format!("{}.mp4", dest.display())),
            dest,
        },
        elapsed: Duration::from_secs(seconds),
        state,
    }
}

fn extent(length_s: f64, not_analysed_s: f64) -> Extent {
    Extent {
        length_s,
        not_analysed_s,
    }
}

fn done(tracks: usize, extent: Extent) -> State {
    State::Done {
        tracks,
        extent,
        kept: Vec::new(),
    }
}

fn damaged() -> State {
    State::Failed(Failure::Error(process::Error::Detection(detection::Error::Input(
        input::Error::FrameCount {
            path: PathBuf::from("nuit/batdetect/video_094.mp4"),
            announced: 30,
            losses: Losses::default(),
            decoded: 20,
        },
    ))))
}

fn evening() -> Tally {
    let kept = vec![
        Kept::SetAside {
            path: PathBuf::from("nuit/batdetect/video_092/video_092_pistes.csv"),
            dated: PathBuf::from("nuit/batdetect/video_092/video_092_pistes_2026-10-16_1430.csv"),
            reason: files::previous::Reason::Verdicts,
        },
        Kept::Left {
            path: PathBuf::from("nuit/batdetect/video_092/video_092_tracks.csv"),
            ours: false,
        },
    ];
    Tally {
        refused: Vec::new(),
        unwritten: Vec::new(),
        rows: vec![
            row("nuit/batdetect/video_089", 130, done(48, extent(962.4, 41.2))),
            row(
                "nuit/batdetect/video_092",
                17,
                State::Done {
                    tracks: 14,
                    extent: extent(301.0, 0.0),
                    kept,
                },
            ),
            row("nuit/batdetect/video_094", 0, damaged()),
            row(
                "nuit/batdetect/video_099",
                72,
                State::TooManyTracks {
                    count: 412,
                    limit: 300,
                    annotated: false,
                    extent: extent(600.0, 62.5),
                    kept: Vec::new(),
                },
            ),
            row("autre/batdetect/video_121", 100, done(1234, extent(598.0, 12.0))),
        ],
    }
}

fn written(tally: &Tally, options: &Options, language: Language) -> String {
    let mut out = Vec::new();
    write(&mut out, tally, options, Texts::new(language)).unwrap();
    String::from_utf8(out).unwrap()
}

fn tuned() -> Options {
    let settings = DetectSettings {
        threshold: 11.0,
        ..DetectSettings::default()
    };
    Options {
        detect: DetectConfig::try_from(&settings).unwrap(),
        ..Options::default()
    }
}

#[test]
fn the_french_summary_groups_the_videos_by_folder_then_lists_what_went_wrong_and_what_was_changed() {
    let expected = "\
Bilan\u{a0}: 5 vidéos, 3 réussies, 1 avec trop de pistes, 1 en échec

nuit/batdetect
  vidéo            durée      pistes   non analysé       calcul
  video_089   16 min 2 s          48          42 s   2 min 10 s
  video_092    5 min 1 s          14           0 s         17 s
  video_094            —       échec             —          0 s
  video_099   10 min 0 s   412, trop     1 min 3 s   1 min 12 s

autre/batdetect
  vidéo            durée   pistes   non analysé       calcul
  video_121   9 min 58 s    1\u{202f}234          12 s   1 min 40 s

Trop de pistes (tableau et image résumé écrits, pas d'extraits)\u{a0}:
  video_099\u{a0}: 412 pistes, au-delà de 300
    à faire\u{a0}: regarder l'image résumé\u{a0}; si ce sont de vrais passages, relancer avec un --max-tracks plus grand

1 vidéo sur 5 en échec\u{a0}:
  video_094\u{a0}: la vidéo est abîmée ou incomplète
    à faire\u{a0}: la recopier depuis les jumelles par le câble USB, sans passer par Stream Vision 2, puis relancer\u{a0}; si \
elle reste abîmée, le signaler

Tableau mis de côté\u{a0}:
  video_092\u{a0}: video_092_pistes.csv porte des verdicts, mis de côté sous video_092_pistes_2026-10-16_1430.csv

Tableau laissé tel quel\u{a0}:
  video_092\u{a0}: video_092_tracks.csv n'a pas la forme d'un tableau des pistes

Réglages changés\u{a0}: --threshold 11
";
    assert_eq!(written(&evening(), &tuned(), Language::French), expected);
}

#[test]
fn the_english_summary_has_the_same_shape() {
    let summary = written(&evening(), &tuned(), Language::English);
    let lines: Vec<&str> = summary.lines().collect();
    assert_eq!(lines[0], "Summary: 5 videos, 3 done, 1 with too many tracks, 1 failed");
    assert_eq!(
        lines[3],
        "  video           length          tracks   not analysed      compute"
    );
    assert!(
        lines.contains(&"  video_121   9 min 58 s     1234           12 s   1 min 40 s"),
        "{summary}"
    );
    assert!(lines.contains(&"1 video of 5 failed:"), "{summary}");
    assert!(lines.contains(&"Changed settings: --threshold 11"), "{summary}");
    assert!(lines.contains(&"Table left as it is:"), "{summary}");
    assert!(
        lines.contains(
            &"  video_092: video_092_pistes.csv holds verdicts, set aside as video_092_pistes_2026-10-16_1430.csv"
        ),
        "{summary}"
    );
}

#[test]
fn a_summary_of_videos_all_done_with_default_settings_has_only_its_title_and_its_table() {
    let tally = Tally {
        refused: Vec::new(),
        unwritten: Vec::new(),
        rows: vec![row("nuit/batdetect/video_092", 17, done(1, extent(301.0, 0.0)))],
    };
    assert_eq!(
        written(&tally, &Options::default(), Language::French),
        "Bilan\u{a0}: 1 vidéo, 1 réussie\n\nnuit/batdetect\n  vidéo           durée   pistes   non analysé   calcul\n  \
         video_092   5 min 1 s        1           0 s     17 s\n"
    );
}

#[test]
fn a_file_the_results_page_needs_but_could_not_write_is_named_at_the_end_with_its_cause() {
    let tally = Tally {
        refused: Vec::new(),
        unwritten: vec![files::error::Error::Write {
            path: PathBuf::from("nuit/batdetect/.batdetect/echecs/v.json"),
            source: std::io::Error::other("disk full"),
        }],
        rows: vec![row("nuit/batdetect/video_092", 17, done(1, extent(301.0, 0.0)))],
    };
    let summary = written(&tally, &Options::default(), Language::French);
    assert!(
        summary.ends_with(
            "\nFichier non écrit, la page de résultats peut être incomplète\u{a0}:\n  \
             nuit/batdetect/.batdetect/echecs/v.json\u{a0}: cannot write nuit/batdetect/.batdetect/echecs/v.json: disk full\n"
        ),
        "{summary}"
    );
    assert!(!tally.all_done());
}

#[test]
fn a_long_video_counts_hours_and_a_fraction_of_a_second_not_analysed_still_shows() {
    let tally = Tally {
        refused: Vec::new(),
        unwritten: Vec::new(),
        rows: vec![row("nuit/batdetect/longue", 4323, done(2, extent(7200.0, 0.3)))],
    };
    let summary = written(&tally, &Options::default(), Language::French);
    assert!(
        summary.contains("  longue   2 h 0 min 0 s        2           1 s   1 h 12 min 3 s"),
        "{summary}"
    );
}

#[test]
fn a_name_decomposed_by_the_file_system_is_composed_before_its_width_is_counted() {
    let tally = Tally {
        refused: Vec::new(),
        unwritten: Vec::new(),
        rows: vec![row(
            "nuit/batdetect/e\u{301}te\u{301}te\u{301}",
            1,
            done(1, extent(1.0, 0.0)),
        )],
    };
    let summary = written(&tally, &Options::default(), Language::French);
    let lines: Vec<&str> = summary.lines().collect();
    assert_eq!(lines[3], "  vidéo   durée   pistes   non analysé   calcul", "{summary}");
    assert!(lines[4].starts_with("  \u{e9}t\u{e9}t\u{e9}   "), "{summary}");
}

#[test]
fn several_failures_and_several_tables_left_as_they_are_are_counted_in_the_plural() {
    let panic = State::Failed(Failure::Panic(Panic {
        message: "boom".to_owned(),
        location: None,
    }));
    let kept = |name: &str| State::Done {
        tracks: 1,
        extent: extent(1.0, 0.0),
        kept: vec![Kept::Left {
            path: PathBuf::from(format!("nuit/batdetect/{name}/{name}_tracks.csv")),
            ours: false,
        }],
    };
    let tally = Tally {
        refused: Vec::new(),
        unwritten: Vec::new(),
        rows: vec![
            row("nuit/batdetect/a", 0, damaged()),
            row("nuit/batdetect/b", 0, panic),
            row("nuit/batdetect/c", 1, kept("c")),
            row("nuit/batdetect/d", 1, kept("d")),
        ],
    };
    let french = written(&tally, &Options::default(), Language::French);
    let lines: Vec<&str> = french.lines().collect();
    assert!(lines.contains(&"2 vidéos sur 4 en échec\u{a0}:"), "{french}");
    assert!(lines.contains(&"Tableaux laissés tels quels\u{a0}:"), "{french}");
    assert!(
        lines.contains(&"    à faire\u{a0}: le signaler avec le détail écrit pendant le traitement"),
        "{french}"
    );
    assert!(!french.contains("ci-dessous"), "{french}");
    let english = written(&tally, &Options::default(), Language::English);
    let lines: Vec<&str> = english.lines().collect();
    assert!(lines.contains(&"2 videos of 4 failed:"), "{english}");
    assert!(lines.contains(&"Tables left as they are:"), "{english}");
    assert!(!english.contains("detail below"), "{english}");
}

#[test]
fn the_changed_settings_are_written_as_they_are_typed() {
    let settings = DetectSettings {
        threshold: 6.5,
        work_width: 480,
        osd_regions: vec!["0,0,1,0.05".parse().unwrap(), "0,0.9,1,1".parse().unwrap()],
        ..DetectSettings::default()
    };
    let track = TrackConfig::default();
    let options = Options {
        detect: DetectConfig::try_from(&settings).unwrap(),
        track: TrackConfig::new(
            track.max_jump(),
            track.max_gap(),
            NonZeroUsize::new(3).unwrap(),
            track.min_travel(),
            track.twin_distance(),
            track.max_median_turn(),
        )
        .unwrap(),
        stability: StabilityConfig::new(None, StabilityConfig::default().pad_s()).unwrap(),
        ..Options::default()
    };
    let tally = Tally {
        refused: Vec::new(),
        unwritten: Vec::new(),
        rows: vec![row("nuit/batdetect/v", 1, done(1, extent(1.0, 0.0)))],
    };
    let summary = written(&tally, &options, Language::French);
    assert!(
        summary.ends_with(
            "Réglages changés\u{a0}: --threshold 6.5, --min-hits 3, --max-blobs 0, --work-width 480, --osd-region (2 zones)\n"
        ),
        "{summary}"
    );
}

fn with_refused(refused: Vec<Refused>) -> Tally {
    Tally {
        refused,
        unwritten: Vec::new(),
        rows: vec![row("nuit/batdetect/video_092", 17, done(1, extent(301.0, 0.0)))],
    }
}

fn missing(path: &str) -> Refused {
    Refused {
        path: PathBuf::from(path),
        reason: Reason::Missing,
    }
}

#[test]
fn the_inputs_refused_before_the_valid_videos_ran_are_counted_and_listed_with_what_to_do_under_their_own_title() {
    let two = with_refused(vec![
        missing("absente.mp4"),
        Refused {
            path: PathBuf::from("README.md"),
            reason: Reason::NotAVideo,
        },
    ]);
    assert_eq!(
        written(&two, &Options::default(), Language::French),
        "Bilan\u{a0}: 1 vidéo, 1 réussie, 2 entrées refusées\n\nnuit/batdetect\n  vidéo           durée   pistes   non \
         analysé   calcul\n  video_092   5 min 1 s        1           0 s     17 s\n\nEntrées refusées, non \
         traitées\u{a0}:\n  absente.mp4\u{a0}: introuvable\n    à faire\u{a0}: vérifier le chemin\u{a0}; le dossier ou la \
         vidéo a peut-être été déplacé ou renommé\n  README.md\u{a0}: pas une vidéo (mp4, mov, avi, mkv ou m4v)\n    à \
         faire\u{a0}: ne donner que des vidéos, ou des dossiers de vidéos\n"
    );
    let english = written(&two, &Options::default(), Language::English);
    let lines: Vec<&str> = english.lines().collect();
    assert_eq!(lines[0], "Summary: 1 video, 1 done, 2 inputs refused", "{english}");
    assert_eq!(
        lines[lines.len() - 5..],
        [
            "Refused inputs, not processed:",
            "  absente.mp4: not found",
            "    what to do: check the path; the folder or the video may have been moved or renamed",
            "  README.md: not a video (mp4, mov, avi, mkv or m4v)",
            "    what to do: give only videos, or folders of videos"
        ]
    );
    assert!(!two.all_done());
}

#[test]
fn a_single_refused_input_is_counted_and_titled_in_the_singular() {
    let one = with_refused(vec![missing("absente.mp4")]);
    for (language, title, heading) in [
        (
            Language::French,
            "Bilan\u{a0}: 1 vidéo, 1 réussie, 1 entrée refusée",
            "Entrée refusée, non traitée\u{a0}:",
        ),
        (
            Language::English,
            "Summary: 1 video, 1 done, 1 input refused",
            "Refused input, not processed:",
        ),
    ] {
        let summary = written(&one, &Options::default(), language);
        let lines: Vec<&str> = summary.lines().collect();
        assert_eq!(lines[0], title, "{summary}");
        assert!(lines.contains(&heading), "{summary}");
    }
}

fn skipped(finish: Finish, tracks: usize, extent: Extent) -> State {
    State::Skipped(Earlier {
        finish,
        tracks,
        limit: 300,
        extent,
    })
}

#[test]
fn a_resumed_evening_counts_the_videos_already_done_and_shows_their_earlier_figures_without_a_compute_time() {
    let tally = Tally {
        unwritten: Vec::new(),
        refused: Vec::new(),
        rows: vec![
            row(
                "nuit/batdetect/video_089",
                0,
                skipped(Finish::Done, 48, extent(962.4, 41.2)),
            ),
            row(
                "nuit/batdetect/video_099",
                0,
                skipped(Finish::TooManyTracks, 412, extent(600.0, 62.5)),
            ),
            row("nuit/batdetect/video_092", 17, done(14, extent(301.0, 0.0))),
        ],
    };
    let summary = written(&tally, &Options::default(), Language::French);
    let lines: Vec<&str> = summary.lines().collect();
    assert_eq!(
        lines[0],
        "Bilan\u{a0}: 3 vidéos, 1 réussie, 1 déjà faite, 1 avec trop de pistes"
    );
    assert_eq!(
        &lines[4..7],
        [
            "  video_089   16 min 2 s          48          42 s        —",
            "  video_099   10 min 0 s   412, trop     1 min 3 s        —",
            "  video_092    5 min 1 s          14           0 s     17 s",
        ]
    );
    assert!(
        lines.contains(&"  video_099\u{a0}: 412 pistes, au-delà de 300"),
        "{summary}"
    );
    let english = written(&tally, &Options::default(), Language::English);
    assert!(
        english.starts_with("Summary: 3 videos, 1 done, 1 already done, 1 with too many tracks\n"),
        "{english}"
    );
}

#[test]
fn a_page_of_someone_else_in_the_way_is_explained_with_what_to_do() {
    let tally = Tally {
        refused: Vec::new(),
        unwritten: vec![files::error::Error::Taken {
            path: PathBuf::from("Bureau/resultats.html"),
        }],
        rows: vec![row("Bureau/video_092", 17, done(1, extent(301.0, 0.0)))],
    };
    let summary = written(&tally, &Options::default(), Language::French);
    assert!(
        summary.ends_with(
            "  Bureau/resultats.html existe déjà et n'est pas une page de batdetect\u{a0}: \
             la renommer ou la déplacer pour avoir la page\n"
        ),
        "{summary}"
    );
}
