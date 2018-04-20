#![allow(dead_code)]
use time_utils;
use misc_utils::f64_max;
use std::io::Write;
use std::io::stdout;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Barrier;
use std::thread;
use std::time::Duration;
use misc_utils::to_camelcase;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PRVerbosityLevel {
    L0,
    L1,
    L2,
    LJSON,
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
struct VerbositySettings {
    verbose_while_active : bool,
    verbose_when_done    : bool,
    json_enabled         : bool,
}

pub struct Context {
    header_printed        : bool,
    finish_printed        : bool,
    header                : String,
    last_report_time      : f64,
    last_reported_units   : u64,
    unit                  : String,
    active_print_elements : Vec<ProgressElement>,
    finish_print_elements : Vec<ProgressElement>,
    max_print_length      : usize,
    verbosity_settings    : VerbositySettings,
}

impl Context {
    pub fn new(header                : &str,
               unit                  : &str,
               pr_verbosity_level    : PRVerbosityLevel,
               active_print_elements : Vec<ProgressElement>,
               finish_print_elements : Vec<ProgressElement>) -> Context {
        Context {
            header_printed        : false,
            finish_printed        : false,
            header                : String::from(header),
            last_report_time      : 0.,
            last_reported_units   : 0,
            unit                  : String::from(unit),
            active_print_elements,
            finish_print_elements,
            max_print_length      : 0,
            verbosity_settings    : pr_verbosity_level_to_settings(pr_verbosity_level),
        }
    }
}

pub struct ProgressReporter<T : 'static + ProgressReport + Send> {
    start_barrier    : Arc<Barrier>,
    start_flag       : Arc<AtomicBool>,
    shutdown_flag    : Arc<AtomicBool>,
    shutdown_barrier : Arc<Barrier>,
    stats            : Arc<Mutex<T>>,
    context          : Arc<Mutex<Context>>,
    active_flag      : Arc<AtomicBool>,
}

impl<T : 'static + ProgressReport + Send> ProgressReporter<T> {
    pub fn new(stats              : &Arc<Mutex<T>>,
               header             : &str,
               unit               : &str,
               pr_verbosity_level : PRVerbosityLevel)
               -> ProgressReporter<T> {
        use self::ProgressElement::*;
        let stats                   = Arc::clone(stats);
        let context                 =
            Arc::new(Mutex::new(Context::new(header,
                                             unit,
                                             pr_verbosity_level,
                                             vec![ProgressBar,
                                                  Percentage,
                                                  CurrentRateShort,
                                                  TimeUsedShort,
                                                  TimeLeftShort],
                                             vec![TimeUsedLong,
                                                  AverageRateLong])));
        let start_barrier           = Arc::new(Barrier::new(2));
        let start_flag              = Arc::new(AtomicBool::new(false));
        let shutdown_flag           = Arc::new(AtomicBool::new(false));
        let shutdown_barrier        = Arc::new(Barrier::new(2));
        let active_flag             = Arc::new(AtomicBool::new(true));
        let runner_stats            = Arc::clone(&stats);
        let runner_context          = Arc::clone(&context);
        let runner_start_barrier    = Arc::clone(&start_barrier);
        let runner_shutdown_flag    = Arc::clone(&shutdown_flag);
        let runner_shutdown_barrier = Arc::clone(&shutdown_barrier);
        let runner_active_flag      = Arc::clone(&active_flag);
        thread::spawn(move || {
            // waiting to be kickstarted
            runner_start_barrier.wait();

            // print at least once so the header is at top
            print_progress::<T>(&mut runner_context.lock().unwrap(),
                                &mut runner_stats.lock().unwrap(),
                                false);

            // let start() know progress text has been printed
            runner_start_barrier.wait();

            while !runner_shutdown_flag.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(300));

                if runner_active_flag.load(Ordering::SeqCst) {
                    print_progress::<T>(&mut runner_context.lock().unwrap(),
                                        &mut runner_stats.lock().unwrap(),
                                        false);
                }
            }

            print_progress::<T>(&mut runner_context.lock().unwrap(),
                                &mut runner_stats.lock().unwrap(),
                                true);

            runner_shutdown_barrier.wait();
        });
        ProgressReporter {
            start_barrier,
            start_flag,
            shutdown_flag,
            shutdown_barrier,
            stats,
            context,
            active_flag,
        }
    }

    pub fn start(&self) {
        if !self.start_flag.swap(true, Ordering::SeqCst) {

            self.stats.lock().unwrap().set_start_time();

            // first wait to kick start
            self.start_barrier.wait();

            // second wait for runner to finish printing for the first time
            self.start_barrier.wait();
        }
    }

    pub fn pause(&self) {
        // overwrite progress text
        eprint!("\r{1:0$}", self.context.lock().unwrap().max_print_length, "");
        eprint!("\r");
        self.active_flag.store(false, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        self.active_flag.store(true, Ordering::SeqCst);
    }

    pub fn stop(&self) {
        if self.start_flag.load(Ordering::SeqCst)
            && !self.shutdown_flag.load(Ordering::SeqCst)
        {
            self.stats.lock().unwrap().set_end_time();

            self.shutdown_flag.store(true, Ordering::SeqCst);

            self.shutdown_barrier.wait();
        }
    }
}

impl<T : 'static + ProgressReport + Send> Drop for ProgressReporter<T> {
    fn drop(&mut self) {
        self.stop();
    }
}

pub trait ProgressReport {
    fn start_time_mut(&mut self) -> &mut f64;

    fn end_time_mut(&mut self) -> &mut f64;

    fn units_so_far(&self) -> u64;

    fn total_units(&self) -> u64;

    fn set_start_time(&mut self) {
        *self.start_time_mut() = time_utils::get_time_now(time_utils::TimeMode::UTC);
    }

    fn get_start_time(&mut self) -> f64 {
        *self.start_time_mut()
    }

    fn set_end_time(&mut self) {
        *self.end_time_mut() = time_utils::get_time_now(time_utils::TimeMode::UTC);
    }

    fn get_end_time(&mut self) -> f64 {
        *self.end_time_mut()
    }
}

pub fn print_progress<T>(context        : &mut Context,
                         stats          : &mut T,
                         pretend_finish : bool)
    where T : ProgressReport
{
    use std::cmp::max;

    let verbose_while_active = context.verbosity_settings.verbose_while_active;
    let verbose_when_done    = context.verbosity_settings.verbose_when_done;

    let units_so_far = stats.units_so_far();
    let total_units  = stats.total_units();

    let percent = helper::calc_percent(units_so_far, total_units);

    let progress_complete = percent == 100 || pretend_finish;

    if ((verbose_while_active && !progress_complete)
        || (verbose_when_done && progress_complete))
        && !(context.finish_printed && progress_complete)
    {
        if context.verbosity_settings.json_enabled {
            let message =
                make_message(context,
                             stats.get_start_time(),
                             stats.get_end_time(),
                             units_so_far,
                             total_units,
                             &[]);
            eprintln!("{}", message);
        } else {
            // print header once if not already
            if !context.header_printed {
                eprintln!("{}", context.header);
                context.header_printed = true;
            }

            let message =
                if progress_complete {
                    make_message(context,
                                 stats.get_start_time(),
                                 stats.get_end_time(),
                                 units_so_far,
                                 total_units,
                                 &context.finish_print_elements)
                } else {
                    make_message(context,
                                 stats.get_start_time(),
                                 stats.get_end_time(),
                                 units_so_far,
                                 total_units,
                                 &context.active_print_elements)
                };

            context.max_print_length = max(context.max_print_length,
                                           message.len());

            eprint!("\r{1:0$}", context.max_print_length, message);
            stdout().flush().unwrap();
        }

        if progress_complete && !context.finish_printed {
            if !context.verbosity_settings.json_enabled {
                eprintln!();
            }
            context.finish_printed = true;
        }

        context.last_report_time    = time_utils::get_time_now(time_utils::TimeMode::UTC);
        context.last_reported_units = units_so_far;
    }
}

pub fn string_to_verbosity_level(string : &str) -> Result<PRVerbosityLevel, ()> {
    match string {
        "0" => Ok(PRVerbosityLevel::L0),
        "1" => Ok(PRVerbosityLevel::L1),
        "2" => Ok(PRVerbosityLevel::L2),
        _   => Err(())
    }
}

fn make_message(context      : &Context,
                start_time   : f64,
                end_time     : f64,
                units_so_far : u64,
                total_units  : u64,
                elements     : &[ProgressElement])
                -> String {
    fn make_string_for_element(percent   : usize,
                               cur_rate  : f64,
                               avg_rate  : f64,
                               unit      : String,
                               time_used : f64,
                               time_left : f64,
                               element   : &ProgressElement)
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
                let (hour, minute, second) = time_utils::seconds_to_hms(time_used as i64);
                format!("used : {:02}:{:02}:{:02}", hour, minute, second) },
            TimeUsedLong     => {
                let (hour, minute, second) = time_utils::seconds_to_hms(time_used as i64);
                format!("Time elapsed : {:02}:{:02}:{:02}", hour, minute, second) },
            TimeLeftShort    => {
                let (hour, minute, second) = time_utils::seconds_to_hms(time_left as i64);
                format!("left : {:02}:{:02}:{:02}", hour, minute, second) },
            TimeLeftLong     => {
                let (hour, minute, second) = time_utils::seconds_to_hms(time_left as i64);
                format!("Time remaining : {:02}:{:02}:{:02}", hour, minute, second) },
        }
    }

    let units_remaining        =
        if total_units >= units_so_far {
            total_units - units_so_far
        } else {
            0
        };
    let percent                = helper::calc_percent(units_so_far, total_units);
    let cur_time               = time_utils::get_time_now(time_utils::TimeMode::UTC);
    let time_used              =
        if percent < 100 {
            f64_max(cur_time - start_time, 0.1)
        } else {
            f64_max(end_time - start_time, 0.1)
        };
    let time_since_last_report =
        f64_max(cur_time - context.last_report_time, 0.1);
    let avg_rate               =
        units_so_far as f64 / time_used;
    let cur_rate               =
        (units_so_far - context.last_reported_units) as f64
        / time_since_last_report;
    let time_left              = units_remaining as f64 / cur_rate;
    let mut res                = String::with_capacity(150);
    if context.verbosity_settings.json_enabled {
        res.push_str("{");
        res.push_str(&format!(" \"{}\": {} ", to_camelcase("units so far"), units_so_far));
        res.push_str(&format!(",\"{}\": {} ", to_camelcase("total units"),  total_units));
        res.push_str(&format!(",\"{}\": {} ", to_camelcase("cur per sec"),  cur_rate));
        res.push_str(&format!(",\"{}\": {} ", to_camelcase("avg per sec"),  cur_rate));
        res.push_str(&format!(",\"{}\": {} ", to_camelcase("time used"),    time_used));
        res.push_str(&format!(",\"{}\": {} ", to_camelcase("time left"),    time_left));
        res.push_str("}");
    } else {
        for e in elements.iter() {
            res.push_str(&make_string_for_element(percent,
                                                  cur_rate,
                                                  avg_rate,
                                                  context.unit.clone(),
                                                  time_used,
                                                  time_left,
                                                  e));
            res.push_str("  ");
        }
    }
    res
}

fn pr_verbosity_level_to_settings (level:PRVerbosityLevel) -> VerbositySettings {
    match level {
        PRVerbosityLevel::L0    => VerbositySettings { verbose_while_active : false,
                                                       verbose_when_done    : false,
                                                       json_enabled         : false, },
        PRVerbosityLevel::L1    => VerbositySettings { verbose_while_active : false,
                                                       verbose_when_done    : true,
                                                       json_enabled         : false, },
        PRVerbosityLevel::L2    => VerbositySettings { verbose_while_active : true,
                                                       verbose_when_done    : true,
                                                       json_enabled         : false, },
        PRVerbosityLevel::LJSON => VerbositySettings { verbose_while_active : true,
                                                       verbose_when_done    : true,
                                                       json_enabled         : true,  },
    }
}

mod helper {
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
                format!("{:6.2}{}", adjusted_rate, 'T')
            } else if rate >      1_000_000_000. {
                let adjusted_rate = rate     /     1_000_000_000.;
                format!("{:6.2}{}", adjusted_rate, 'G')
            } else if rate >          1_000_000. {
                let adjusted_rate = rate     /         1_000_000.;
                format!("{:6.2}{}", adjusted_rate, 'M')
            } else if rate >              1_000. {
                let adjusted_rate = rate     /             1_000.;
                format!("{:6.0}{}", adjusted_rate, 'K')
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
