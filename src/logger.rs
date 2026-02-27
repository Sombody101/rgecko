#[derive(Clone, Copy)]
pub struct Logger {
    pub verbose: bool,
}

impl Logger {
    pub(crate) fn new() -> Logger {
        return Logger { verbose: false };
    }
}

impl Logger {
    // pub fn verbose_closure<F, T>(&self, f: F)
    // where
    //     F: FnOnce() -> T,
    //     T: std::fmt::Display,
    // {
    //     if self.verbose {
    //         eprintln!("{}", f())
    //     }
    // }
    //
    // pub fn verbose(&self, v: &str) {
    //     if self.verbose {
    //         eprintln!("{}", v);
    //     }
    // }
}

#[macro_export]
macro_rules! v_log {
    ($logger:expr, $($arg:tt)*) => {
        if (&$logger).verbose {
            println!("[{}:{}] {}", module_path!(), line!(), format_args!($($arg)*));
        }
    };
}
