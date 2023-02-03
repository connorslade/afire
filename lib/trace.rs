//! Basic built-in logging system

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

static LEVEL: AtomicU8 = AtomicU8::new(1);
static COLOR: AtomicBool = AtomicBool::new(true);

/// Log levels
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
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

impl Level {
    /// Returns the log level as a string
    fn as_str(&self) -> &'static str {
        match self {
            Level::Off => "OFF",
            Level::Error => "ERROR",
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
        }
    }

    fn get_color(&self) -> &'static str {
        match self {
            Level::Trace | Level::Off => "\x1b[0m",
            Level::Error => "\x1b[31m",
            Level::Debug => "\x1b[36m",
        }
    }
}

/// Sets the global afire log level
///
/// Setting to [`Level::Off`] will disable all logging
pub fn set_log_level(level: Level) {
    LEVEL.store(level as u8, Ordering::Relaxed);
}

#[doc(hidden)]
pub fn _trace(level: Level, str: String) {
    let log_level = LEVEL.load(Ordering::Relaxed);
    let color = COLOR.load(Ordering::Relaxed);

    if level as u8 > log_level {
        return;
    }

    println!(
        "[{}] {}{}{}",
        level.as_str(),
        if color { level.get_color() } else { "" },
        str,
        if color { "\x1b[0m" } else { "" }
    );
}

/// Internal Debug Printing
///
/// Enabled with the `tracing` feature
#[macro_export]
macro_rules! trace {
    (Level::$level: ident, $($arg: tt) *) => {
        #[cfg(feature = "tracing")]
        $crate::trace::_trace($crate::trace::Level::$level, format!($($arg)+));
    };
    ($($arg : tt) +) => {
        #[cfg(feature = "tracing")]
        $crate::trace::_trace($crate::trace::Level::Trace, format!($($arg)+));
    };
}
