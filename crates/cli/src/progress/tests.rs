use super::{Progress, Silent, Step};

#[test]
fn the_silent_reporter_accepts_every_step() {
    for step in Step::ALL {
        assert!(Silent.step(step).is_ok());
    }
}
