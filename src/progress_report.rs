pub struct Settings {
    silent_while_active : bool,
    silent_when_done    : bool
}

pub struct Context {
    header_printed : bool,
    header         : String
}

pub fn print_progress (settings     : &Settings,
                       context      : &mut Context,
                       start_time   : i64,
                       units_so_far : u64,
                       total_units  : u64) {
    let silent_while_active = settings.silent_while_active;
    let silent_when_done    = settings.silent_when_done;

    if !silent_while_active && silent_when_done {
        // print header once if not already
        if !context.header_printed {
            println!("{}", context.header);
            context.header_printed = true;
        }
    }
}

mod helper {
    pub fn seconds_to_hms (total_secs : i64) -> (usize, usize, usize) {
        let hour   : usize = (total_secs / (60 * 60)) as usize;
        let minute : usize = ((total_secs - (hour as i64) * 60 * 60) / 60) as usize;
        let second : usize = (total_secs
                              - (hour   as i64) * 60 * 60
                              - (minute as i64) * 60) as usize;
        (hour, minute, second)
    }

    pub fn calc_percent (units_so_far : u64, total_units : u64) -> usize {
        use std::cmp::min;
        if total_units == 0 {
            100 // just say 100% done if there is nothing to do
        } else {
            min (((units_so_far * 100) / total_units) as usize, 100)
        }
    }

    pub fn make_readable_rate (rate : f64, unit : String) -> String {
        let rate_string : String =
            if        rate >  1_000_000_000_000. {
                let adjusted_rate = rate     / 1_000_000_000_000.;
                format!("{:6.2} {}", adjusted_rate, b'T')
            } else if rate >      1_000_000_000. {
                let adjusted_rate = rate     /     1_000_000_000.;
                format!("{:6.2} {}", adjusted_rate, b'G')
            } else if rate >          1_000_000. {
                let adjusted_rate = rate     /         1_000_000.;
                format!("{:6.2} {}", adjusted_rate, b'M')
            } else if rate >              1_000. {
                let adjusted_rate = rate     /             1_000.;
                format!("{:6.0} {}", adjusted_rate, b'K')
            } else {
                format!("{:7.0}",   rate)
            };
        format!("{} {}/s", rate_string, unit)
    }
}
