// If file logging is enabled
use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::prelude::*;

use crate::common::remove_address_port;
use crate::{Request, Server};

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
    /// Is Logger enabled
    enabled: bool,

    /// What level of logs to show
    level: Level,

    /// Optional file to write logs to
    file: Option<&'static str>,

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
    /// use afire::{Logger, Level};
    ///
    /// // Create a new logger
    /// let logger = Logger::new();
    /// ```
    pub fn new() -> Logger {
        Logger {
            enabled: true,
            level: Level::Info,
            file: None,
            console: true,
        }
    }

    /// Set the log Level of a logger
    /// ## Example
    /// ```rust
    /// // Import Lib
    /// use afire::{Logger, Level};
    ///
    /// // Create a new logger
    /// let logger = Logger::new()
    ///     .level(Level::Debug);
    /// ```
    pub fn level(self, level: Level) -> Logger {
        Logger { level, ..self }
    }

    /// Set the log file of a logger
    /// ## Example
    /// ```rust
    /// // Import Lib
    /// use afire::{Logger, Level};
    ///
    /// // Create a new logger
    /// let logger = Logger::new()
    ///     .file(Some("nose.txt"));
    /// ```
    pub fn file(self, file: Option<&'static str>) -> Logger {
        Logger { file, ..self }
    }

    /// Set the log Level of a logger
    /// ## Example
    /// ```rust
    /// // Import Lib
    /// use afire::{Logger, Level};
    ///
    /// // Create a new logger
    /// let logger = Logger::new()
    ///     .console(false);
    /// ```
    pub fn console(self, console: bool) -> Logger {
        Logger { console, ..self }
    }

    /// Attach a logger to a server
    /// ## Example
    /// ```rust
    /// // Import Lib
    /// use afire::{Logger, Level, Server};
    ///
    /// // Create a new server
    /// let mut server: Server = Server::new("localhost", 1234);
    ///
    /// // Create a new logger and attach it to the server
    /// Logger::new().attach(&mut server);
    ///
    /// // Start the server
    /// // This is *still* blocking
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn attach(self, server: &mut Server) {
        let logger = RefCell::new(self);

        server.middleware(Box::new(move |req| {
            logger.borrow_mut().log(req);
            None
        }));
    }

    /// Take a request and log it
    fn log(&self, req: &Request) {
        // If the logger is disabled, don't do anything
        if !self.enabled {
            return;
        }

        match self.level {
            // Add Headers and Body to this one
            Level::Debug => {
                // Format headers as strings
                let mut headers = "".to_string();
                for i in &req.headers {
                    headers += &format!("{}: {}, ", i.name, i.value);
                }
                if headers.len() >= 2 {
                    headers = headers[0..headers.len() - 2].to_string()
                }

                // Format Query as string
                let mut query = "".to_string();
                for i in &req.query.data {
                    query += &format!("{}: {}, ", i[0], i[1]);
                }
                if query.len() >= 2 {
                    query = query[0..query.len() - 2].to_string()
                }

                let mut new_path = req.path.clone();
                if new_path.is_empty() {
                    new_path = "/".to_string();
                }

                self.send_log(format!(
                    "[{}] {} {} [{}] ({}) {{{}}}",
                    remove_address_port(&req.address),
                    req.method.to_string(),
                    new_path,
                    query,
                    headers,
                    req.body.replace('\n', "\\n")
                ))
            }

            Level::Info => {
                let mut new_path = req.path.clone();
                if new_path.is_empty() {
                    new_path = "/".to_string();
                }

                self.send_log(format!(
                    "[{}] {} {}{}",
                    remove_address_port(&req.address),
                    req.method.to_string(),
                    new_path,
                    req.query.to_string()
                ))
            }
        }
    }

    /// Send log data to file / stdout
    fn send_log(&self, data: String) {
        if self.console {
            println!("{}", data);
        }

        if self.file.is_some() {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(self.file.unwrap())
                .unwrap();

            if writeln!(file, "{}", data).is_err() {
                println!("[-] Erm... Error writhing to file '{}", self.file.unwrap())
            }
        }
    }
}

// Impl Default for Response
impl Default for Logger {
    fn default() -> Logger {
        Logger::new()
    }
}
