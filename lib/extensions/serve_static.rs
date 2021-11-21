use std::cell::RefCell;
use std::fs;

use crate::Header;
use crate::Request;
use crate::Response;
use crate::Server;

/// Serve Static Content
pub struct ServeStatic {
    /// Content Path
    pub data_dir: String,

    /// Disabled file paths (relative from data dir)
    pub disabled_files: Vec<String>,

    /// Page not found route
    pub not_found: fn(&Request, String) -> Response,

    /// Middleware
    pub middleware: Vec<fn(&Request, Vec<u8>) -> Option<Response>>,

    /// MMIE Types
    pub types: Vec<(String, String)>,
}

impl ServeStatic {
    /// Make a new Static Server
    pub fn new<T>(path: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self {
            data_dir: path.to_string(),
            disabled_files: Vec::new(),
            not_found: |req, _| {
                Response::new()
                    .status(404)
                    .text(format!("The page `{}` was not found...", req.path))
                    .header(Header::new("Content-Type", "text/plain"))
            },
            middleware: Vec::new(),
            types: vec![("html".to_owned(), "text/html".to_owned())],
        }
    }

    /// Disable serveing a file
    /// Path is relative to the dir being served
    pub fn disabled<T>(self, file_path: T) -> Self
    where
        T: std::fmt::Display,
    {
        let mut disabled = self.disabled_files;
        disabled.push(file_path.to_string());

        Self {
            disabled_files: disabled,
            ..self
        }
    }

    /// Attatch it to a Server
    pub fn attach(self, server: &mut Server) {
        let cell = RefCell::new(self);

        server.middleware(Box::new(move |req| {
            let mut path = format!("{}{}", cell.borrow().data_dir, req.path.replace("/..", ""));

            // Add Index.html if path ends with /
            if path.ends_with('/') {
                path.push_str("index.html");
            }

            // Also add '/index.html' if path dose not end with a file
            if !path.split('/').last().unwrap_or_default().contains('.') {
                path.push_str("/index.html");
            }

            if cell.borrow().disabled_files.contains(
                &path
                    .splitn(2, &cell.borrow().data_dir)
                    .last()
                    .unwrap()
                    .to_string(),
            ) {
                return Some((cell.borrow().not_found)(req, path));
            }

            // Try to read File
            Some(match fs::read(&path) {
                // If its found send it as response
                Ok(content) => Response::new().bytes(content).header(Header::new(
                    "Content-Type",
                    get_type(&path, &cell.borrow().types),
                )),

                // If not send the 404 route defined
                Err(_) => (cell.borrow().not_found)(req, path),
            })
        }));
    }
}

fn get_type(path: &str, types: &Vec<(String, String)>) -> String {
    for i in types {
        if i.0 == path.split('.').last().unwrap_or("") {
            return i.1.clone();
        }
    }

    "application/octet-stream".to_owned()
}
