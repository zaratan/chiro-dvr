use engine::config::DetectConfig;

#[path = "support/flash.rs"]
mod flash;

use flash::{assert_within_bounds, flash_cost};

#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

#[test]
#[ignore = "slow"]
fn a_full_frame_flash_seen_by_every_worker_costs_at_most_32_bytes_per_work_pixel_and_worker() {
    let cost = flash_cost(&DetectConfig::default());

    assert_within_bounds(&cost);
}
