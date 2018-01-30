use super::time;

pub fn get_time_now() -> f64 {
    let time = time::get_time();

    time.sec as f64 + (time.nsec as f64 / 1_000_000_000.)
}

pub fn seconds_to_hms (total_secs : i64) -> (usize, usize, usize) {
    use std::cmp::max;
    let total_secs = max(total_secs, 0);
    let hour   : usize = (total_secs / (60 * 60)) as usize;
    let minute : usize = ((total_secs - (hour as i64) * 60 * 60) / 60) as usize;
    let second : usize = (total_secs
                          - (hour   as i64) * 60 * 60
                          - (minute as i64) * 60) as usize;
    (hour, minute, second)
}
