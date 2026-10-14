use std::env;
use std::panic;
use std::path::PathBuf;
use std::process::Command;
use std::thread;

use batdetect::jobs::Job;
use batdetect::outcome::{Failure, quiet_panics, run_all};

const CHILD: &str = "BATDETECT_PANIC_HOOK_CHILD";
const TEST: &str = "a_panic_in_a_video_is_kept_with_its_place_and_written_nowhere_while_one_outside_stays_visible";

#[test]
fn a_panic_in_a_video_is_kept_with_its_place_and_written_nowhere_while_one_outside_stays_visible() {
    if env::var_os(CHILD).is_some() {
        quiet_panics();
        let jobs = [PathBuf::from("v.mp4"), PathBuf::from("w.mp4")].map(|video| Job {
            dest: PathBuf::from("out").join(&video),
            video,
        });
        let outcomes = run_all(
            &jobs,
            |_, rank| {
                thread::scope(|scope| {
                    if rank.position == 2 {
                        scope.spawn(|| panic!("in a thread {}", 1));
                    }
                });
                panic!("inside a video")
            },
            |_, _, _| Ok(()),
        )
        .unwrap();
        for outcome in &outcomes {
            if let Err(Failure::Panic(panic)) = outcome {
                println!(
                    "kept: {} at {}",
                    panic.message,
                    panic.location.clone().unwrap_or_default()
                );
            }
        }
        assert!(panic::catch_unwind(|| panic!("outside any video")).is_err());
        return;
    }
    let output = Command::new(env::current_exe().unwrap())
        .args([TEST, "--exact", "--nocapture", "--test-threads=1"])
        .env(CHILD, "1")
        .output()
        .unwrap();
    let (out, errors) = (
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(output.status.success(), "{out}\n{errors}");
    assert!(
        out.lines()
            .any(|line| line.contains(&format!("kept: inside a video at {}:", file!()))),
        "{out}"
    );
    assert!(
        out.lines()
            .any(|line| line.contains(&format!("kept: in a thread 1 at {}:", file!()))),
        "{out}"
    );
    assert!(!errors.contains("inside a video"), "{errors}");
    assert!(!errors.contains("in a thread"), "{errors}");
    assert!(errors.contains("outside any video"), "{errors}");
}
