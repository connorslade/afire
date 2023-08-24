//! This is afire's built-in logging system.
//! Enabled with the `tracing` feature (enabled by default).
//! The default log level is [`Level::Error`].
//!
//! It's used for both logging helpful information during startup, and for helping me debug afire (set log level to Debug and use websockets to [see what I mean](https://i.imgur.com/fJV6UYx.png) :sob:).
//! You can make use of the logger through the [`trace!`] macro in your own code.
//!
//! Because the logger is used all over the place internally, it would not be practical to allow customizing it per server, so all configuration is global.
//! There are three settings that can be configured: the log level, whether or not to colorize the log output, and the log formatter.
//! The log formatter is the most powerful of these, and can be used to do anything from redirecting the log output to a file, to passing it to another logging system (check [this](https://github.com/rcsc/amplitude/blob/48570f3df7280efc6058dcf462bb09f1a7f5e235/amplitude/src/logger.rs) out to see afire's logs redirected to the [tracing](http://crates.io/crates/tracing) crate).
//!
//! ## Example
//! ```
//! # use afire::{
//! #     trace,
//! #     trace::{set_log_color, set_log_formatter, set_log_level, Formatter, Level},
//! # };
//! fn main() {
//!     // Set the log level to trace at the start of the program.
//!     set_log_level(Level::Trace);
//!
//!     // Set the global log formatter to our custom formatter.
//!     set_log_formatter(MyFormatter);
//!
//!     // Disable colorized log output.
//!     set_log_color(false);
//!
//!     // Use the trace! macro to log some stuff.
//!     trace!(Level::Trace, "Hello, world!");
//!     trace!(Level::Error, "Buffer: {:?}", [1, 2, 3, 4]);
//!
//!     let world = "World!";
//!     trace!(Level::Debug, "Hello {world}");
//! }
//!
//! struct MyFormatter;
//!
//! impl Formatter for MyFormatter {
//!     fn format(&self, level: Level, _color: bool, msg: std::fmt::Arguments) {
//!         println!("{}: {}", level, msg);
//!     }
//! }
//! ```

use std::{
    fmt::{self, Arguments, Debug, Display},
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
/// Used because loading an atomic bool is faster than a RwLock.
/// This is always loaded before the RwLock to improve performance when using the default formatter.
static FORMATTER_PRESENT: AtomicBool = AtomicBool::new(false);

/// Log levels.
/// Used to control the verbosity of afire's internal logging.
/// The default log level is [`Level::Error`].
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
        f.write_str(self.as_str())
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
pub fn _trace(level: Level, fmt: Arguments) {
    let log_level = LEVEL.load(Ordering::Relaxed);
    if level as u8 > log_level {
        return;
    }

    if FORMATTER_PRESENT.load(Ordering::Relaxed) {
        let formatter = FORMATTER.read().unwrap();
        if let Some(formatter) = &*formatter {
            formatter.format(level, COLOR.load(Ordering::Relaxed), fmt);
            return;
        }
    }

    DefaultFormatter.format(level, COLOR.load(Ordering::Relaxed), fmt);
}

// TODO: convert to macro for compile time concat!
// this is a totally normal and necessary function
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
    (Level::$level: ident, $($arg: tt) +) => {
        #[cfg(feature = "tracing")]
        $crate::trace::_trace($crate::trace::Level::$level, format_args!($($arg)+));
    };
    ($($arg: tt) +) => {
        #[cfg(feature = "tracing")]
        $crate::trace::_trace($crate::trace::Level::Trace, format_args!($($arg)+));
    };
}

/// A wrapper for [`Display`] or [`Debug`] types that only evaluates the inner value when it is actually used.
/// This is useful built-in debug logging, as it allows you to avoid the overhead of formatting the message if it is not going to be logged (due to log level, or custom formatter).
/// ## Example
/// ```
/// # use afire::{trace, trace::LazyFmt};
/// let buffer = [72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, /* ... */];
/// // Calling `from_utf8_lossy` on large buffers unnecessarily is very expensive.
/// // Using `LazyFmt` allows us to avoid this overhead if the message is not going to be logged.
/// trace!(Level::Debug, "Buffer: {}", LazyFmt(|| String::from_utf8_lossy(&buffer)));
///
pub struct LazyFmt<T, F: Fn() -> T>(pub F);

impl<T: Display, F: Fn() -> T> Display for LazyFmt<T, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", (self.0)())
    }
}

impl<T: Debug, F: Fn() -> T> Debug for LazyFmt<T, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", (self.0)())
    }
}

/// A trait for custom log formatters.
pub trait Formatter {
    /// Processes a log message.
    /// This will usually print the message to stdout, write it to a file, or pass it to another logging system.
    ///
    /// Note: Only log messages with a level equal to or higher than the global log level will be passed to the formatter.
    fn format(&self, level: Level, color: bool, msg: Arguments);
}

/// The default log formatter.
/// afire will use this if no custom formatter is set.
///
/// Prints logs to stdout in the following format with ansi colorization (unless disabled).
/// ```text
/// [LEVEL] MESSAGE
/// ```
pub struct DefaultFormatter;

impl Formatter for DefaultFormatter {
    fn format(&self, level: Level, color: bool, msg: Arguments) {
        println!(
            "[{}] {}{}{}",
            level.as_str(),
            if color { level.get_color() } else { "" },
            msg,
            if color { "\x1b[0m" } else { "" }
        );
    }
}
