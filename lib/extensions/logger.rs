//! Log requests to the console or a file

// If file logging is enabled
use std::fs::{File, OpenOptions};
use std::io::{self, prelude::*};
use std::path::Path;
use std::sync::Mutex;

use crate::{extension::RealIp, HeaderType, Middleware, Request, Response};

/// Define Log Levels
#[derive(Debug)]
pub enum Level {
    /// Give lots of information on what's going on.
    ///
    /// Adds Request Headers and Body
    Debug,

    /// Give a reasonable amount of information on what's going on.
    ///
    /// So IP, Method and Path
    Info,
}

/// Logger
#[derive(Debug)]
pub struct Logger {
    /// What level of logs to show
    level: Level,

    /// What header to use to get the clients actual IP
    real_ip: Option<HeaderType>,

    /// Optional file to write logs to
    file: Option<Mutex<File>>,

    /// If logs should also be printed to stdout
    console: bool,
}

impl Logger {
    /// Make a new logger
    ///
    /// The default settings are as follows
    ///
    /// - Log Level: `Level::Info`
    ///
    /// - File: `None`
    ///
    /// - Console: `true`
    /// ## Example
    /// ```rust
    /// // Import Lib
    /// use afire::extension::logger::{Logger, Level};
    ///
    /// // Create a new logger
    /// let logger = Logger::new();
    /// ```
    pub fn new() -> Logger {
        Logger {
            level: Level::Info,
            real_ip: None,
            file: None,
            console: true,
        }
    }

    /// Set the log Level of a logger
    /// ## Example
    /// ```rust
    /// // Import Lib
    /// use afire::extension::logger::{Logger, Level};
    ///
    /// // Create a new logger
    /// let logger = Logger::new()
    ///     .level(Level::Debug);
    /// ```
    pub fn level(self, level: Level) -> Self {
        Self { level, ..self }
    }

    /// Uses the [`RealIP`] extention for log IPs.
    /// You will need to supply the header that will contain the IP address, for example the [X-Forwarded-For header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Forwarded-For) ([`HeaderType::XForwardedFor`])
    ///
    /// **Warning**: Make sure your reverse proxy is overwriting the specified header on the incoming requests so clients cant spoof their original Ips.
    pub fn real_ip(self, real_ip: HeaderType) -> Self {
        Self {
            real_ip: Some(real_ip),
            ..self
        }
    }

    /// Set the log file of a logger
    /// ## Example
    /// ```rust
    /// // Import Lib
    /// use afire::extension::logger::{Logger, Level};
    ///
    /// // Create a new logger and enable logging to file
    /// let logger = Logger::new()
    ///     .file("nose.txt");
    /// ```
    pub fn file(self, file: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self {
            file: Some(Mutex::new(
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(file)?,
            )),
            ..self
        })
    }

    /// Enable writing events to stdout
    /// ## Example
    /// ```rust
    /// // Import Lib
    /// use afire::extension::logger::{Logger, Level};
    ///
    /// // Create a new logger and enable console
    /// let logger = Logger::new()
    ///     .console(true );
    /// ```
    pub fn console(self, console: bool) -> Self {
        Self { console, ..self }
    }

    /// Take a request and log it
    fn log(&self, req: &Request) {
        let ip = match &self.real_ip {
            Some(i) => req.real_ip_header(i),
            None => req.address.ip(),
        };

        match self.level {
            // Add Headers and Body to this one
            Level::Debug => {
                // Format headers as strings
                let mut headers = "".to_string();
                for i in &*req.headers {
                    headers += &format!("{}: {}, ", i.name, i.value);
                }
                if headers.len() >= 2 {
                    headers = headers[0..headers.len() - 2].to_string()
                }

                // Format Query as string
                let mut query = "".to_string();
                for i in req.query.iter() {
                    query += &format!("{}: {}, ", i[0], i[1]);
                }
                if query.len() >= 2 {
                    query = query[0..query.len() - 2].to_string()
                }

                let mut new_path = req.path.to_owned();
                if new_path.is_empty() {
                    new_path = "/".to_string();
                }

                self.send_log(format!(
                    "[{ip}] {} {} [{}] ({}) {{{}}}",
                    req.method,
                    new_path,
                    query,
                    headers,
                    String::from_utf8_lossy(&req.body).replace('\n', "\\n")
                ))
            }

            Level::Info => {
                let mut new_path = req.path.clone();
                if new_path.is_empty() {
                    new_path = "/".to_string();
                }

                self.send_log(format!("[{ip}] {} {}{}", req.method, new_path, req.query))
            }
        }
    }

    /// Send log data to file / stdout
    fn send_log(&self, data: String) {
        if self.console {
            println!("{data}");
        }

        if let Some(i) = &self.file {
            if let Err(e) = writeln!(i.lock().unwrap(), "{data}") {
                eprintln!("[-] Erm... Error writhing to log file: {e}")
            }
        }
    }
}

impl Middleware for Logger {
    fn end(&self, req: &Request, _res: &Response) {
        self.log(req);
    }
}

// Impl Default for Response
impl Default for Logger {
    fn default() -> Logger {
        Logger::new()
    }
}
