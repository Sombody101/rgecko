#[derive(Clone, Copy)]
pub struct Logger {
    pub verbose: bool,
}

impl Logger {
    pub(crate) fn new() -> Logger {
        Logger { verbose: false }
    }
}

#[macro_export]
macro_rules! v_log {
    ($logger:expr, $($arg:tt)*) => {
        if (&$logger).verbose {
            // module slice to remove "rgecko::"
            eprintln!("[{}:{}] {}", &module_path!()[8..], line!(), format_args!($($arg)*));
        }
    };
}
