use std::cmp::PartialOrd;

#[derive(Clone, Copy)]
pub struct Logger {
    pub mode: LoggerMode,
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum LoggerMode {
    Performance = 0, // Doesn't mean the code is performant, it allows v_logperf to print
    Verbose = 1,
    Disabled = 2,
}

impl Logger {
    pub(crate) fn new() -> Logger {
        Logger {
            mode: LoggerMode::Disabled,
        }
    }
}

#[macro_export]
macro_rules! v_log {
    ($logger:expr, $($arg:tt)*) => {
        if (&$logger).mode <= $crate::logger::LoggerMode::Verbose {
            $crate::_v_print!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! v_logperf {
    ($logger:expr, $($arg:tt)*) => {
        if (&$logger).mode <= $crate::logger::LoggerMode::Performance {
            $crate::_v_print!($($arg)*);
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! _v_print {
    ($($arg:tt)*) => {
        // module slice to remove "rgecko::"
        eprintln!("[{}:{}] {}", &module_path!()[8..], line!(), format_args!($($arg)*));
    }
}
