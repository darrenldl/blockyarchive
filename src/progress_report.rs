use super::time;
use std::io::Write;
use std::io::stdout;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SilenceLevel {
    L0,
    L1,
    L2
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ProgressElement {
    Percentage,
    ProgressBar,
    CurrentRateShort,
    AverageRateShort,
    TimeUsedShort,
    TimeLeftShort,
    CurrentRateLong,
    AverageRateLong,
    TimeUsedLong,
    TimeLeftLong,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SilenceSettings {
    pub silent_while_active : bool,
    pub silent_when_done    : bool
}

pub struct Context {
    pub header_printed        : bool,
    pub header                : String,
    pub start_time            : i64,
    pub last_report_time      : i64,
    pub last_reported_units   : u64,
    pub unit                  : String,
    pub active_print_elements : Vec<ProgressElement>,
    pub finish_print_elements : Vec<ProgressElement>,
    pub max_print_length      : usize,
}

impl Context {
    pub fn new(header                : String,
               start_time            : i64,
               unit                  : String,
               active_print_elements : Vec<ProgressElement>,
               finish_print_elements : Vec<ProgressElement>) -> Context {
        Context {
            header_printed      : false,
            header,
            start_time,
            last_report_time    : start_time,
            last_reported_units : 0,
            unit,
            active_print_elements,
            finish_print_elements,
            max_print_length    : 0,
        }
    }
}

pub fn print_progress (settings     : &SilenceSettings,
                       context      : &mut Context,
                       units_so_far : u64,
                       total_units  : u64) {
    use std::cmp::max;

    let silent_while_active = settings.silent_while_active;
    let silent_when_done    = settings.silent_when_done;

    let percent             = helper::calc_percent(units_so_far, total_units);

    if !(silent_while_active && percent != 100)
    && !(silent_when_done    && percent == 100) {
        // print header once if not already
        if !context.header_printed {
            println!("{}", context.header);
            context.header_printed = true;
        }

        let message =
            if percent != 100 {
                make_message(context,
                             units_so_far,
                             total_units,
                             &context.active_print_elements)
            } else {
                make_message(context,
                             units_so_far,
                             total_units,
                             &context.finish_print_elements)
            };

        context.max_print_length = max(context.max_print_length,
                                       message.len());

        print!("\r");
        print!("{1:0$}", context.max_print_length, message);
        stdout().flush().unwrap();

        if percent == 100 {
            println!();
        }

        context.last_report_time    = time::get_time().sec;
        context.last_reported_units = units_so_far;
    }
}

fn make_message (context      : &Context,
                 units_so_far : u64,
                 total_units  : u64,
                 elements     : &[ProgressElement])
                 -> String {
    fn make_string_for_element (percent      : usize,
                                cur_rate     : f64,
                                avg_rate     : f64,
                                unit         : String,
                                time_used    : i64,
                                time_left    : i64,
                                units_so_far : u64,
                                total_units  : u64,
                                element      : &ProgressElement)
                                -> String {
        use self::ProgressElement::*;
        match *element {
            Percentage       => format!("{:3}%", percent),
            ProgressBar      => helper::make_progress_bar(percent),
            CurrentRateShort => format!("cur : {}", helper::make_readable_rate(cur_rate, unit)),
            CurrentRateLong  => format!("Current rate : {}", helper::make_readable_rate(cur_rate, unit)),
            AverageRateShort => format!("avg : {}", helper::make_readable_rate(avg_rate, unit)),
            AverageRateLong  => format!("Average rate : {}", helper::make_readable_rate(avg_rate, unit)),
            TimeUsedShort    => {
                let (hour, minute, second) = helper::seconds_to_hms(time_used);
                format!("used : {:02}:{:02}:{:02}", hour, minute, second) },
            TimeUsedLong     => {
                let (hour, minute, second) = helper::seconds_to_hms(time_used);
                format!("Time elapsed : {:02}:{:02}:{:02}", hour, minute, second) },
            TimeLeftShort    => {
                let (hour, minute, second) = helper::seconds_to_hms(time_left);
                format!("left : {:02}:{:02}:{:02}", hour, minute, second) },
            TimeLeftLong     => {
                let (hour, minute, second) = helper::seconds_to_hms(time_left);
                format!("Time remaining : {:02}:{:02}:{:02}", hour, minute, second) },
        }
    }

    use std::cmp::max;

    let units_remaining        = total_units - units_so_far;
    let percent                = helper::calc_percent(units_so_far, total_units);
    let cur_time               = time::get_time().sec;
    let time_used              =
        max(cur_time - context.start_time, 1);
    let time_since_last_report =
        max(cur_time - context.last_report_time, 1);
    let avg_rate               =
        units_so_far as f64 / time_used as f64;
    let cur_rate               =
        (units_so_far - context.last_reported_units) as f64
        / time_since_last_report as f64;
    let time_left              = (units_remaining as f64 / cur_rate) as i64;
    let mut res                = String::with_capacity(100);
    for e in elements.iter() {
        res.push_str(&make_string_for_element(percent,
                                              cur_rate,
                                              avg_rate,
                                              context.unit.clone(),
                                              time_used,
                                              time_left,
                                              units_so_far,
                                              total_units,
                                              e));
        res.push(' ');
    }
    res
}

pub fn silence_level_to_settings (level:SilenceLevel) -> SilenceSettings {
    match level {
        SilenceLevel::L0 => SilenceSettings { silent_while_active : false,
                                              silent_when_done    : false },
        SilenceLevel::L1 => SilenceSettings { silent_while_active : true,
                                              silent_when_done    : false },
        SilenceLevel::L2 => SilenceSettings { silent_while_active : true,
                                              silent_when_done    : true },
    }
}

mod helper {
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

    pub fn calc_percent (units_so_far : u64, total_units : u64) -> usize {
        use std::cmp::min;
        if total_units == 0 {
            100 // just say 100% done if there is nothing to do
        } else {
            min (((100 * units_so_far) / total_units) as usize, 100)
        }
    }

    pub fn make_readable_rate (rate : f64, unit : String) -> String {
        let rate_string : String =
            if        rate >  1_000_000_000_000. {
                let adjusted_rate = rate     / 1_000_000_000_000.;
                format!("{:6.2} {}", adjusted_rate, 'T')
            } else if rate >      1_000_000_000. {
                let adjusted_rate = rate     /     1_000_000_000.;
                format!("{:6.2} {}", adjusted_rate, 'G')
            } else if rate >          1_000_000. {
                let adjusted_rate = rate     /         1_000_000.;
                format!("{:6.2} {}", adjusted_rate, 'M')
            } else if rate >              1_000. {
                let adjusted_rate = rate     /             1_000.;
                format!("{:6.0} {}", adjusted_rate, 'K')
            } else {
                format!("{:7.0}",   rate)
            };
        format!("{} {}/s", rate_string, unit)
    }

    pub fn make_progress_bar (percent : usize) -> String {
        let fill_char   = '#';
        let empty_char  = '-';
        let total_len   = 25;
        let filled_len  = total_len * percent / 100;
        let empty_len   = total_len - filled_len;
        let mut res = String::with_capacity(total_len);
        res.push('[');
        for _ in 0..filled_len {
            res.push(fill_char);
        }
        for _ in 0..empty_len {
            res.push(empty_char);
        }
        res.push(']');
        res
    }
}
