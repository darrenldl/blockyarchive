#![cfg(test)]

use crate::time_utils::*;

quickcheck! {
    fn qc_seconds_to_hms_to_seconds(total_secs : i64) -> bool {
        let (hour, minute, second) = seconds_to_hms(total_secs);

        hour as i64 * 60 * 60
            + minute as i64 * 60 + second as i64 == total_secs
    }
}
