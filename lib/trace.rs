//! Basic built-in logging system

use std::sync::atomic::{AtomicUsize, Ordering};

/// Log levels
pub enum Level {
    /// Disables all logging
    Off = 0,
    /// Only shows socket errors
    Error = 1,
    /// Shows [`Error`] and route ordering
    Trace = 2,
    /// Shows [`Error`], [`Trace`] and raw socket stuff.
    /// You probably don't need this, its intended for afire development.
    Debug = 3,
}

static LEVEL: AtomicUsize = AtomicUsize::new(1);

/// Sets the global afire log level
///
/// Setting to [`Level::Off`] will disable all logging
pub fn set_log_level(level: Level) {
    LEVEL.store(level as usize, Ordering::Relaxed);
}

#[doc(hidden)]
pub fn _trace(level: Level, str: String) {
    let log_level = LEVEL.load(Ordering::Relaxed);
    match level {
        Level::Error if log_level >= 1 => println!("[ERROR] {}", str),
        Level::Trace if log_level >= 2 => println!("[TRACE] {}", str),
        Level::Debug if log_level >= 3 => println!("[DEBUG] {}", str),
        _ => {}
    }
}

/// Internal Debug Printing
///
/// Enabled with the `tracing` feature
#[macro_export]
macro_rules! trace {
    (Level::$level: ident, $($arg: tt) *) => {
        $crate::trace::_trace($crate::trace::Level::$level, format!($($arg)+));
    };
    ($($arg : tt) +) => {
        $crate::trace::_trace($crate::trace::Level::Trace, format!($($arg)+));
    };
}
