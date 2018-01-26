use super::time;

pub fn get_time_now() -> f64 {
    let time = time::get_time();

    time.sec as f64 + (time.nsec as f64 / 1_000_000_000.)
}
