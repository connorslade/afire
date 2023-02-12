//! Basic built-in logging system

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

/// afire's global log level.
static LEVEL: AtomicU8 = AtomicU8::new(1);
/// Whether or not to colorize the log output.
static COLOR: AtomicBool = AtomicBool::new(true);

/// Log levels.
/// Used to control the verbosity of afire's internal logging.
/// The default log level is [`Level::Off`].
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Level {
    /// Disables all logging.
    Off = 0,
    /// Only shows errors.
    Error = 1,
    /// Shows [`Level::Error`] and some helpful information during startup.
    Trace = 2,
    /// Shows [`Level::Error`], [`Level::Trace`] and raw socket stuff.
    /// You probably don't need this, its really only intended for afire development.
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

    /// Gets a color code for the log level.
    /// This is used to colorize the log output if [`COLOR`] is enabled.
    fn get_color(&self) -> &'static str {
        match self {
            Level::Trace | Level::Off => "\x1b[0m",
            Level::Error => "\x1b[31m",
            Level::Debug => "\x1b[36m",
        }
    }
}

/// Sets the global afire log level.
/// Setting to [`Level::Off`] will disable all logging.
pub fn set_log_level(level: Level) {
    LEVEL.store(level as u8, Ordering::Relaxed);
}

/// Globally enables or disables colorized log output.
/// Enabled by default.
pub fn set_log_color(color: bool) {
    COLOR.store(color, Ordering::Relaxed);
}

/// Logs a message at the specified log level.
/// Hidden from the docs, as it is only intended for internal use through the [`trace!`] macro.
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

/// Simple logging system.
/// See [`mod@crate::trace`] for more information.
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
