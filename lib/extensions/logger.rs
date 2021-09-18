// If file logging is enabled
use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::prelude::*;

use crate::{Request, Server};

/// Define Log Levels
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
    /// ## Example
    /// ```rust
    /// // Import Lib
    /// use afire::{Logger, Level};
    ///
    /// // Create a new logger
    /// let logger = Logger::new(Level::Debug, Some("logs/log.txt"), true);
    /// ```
    pub fn new(level: Level, file: Option<&'static str>, console: bool) -> Logger {
        Logger {
            enabled: true,
            level,
            file,
            console,
        }
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

                self.send_log(format!(
                    "[{}] {} {} [{}] ({}) {{{}}}",
                    remove_address_port(&req.address),
                    req.method.to_string(),
                    req.path,
                    query,
                    headers,
                    req.body.replace('\n', "\\n")
                ))
            }

            Level::Info => self.send_log(format!(
                "[{}] {} {}",
                remove_address_port(&req.address),
                req.method.to_string(),
                req.path
            )),
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
    /// Logger::attach(&mut server, Logger::new(Level::Debug, Some("logs/log.txt"), true));
    ///
    /// // Start the server
    /// // This is *still* blocking
    /// # server.set_run(false);
    /// server.start();
    /// ```
    pub fn attach(server: &mut Server, logger: Logger) {
        let logger = RefCell::new(logger);

        server.every(Box::new(move |req| {
            logger.borrow_mut().log(req);
            None
        }));
    }
}

/// Remove the port from an address
///
/// '192.168.1.26:1234' -> '192.168.1.26'
fn remove_address_port(address: &str) -> String {
    address
        .split(':')
        .collect::<Vec<&str>>()
        .first()
        .unwrap_or(&"null")
        .to_string()
}
