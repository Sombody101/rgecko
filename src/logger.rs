#[derive(Clone, Copy)]
pub struct Logger {
    pub verbose: bool,
}

impl Logger {
    pub(crate) fn new() -> Logger {
        return Logger { verbose: false };
    }
}

#[macro_export]
macro_rules! v_log {
    ($logger:expr, $($arg:tt)*) => {
        if (&$logger).verbose {
            eprintln!("[{}:{}] {}", module_path!(), line!(), format_args!($($arg)*));
        }
    };
}
