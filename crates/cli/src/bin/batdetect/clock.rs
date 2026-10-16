#[cfg(test)]
mod tests;

const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 3600;

pub(crate) fn clock(seconds: u64) -> String {
    let (hours, rest) = (seconds / SECONDS_PER_HOUR, seconds % SECONDS_PER_HOUR);
    let (minutes, seconds) = (rest / SECONDS_PER_MINUTE, rest % SECONDS_PER_MINUTE);
    match (hours, minutes) {
        (0, 0) => format!("{seconds} s"),
        (0, _) => format!("{minutes} min {seconds} s"),
        _ => format!("{hours} h {minutes} min {seconds} s"),
    }
}
