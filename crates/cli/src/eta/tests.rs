use super::Evening;

#[test]
fn nothing_is_estimated_before_a_first_video_is_computed() {
    let evening = Evening::new(vec![900, 900, 900]);
    assert_eq!(evening.left_during(1, 5.0), None);
    assert_eq!(evening.left_after(1), None);
}

#[test]
fn after_the_first_video_the_frames_left_take_the_seconds_per_frame_it_took() {
    let mut evening = Evening::new(vec![900, 1800, 900]);
    evening.computed(1, 30.0);
    assert_eq!(evening.left_after(1), Some(90.0));
}

#[test]
fn the_pace_is_the_mean_over_every_video_computed_since_the_start_weighted_by_frames() {
    let mut evening = Evening::new(vec![1000, 3000, 1000]);
    evening.computed(1, 10.0);
    evening.computed(2, 70.0);
    assert_eq!(evening.left_after(2), Some(20.0));
}

#[test]
fn during_a_video_the_time_it_has_already_taken_is_taken_off_its_own_share() {
    let mut evening = Evening::new(vec![1000, 1000, 1000]);
    evening.computed(1, 30.0);
    assert_eq!(evening.left_during(2, 12.0), Some(48.0));
}

#[test]
fn a_video_running_over_its_share_leaves_only_the_videos_after_it() {
    let mut evening = Evening::new(vec![1000, 1000, 1000]);
    evening.computed(1, 30.0);
    assert_eq!(evening.left_during(2, 45.0), Some(30.0));
}

#[test]
fn videos_the_resume_skips_have_no_frames_to_compute_and_add_no_time() {
    let mut evening = Evening::new(vec![1000, 0, 0, 1000]);
    evening.computed(1, 20.0);
    assert_eq!(evening.left_after(1), Some(20.0));
}

#[test]
fn a_video_that_failed_is_not_reported_so_it_does_not_drag_the_pace() {
    let mut evening = Evening::new(vec![1000, 1000, 1000]);
    evening.computed(2, 40.0);
    assert_eq!(evening.left_after(2), Some(40.0));
}

#[test]
fn once_nothing_is_left_to_compute_there_is_no_finish_to_announce() {
    let mut evening = Evening::new(vec![1000, 1000, 0]);
    evening.computed(1, 20.0);
    evening.computed(2, 20.0);
    assert_eq!(evening.left_after(2), None);
    assert_eq!(evening.left_during(3, 0.0), None);
}

#[test]
fn the_last_video_still_has_its_own_share_while_it_runs() {
    let mut evening = Evening::new(vec![1000, 1000]);
    evening.computed(1, 20.0);
    assert_eq!(evening.left_during(2, 5.0), Some(15.0));
}

#[test]
fn a_position_outside_the_evening_or_a_spent_time_that_is_not_a_duration_is_ignored() {
    let mut evening = Evening::new(vec![1000, 1000]);
    evening.computed(0, 20.0);
    evening.computed(3, 20.0);
    evening.computed(1, f64::NAN);
    evening.computed(1, -1.0);
    assert_eq!(evening.left_after(1), None);
    assert_eq!(evening.left_during(0, 0.0), None);
}
