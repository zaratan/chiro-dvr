use std::path::{Path, PathBuf};

use files::names::Part;
use files::params::SummaryEntry;

use super::summaries;
use crate::overview::Sheet;

fn sheet(page: &str) -> Sheet {
    Sheet {
        page: PathBuf::from(page),
        part: Some(Part { number: 2, count: 3 }),
        window: [600.0, 900.066_67],
        ignored: vec![[612.333, 640.0]],
        damaged: Vec::new(),
        other_starts: vec![601.234_5],
        radius: 23.0,
    }
}

#[test]
fn each_summary_keeps_its_window_its_part_and_its_periods_to_the_hundredth_under_its_name_in_the_folder() {
    assert_eq!(
        summaries(Path::new("out/v"), &[sheet("out/v/v_resume_010m-016m_2_sur_3.png")]),
        [SummaryEntry {
            file: "v_resume_010m-016m_2_sur_3.png".to_owned(),
            window_s: [600.0, 900.07],
            part: Some([2, 3]),
            ignored_s: vec![[612.33, 640.0]],
            damaged_s: Vec::new(),
            other_starts_s: vec![601.23],
            radius: 23.0,
        }]
    );
}

#[test]
fn a_summary_outside_the_folder_is_left_out() {
    assert_eq!(summaries(Path::new("out/v"), &[sheet("elsewhere/v_resume.png")]), []);
}
