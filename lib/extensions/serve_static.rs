//! Extention to serve static files from disk

use std::fs;
use std::sync::RwLock;

use crate::{Method, Request, Response, Server};

type Middleware = fn(req: Request, res: Response, success: bool) -> Option<(Response, bool)>;

/// Serve Static Content
#[derive(Clone)]
pub struct ServeStatic {
    /// Content Path
    pub data_dir: String,

    /// Disabled file paths (relative from data dir)
    pub disabled_files: Vec<String>,

    /// Page not found route
    pub not_found: fn(&Request, bool) -> Response,

    /// Middleware
    ///
    /// (Request, Static Response, Sucess [eg If file found])
    pub middleware: Vec<Middleware>,

    /// MIME Types
    pub types: Vec<(String, String)>,
}

impl ServeStatic {
    /// Make a new Static File Server
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, ServeStatic};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server: Server = Server::new("localhost", 8080);
    ///
    /// // Make a new static file server and attach it to the afire server
    /// ServeStatic::new("data/static").attach(&mut server);
    ///
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
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
                    .header("Content-Type", "text/plain")
            },
            middleware: Vec::new(),
            types: TYPES
                .to_vec()
                .iter()
                .map(|x| (x.0.to_owned(), x.1.to_owned()))
                .collect(),
        }
    }

    /// Disable serving a file
    /// Path is relative to the dir being served
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, ServeStatic};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server: Server = Server::new("localhost", 8080);
    ///
    /// // Make a new static sevrer
    /// ServeStatic::new("data/static")
    ///     // Disable a file from being served
    ///     .disable("index.scss")
    ///     // Attatch it to the afire server
    ///     .attach(&mut server);
    ///
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn disable<T>(self, file_path: T) -> Self
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

    /// Disable serving a many files at once
    /// Path is relative to the dir being served
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, ServeStatic};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server: Server = Server::new("localhost", 8080);
    ///
    /// // Make a new static sevrer
    /// ServeStatic::new("data/static")
    ///     // Disable a vec of files from being served
    ///     .disable_vec(vec!["index.scss", "index.css.map"])
    ///     // Attatch it to the afire server
    ///     .attach(&mut server);
    ///
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn disable_vec<T>(self, file_paths: Vec<T>) -> Self
    where
        T: std::fmt::Display,
    {
        let mut disabled = self.disabled_files;
        for i in file_paths {
            disabled.push(i.to_string());
        }

        Self {
            disabled_files: disabled,
            ..self
        }
    }

    /// Add a middleware to the static file server
    ///
    /// Middleware here works much diffrently to afire middleware
    ///
    /// The middleware priority is still by most recently defined
    ///
    /// But this middleware takes functions only - no closures.
    /// The Resultes of the middleware are put togther so more then one middleware can affect thre response
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, ServeStatic};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server: Server = Server::new("localhost", 8080);
    ///
    /// // Make a new static sevrer
    /// ServeStatic::new("data/static")
    ///     // Add some middleware to the Static File Server
    ///     .middleware(|req, res, suc| {
    ///        // Print the path of the file served
    ///        println!("Staticly Served: {}", req.path);
    ///
    ///         None
    ///     })
    ///     // Attatch it to the afire server
    ///     .attach(&mut server);
    ///
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn middleware(self, f: Middleware) -> Self {
        let mut middleware = self.middleware;
        middleware.push(f);

        Self { middleware, ..self }
    }

    /// Set the not found page
    ///
    /// This will run if no file is found to serve or the file is disabled
    ///
    /// The bool in the fn parms is if the file is blocked
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Response, Server, ServeStatic};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server: Server = Server::new("localhost", 8080);
    ///
    /// // Make a new static sevrer
    /// ServeStatic::new("data/static")
    ///     // Set a new file not found page
    ///     .not_found(|_req, _dis| Response::new().status(404).text("Page Not Found!"))
    ///     // Attatch it to the afire server
    ///     .attach(&mut server);
    ///
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn not_found(self, f: fn(&Request, bool) -> Response) -> Self {
        Self {
            not_found: f,
            ..self
        }
    }

    /// Add a MIME type to the Static file Server
    ///
    /// This extension comes with alot of builtin MIME types
    /// but if you need to add more thats what this is for
    ///
    /// The key is the file extension
    ///
    /// The value is the MIME type
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, ServeStatic};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server: Server = Server::new("localhost", 8080);
    ///
    /// // Make a new static sevrer
    /// ServeStatic::new("data/static")
    ///     // Add a new MIME type
    ///     .mime_type(".3gp", "video/3gpp")
    ///     // Attatch it to the afire server
    ///     .attach(&mut server);
    ///
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn mime_type<T, M>(self, key: T, value: M) -> Self
    where
        T: std::fmt::Display,
        M: std::fmt::Display,
    {
        let mut types = self.types;

        types.push((key.to_string(), value.to_string()));

        Self { types, ..self }
    }

    /// Add a vector of MIME type to the Static file Server
    ///
    /// The key is the file extension
    ///
    /// The value is the MIME type
    ///
    /// Ex: ("html", "text/html")
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, ServeStatic};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server: Server = Server::new("localhost", 8080);
    ///
    /// // Make a new static sevrer
    /// ServeStatic::new("data/static")
    ///     // Add a new MIME type
    ///     .mime_types(vec![(".3gp", "video/3gpp")])
    ///     // Attatch it to the afire server
    ///     .attach(&mut server);
    ///
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn mime_types<T, M>(self, new_types: Vec<(T, M)>) -> Self
    where
        T: std::fmt::Display,
        M: std::fmt::Display,
    {
        let mut new_types = new_types
            .iter()
            .map(|x| (x.0.to_string(), x.1.to_string()))
            .collect();
        let mut types = self.types;

        types.append(&mut new_types);

        Self { types, ..self }
    }

    /// Attatch it to a Server
    ///
    /// Not much to say really
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, ServeStatic};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server: Server = Server::new("localhost", 8080);
    ///
    /// // Make a new static sevrer
    /// ServeStatic::new("data/static")
    ///     // Attatch it to the afire server
    ///     .attach(&mut server);
    ///
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn attach(self, server: &mut Server) {
        let cell = RwLock::new(self);

        server.route(Method::ANY, "**", move |req| {
            let mut res = process_req(req.clone(), &cell);

            for i in cell.read().unwrap().middleware.clone().iter().rev() {
                if let Some(i) = i(req.clone(), res.0.clone(), res.1) {
                    res = i
                };
            }

            res.0
        });
    }
}

fn process_req(req: Request, cell: &RwLock<ServeStatic>) -> (Response, bool) {
    let this = cell.read().unwrap();

    let mut path = format!("{}{}", this.data_dir, req.path.replace("/..", ""));

    // Add Index.html if path ends with /
    if path.ends_with('/') {
        path.push_str("index.html");
    }

    // Also add '/index.html' if path dose not end with a file
    if !path.split('/').last().unwrap_or_default().contains('.') {
        path.push_str("/index.html");
    }

    if this
        .disabled_files
        .contains(&path.splitn(2, &this.data_dir).last().unwrap().to_string())
    {
        return ((this.not_found)(&req, true), false);
    }

    // Try to read File
    match fs::read(&path) {
        // If its found send it as response
        Ok(content) => (
            Response::new()
                .bytes(content)
                .header("Content-Type", get_type(&path, &this.types)),
            true,
        ),

        // If not send the 404 route defined
        Err(_) => ((this.not_found)(&req, false), false),
    }
}

fn get_type(path: &str, types: &[(String, String)]) -> String {
    for i in types {
        if i.0 == path.split('.').last().unwrap_or("") {
            return i.1.to_owned();
        }
    }

    "application/octet-stream".to_owned()
}

/// Common MIME Types
///
/// Used by Servestatic Extentions
pub const TYPES: [(&str, &str); 56] = [
    ("html", "text/html"),
    ("css", "text/css"),
    ("js", "application/javascript"),
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("ico", "image/x-icon"),
    ("svg", "image/svg+xml"),
    ("txt", "text/plain"),
    ("aac", "audio/aac"),
    ("avi", "video/x-msvideo"),
    ("bin", "application/octet-stream"),
    ("bmp", "image/bmp"),
    ("bz", "application/x-bzip"),
    ("bz2", "application/x-bzip2"),
    ("cda", "application/x-cdf"),
    ("csv", "text/csv"),
    ("epub", "application/epub+zip"),
    ("gz", "application/gzip"),
    ("htm", "text/html"),
    ("ics", "text/calendar"),
    ("jar", "application/java-archive"),
    ("json", "application/json"),
    ("jsonld", "application/ld+json"),
    ("midi", "audio/midi audio/x-midi"),
    ("mid", "audio/midi audio/x-midi"),
    ("mjs", "text/javascript"),
    ("mp3", "audio/mpeg"),
    ("mp4", "video/mp4"),
    ("mpeg", "video/mpeg"),
    ("oga", "audio/ogg"),
    ("ogv", "video/ogg"),
    ("ogx", "application/ogg"),
    ("opus", "audio/opus"),
    ("otf", "font/otf"),
    ("pdf", "application/pdf"),
    ("rar", "application/vnd.rar"),
    ("rtf", "application/rtf"),
    ("sh", "application/x-sh"),
    ("swf", "application/x-shockwave-flash"),
    ("tar", "application/x-tar"),
    ("tif", "image/tiff"),
    ("tiff", "image/tiff"),
    ("ts", "text/x-typescript"),
    ("ttf", "font/ttf"),
    ("wav", "audio/wav"),
    ("weba", "audio/webm"),
    ("webm", "video/webm"),
    ("webp", "image/webp"),
    ("woff", "font/woff"),
    ("woff2", "font/woff2"),
    ("xhtml", "application/xhtml+xml"),
    ("xml", "application/xml"),
    ("zip", "application/zip"),
    ("7z", "application/x-7z-compressed"),
];
