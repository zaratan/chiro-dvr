#[path = "support/frozen.rs"]
mod frozen;

use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Command;

use media::fingerprint;
use png::Decoder;
use reference::fixture::fixture;
use reference::location::{recorded_video, reference_video};
use render::clips::{Clip, Outputs};
use tempfile::TempDir;

use crate::frozen::{Frozen, clip, tracked};

const BINARY: &str = env!("CARGO_BIN_EXE_batdetect");
const FAST_PASS_OF_3_48_S: f64 = 228.27;
const SLOW_TRACK_OF_3_29_S: f64 = 209.59;
const ALONG_THE_EDGE_OF_0_28_S: f64 = 28.40;
const ZOOMED_FOUR_TIMES_OF_0_47_S: f64 = 47.12;
const SHORT_TRACK_OF_1_54_S: f64 = 114.62;
const INTERPOLATED_BOX_OF_1_48_S: f64 = 108.52;
const FAST_PASS_CLIP: Frozen = Frozen {
    frames: 108,
    fingerprint: 17_728_931_892_283_817_342,
};
const SLOW_TRACK_CLIP: Frozen = Frozen {
    frames: 380,
    fingerprint: 8_055_564_025_640_409_189,
};
const ALONG_THE_EDGE_ZOOM: Frozen = Frozen {
    frames: 128,
    fingerprint: 17_836_762_381_287_940_579,
};
const ZOOMED_FOUR_TIMES_ZOOM: Frozen = Frozen {
    frames: 110,
    fingerprint: 17_777_991_179_345_001_169,
};
const SHORT_TRACK_CLIP: Frozen = Frozen {
    frames: 96,
    fingerprint: 912_426_859_394_260_697,
};
const INTERPOLATED_BOX_CLIP: Frozen = Frozen {
    frames: 270,
    fingerprint: 6_458_377_840_674_619_821,
};
const CROWDED_097: [(&str, u64); 8] = [
    ("video_097_resume_1_sur_8.png", 3_016_591_617_919_252_239),
    ("video_097_resume_2_sur_8.png", 12_816_361_956_728_239_556),
    ("video_097_resume_3_sur_8.png", 11_276_992_867_599_041_228),
    ("video_097_resume_4_sur_8.png", 6_923_572_826_997_853_438),
    ("video_097_resume_5_sur_8.png", 15_749_111_215_172_610_700),
    ("video_097_resume_6_sur_8.png", 4_962_891_853_068_014_737),
    ("video_097_resume_7_sur_8.png", 1_996_271_331_930_406_870),
    ("video_097_resume_8_sur_8.png", 13_978_686_305_726_527_557),
];
const UNANALYSED_PERIODS_089_091: [(&str, u64); 4] = [
    ("video_089-091_resume_000m-010m_1_sur_2.png", 13_343_444_481_319_136_414),
    ("video_089-091_resume_000m-010m_2_sur_2.png", 17_283_076_105_502_241_980),
    ("video_089-091_resume_010m-016m_1_sur_2.png", 18_072_565_057_517_773_411),
    ("video_089-091_resume_010m-016m_2_sur_2.png", 10_750_407_845_215_895_006),
];

fn zoom(wanted: Vec<Clip>) -> Outputs {
    Outputs {
        clips: Vec::new(),
        zooms: wanted,
        annotated: None,
    }
}

#[cfg(test)]
fn summary_pixels_in_french(video: &Path) -> Vec<(String, u64)> {
    let out = TempDir::new().unwrap();
    let output = Command::new(BINARY)
        .arg(video)
        .arg("-o")
        .arg(out.path())
        .env_remove("LANGUAGE")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env("LANG", "fr_FR.UTF-8")
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let folder = out.path().join(video.file_stem().unwrap());
    let mut images: Vec<PathBuf> = fs::read_dir(&folder)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "png"))
        .collect();
    images.sort();
    images
        .iter()
        .map(|path| {
            let mut reader = Decoder::new(BufReader::new(File::open(path).unwrap()))
                .read_info()
                .unwrap();
            let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
            let info = reader.next_frame(&mut pixels).unwrap();
            pixels.truncate(info.buffer_size());
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            (name, fingerprint::of_pixels(info.width, info.height, &pixels))
        })
        .collect()
}

#[test]
#[ignore = "local"]
fn the_clips_of_the_092_judged_as_good_as_v0_1_1_draw_the_same_frames() {
    let drawing = tracked(&fixture("video_092_original.mp4").unwrap());

    let found = [
        drawing.frozen(FAST_PASS_OF_3_48_S, clip),
        drawing.frozen(SLOW_TRACK_OF_3_29_S, clip),
    ];

    assert_eq!(found, [FAST_PASS_CLIP, SLOW_TRACK_CLIP]);
}

#[test]
#[ignore = "local"]
fn the_zooms_of_the_125_judged_as_good_as_v0_1_1_draw_the_same_frames() {
    let drawing = tracked(&recorded_video("video_125.mp4").unwrap());

    let found = [
        drawing.frozen(ALONG_THE_EDGE_OF_0_28_S, zoom),
        drawing.frozen(ZOOMED_FOUR_TIMES_OF_0_47_S, zoom),
    ];

    assert_eq!(found, [ALONG_THE_EDGE_ZOOM, ZOOMED_FOUR_TIMES_ZOOM]);
}

#[test]
#[ignore = "local"]
fn the_clips_of_the_097_judged_as_good_as_v0_1_1_draw_the_same_frames() {
    let drawing = tracked(&recorded_video("video_097.mp4").unwrap());

    let found = [
        drawing.frozen(SHORT_TRACK_OF_1_54_S, clip),
        drawing.frozen(INTERPOLATED_BOX_OF_1_48_S, clip),
    ];

    assert_eq!(found, [SHORT_TRACK_CLIP, INTERPOLATED_BOX_CLIP]);
}

#[test]
#[ignore = "local"]
fn the_summary_images_judged_as_good_as_v0_1_1_keep_their_pixels() {
    let crowded = summary_pixels_in_french(&recorded_video("video_097.mp4").unwrap());
    let with_unanalysed_periods = summary_pixels_in_french(&reference_video("video_089-091.mp4").unwrap());

    let named = |frozen: &[(&str, u64)]| -> Vec<(String, u64)> {
        frozen
            .iter()
            .map(|(name, pixels)| ((*name).to_owned(), *pixels))
            .collect()
    };
    assert_eq!(
        (crowded, with_unanalysed_periods),
        (named(&CROWDED_097), named(&UNANALYSED_PERIODS_089_091))
    );
}
