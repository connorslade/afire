//! extension to serve static files from disk

use std::fs;

use crate::{
    middleware::{HandleError, MiddleResponse, Middleware},
    path::normalize_path,
    Request, Response,
};

type SSMiddleware = fn(req: Request, res: Response, success: bool) -> Option<(Response, bool)>;

/// Serve Static Content
#[derive(Clone)]
pub struct ServeStatic {
    /// Path to serve static content on
    ///
    /// Defaults to '/' (root)
    pub serve_path: String,

    /// Content Path
    pub data_dir: String,

    /// Disabled file paths (relative from data dir)
    pub disabled_files: Vec<String>,

    /// Page not found route
    pub not_found: fn(&Request, bool) -> Response,

    /// Middleware
    ///
    /// (Request, Static Response, Sucess [eg If file found])
    pub middleware: Vec<SSMiddleware>,

    /// MIME Types
    pub types: Vec<(String, String)>,
}

impl Middleware for ServeStatic {
    fn post(&self, req: &Request, res: Result<Response, HandleError>) -> MiddleResponse {
        let path = match res {
            Err(HandleError::NotFound(_, i)) => i,
            _ => return MiddleResponse::Continue,
        };

        if !path.starts_with(&self.serve_path) {
            return MiddleResponse::Continue;
        }

        let mut res = process_req(req, self);
        for i in self.middleware.iter().rev() {
            if let Some(i) = i(req.clone(), res.0.clone(), res.1) {
                res = i
            };
        }

        MiddleResponse::Add(res.0)
    }
}

impl ServeStatic {
    /// Make a new Static File Server
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, extension::ServeStatic, Middleware};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
    ///
    /// // Make a new static file server and attach it to the afire server
    /// ServeStatic::new("data/static").attach(&mut server);
    ///
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn new<T>(data_path: T) -> Self
    where
        T: AsRef<str>,
    {
        Self {
            serve_path: normalize_path("/".to_owned()),
            data_dir: data_path.as_ref().to_owned(),
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
    /// use afire::{Server, extension::ServeStatic, Middleware};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
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
        T: AsRef<str>,
    {
        let mut disabled = self.disabled_files;
        disabled.push(file_path.as_ref().to_owned());

        Self {
            disabled_files: disabled,
            ..self
        }
    }

    /// Disable serving many files at once
    /// Path is relative to the dir being served
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, extension::ServeStatic, Middleware};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
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
        T: AsRef<str>,
    {
        let mut disabled = self.disabled_files;
        for i in file_paths {
            disabled.push(i.as_ref().to_owned());
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
    /// use afire::{Server, extension::ServeStatic, Middleware};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
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
    pub fn middleware(self, f: SSMiddleware) -> Self {
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
    /// use afire::{Response, Server, extension::ServeStatic, Middleware};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
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
    /// use afire::{Server, extension::ServeStatic, Middleware};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
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
        T: AsRef<str>,
        M: AsRef<str>,
    {
        let mut types = self.types;

        types.push((key.as_ref().to_owned(), value.as_ref().to_owned()));

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
    /// use afire::{Server, extension::ServeStatic, Middleware};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
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
        T: AsRef<str>,
        M: AsRef<str>,
    {
        let mut new_types = new_types
            .iter()
            .map(|x| (x.0.as_ref().to_owned(), x.1.as_ref().to_owned()))
            .collect();
        let mut types = self.types;

        types.append(&mut new_types);

        Self { types, ..self }
    }

    /// Set path to serve static files on
    ///
    /// Default is '/' (root)
    /// ## Example
    /// ```rust
    /// // Import Library
    /// use afire::{Server, extension::ServeStatic, Middleware};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
    ///
    /// // Make a new static sevrer
    /// ServeStatic::new("data/static")
    ///     // Set serve path
    ///     .path("/static")
    ///     // Attatch it to the afire server
    ///     .attach(&mut server);
    ///
    /// # server.set_run(false);
    /// server.start().unwrap();
    /// ```
    pub fn path<T>(self, path: T) -> Self
    where
        T: AsRef<str>,
    {
        Self {
            serve_path: normalize_path(path.as_ref().to_owned()),
            ..self
        }
    }
}

fn process_req(req: &Request, this: &ServeStatic) -> (Response, bool) {
    let mut path = format!(
        "{}/{}",
        this.data_dir,
        safe_path(
            req.path
                .strip_prefix(&this.serve_path)
                .unwrap_or(&req.path)
                .to_owned()
        )
    );

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
        return ((this.not_found)(req, true), false);
    }

    // Try to read File
    match fs::read(&path) {
        // If its found send it as response
        Ok(content) => (
            Response::new().bytes(content).header(
                "Content-Type",
                get_type(&path, &this.types)
                    .unwrap_or_else(|| "application/octet-stream".to_owned()),
            ),
            true,
        ),

        // If not send the 404 route defined
        Err(_) => ((this.not_found)(req, false), false),
    }
}

fn get_type(path: &str, types: &[(String, String)]) -> Option<String> {
    let ext = path.split('.').last()?;
    Some(types.iter().map(|x| x.to_owned()).find(|x| x.0 == ext)?.1)
}

#[inline]
fn safe_path(mut path: String) -> String {
    path = path.replace('\\', "/");
    while path.contains("/..") {
        path = path.replace("/..", "");
    }
    path
}

/// Common MIME Types
///
/// Used by Servestatic extensions
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
