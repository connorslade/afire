//! Basic built-in logging system

use std::{
    fmt::{self, Display},
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        RwLock,
    },
};

/// afire's global log level.
static LEVEL: AtomicU8 = AtomicU8::new(1);
/// Whether or not to colorize the log output.
static COLOR: AtomicBool = AtomicBool::new(true);
/// The global log formatter.
/// Will use [`DefaultFormatter`] if none is set.
static FORMATTER: RwLock<Option<Box<dyn Formatter + Send + Sync + 'static>>> = RwLock::new(None);
/// Whether or not a formatter has been set.
/// Used because loading a bool is faster than a RwLock.
static FORMATTER_PRESENT: AtomicBool = AtomicBool::new(false);

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
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Off => "OFF",
            Level::Error => "ERROR",
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
        }
    }

    /// Gets the ansi color code for the log level.
    /// By default, this is used to colorize the log output if color is enabled.
    pub fn get_color(&self) -> &'static str {
        match self {
            Level::Trace | Level::Off => "\x1b[0m",
            Level::Error => "\x1b[31m",
            Level::Debug => "\x1b[36m",
        }
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
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

/// Sets the global log formatter.
/// This can be used to redirect afire's log output to a file, or to another logging system.
/// By default, afire will use a simple formatter that prints to stdout.
pub fn set_log_formatter(formatter: impl Formatter + Send + Sync + 'static) {
    FORMATTER_PRESENT.store(true, Ordering::Relaxed);
    *FORMATTER.write().unwrap() = Some(Box::new(formatter));
}

/// Logs a message at the specified log level.
/// Hidden from the docs, as it is only intended for internal use through the [`trace!`] macro.
#[doc(hidden)]
pub fn _trace(level: Level, str: String) {
    let log_level = LEVEL.load(Ordering::Relaxed);
    if level as u8 > log_level {
        return;
    }

    if FORMATTER_PRESENT.load(Ordering::Relaxed) {
        let formatter = FORMATTER.read().unwrap();
        if let Some(formatter) = &*formatter {
            formatter.format(level, COLOR.load(Ordering::Relaxed), str);
            return;
        }
    }

    DefaultFormatter.format(level, COLOR.load(Ordering::Relaxed), str);
}

pub(crate) fn emoji(emoji: &str) -> String {
    #[cfg(feature = "emoji-logging")]
    return emoji.to_owned() + " ";
    #[cfg(not(feature = "emoji-logging"))]
    String::new()
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

/// A trait for custom log formatters.
pub trait Formatter {
    /// Processes a log message.
    /// This will usually print the message to stdout, write it to a file, or pass it to another logging system.
    ///
    /// Note: Only log messages with a level equal to or higher than the global log level will be passed to the formatter.
    fn format(&self, level: Level, color: bool, msg: String);
}

/// The default log formatter.
/// afire will use this if no custom formatter is set.
///
/// Prints logs to stdout in the following format:
/// ```text
/// [LEVEL] MESSAGE
/// ```
pub struct DefaultFormatter;

impl Formatter for DefaultFormatter {
    fn format(&self, level: Level, _color: bool, msg: String) {
        let color = COLOR.load(Ordering::Relaxed);

        println!(
            "[{}] {}{}{}",
            level.as_str(),
            if color { level.get_color() } else { "" },
            msg,
            if color { "\x1b[0m" } else { "" }
        );
    }
}
