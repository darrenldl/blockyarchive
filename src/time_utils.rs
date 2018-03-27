use chrono::prelude::*;

pub enum TimeMode {
    UTC,
    Local
}

pub fn get_time_now(mode : TimeMode) -> f64 {
    let (sec, nsec) = match mode {
        TimeMode::UTC   => {
            let time = Utc::now();
            (time.timestamp(), time.timestamp_subsec_nanos())
        },
        TimeMode::Local => {
            let time = Local::now();
            (time.timestamp(), time.timestamp_subsec_nanos())
        }
    };

    sec as f64 + (nsec as f64 / 1_000_000_000.)
}

pub fn seconds_to_hms(total_secs : i64) -> (isize, isize, isize) {
    let hour   : isize = (total_secs / (60 * 60)) as isize;
    let minute : isize = ((total_secs - (hour as i64) * 60 * 60) / 60) as isize;
    let second : isize = (total_secs
                          - (hour   as i64) * 60 * 60
                          - (minute as i64) * 60) as isize;

    (hour, minute, second)
}

pub fn i64_secs_to_date_time_string(secs : i64,
                                    mode : TimeMode) -> Option<String> {
    let datetime =
        match NaiveDateTime::from_timestamp_opt(secs, 0) {
            None    => None,
            Some(x) => match mode {
                TimeMode::UTC   => Some((x.year(),
                                         x.month(),
                                         x.day(),
                                         x.hour(),
                                         x.minute(),
                                         x.second())),
                TimeMode::Local => {
                    let x = Local.from_utc_datetime(&x);
                    Some((x.year(),
                          x.month(),
                          x.day(),
                          x.hour(),
                          x.minute(),
                          x.second()))
                }
            }
        };

    match datetime {
        None                                           => None,
        Some((year, month, day, hour, minute, second)) => Some(format!("{}-{:02}-{:02} {:02}:{:02}:{:02}",
                                                                       year,
                                                                       month,
                                                                       day,
                                                                       hour,
                                                                       minute,
                                                                       second)),
    }
}
