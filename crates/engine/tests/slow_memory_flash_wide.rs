use engine::config::DetectConfig;
use engine::settings::DetectSettings;

#[path = "support/flash.rs"]
mod flash;

use flash::{assert_within_bounds, flash_cost};

#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

#[test]
#[ignore = "slow"]
fn working_at_the_full_1440_pixels_a_flash_still_costs_at_most_32_bytes_per_work_pixel_and_worker() {
    let config = DetectConfig::try_from(&DetectSettings {
        work_width: 1440,
        ..DetectSettings::default()
    })
    .unwrap();

    let cost = flash_cost(&config);

    assert_within_bounds(&cost);
}
