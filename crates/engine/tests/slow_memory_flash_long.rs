use engine::config::DetectConfig;
use engine::settings::DetectSettings;

#[path = "support/flash.rs"]
mod flash;

use flash::{assert_within_bounds, flash_cost};

#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

const SAMPLES: usize = 61;
const BYTES_PER_UNIT: usize = 2;
const FLASHES_OF_THE_LAST_FLUSH: [usize; 4] = [35, 43, 51, 59];

#[test]
#[ignore = "slow"]
fn with_sixty_one_background_samples_a_flash_still_costs_at_most_32_bytes_per_work_pixel_and_worker() {
    let config = DetectConfig::try_from(&DetectSettings {
        bg_window_s: 2.0,
        bg_step: 1,
        ..DetectSettings::default()
    })
    .unwrap();

    let cost = flash_cost(&config, &FLASHES_OF_THE_LAST_FLUSH);

    assert_within_bounds(&cost);
    assert!(
        cost.calm_bytes > SAMPLES * BYTES_PER_UNIT * cost.work_pixels,
        "{}",
        cost.calm_bytes
    );
}
