//! Serve Static Content from the file system.

use std::{borrow::Cow, fs::File, sync::Arc};

use crate::{
    error::{HandleError, Result},
    middleware::{MiddleResult, Middleware},
    path::normalize_path,
    Error, HeaderType, Request, Response, Status,
};

type SSMiddleware = Box<dyn Fn(Arc<Request>, &mut Response, &mut bool) + Send + Sync>;

/// Serve Static Content
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
    pub not_found: fn(Arc<Request>, bool) -> Response,

    /// Middleware
    ///
    /// (Request, Static Response, success [eg If file found])
    pub middleware: Vec<SSMiddleware>,

    /// MIME Types
    pub types: Vec<(String, String)>,
}

impl Middleware for ServeStatic {
    fn post_raw(&self, req: Result<Arc<Request>>, res: &mut Result<Response>) -> MiddleResult {
        let req = match req {
            Ok(req) => req,
            Err(_) => return MiddleResult::Continue,
        };

        let path = match res {
            Err(Error::Handle(e)) => match &**e {
                HandleError::NotFound(_, i) => i,
                _ => return MiddleResult::Continue,
            },
            _ => return MiddleResult::Continue,
        };

        if !path.starts_with(&self.serve_path) {
            return MiddleResult::Continue;
        }

        let mut new_res = process_req(req.clone(), self);
        for i in self.middleware.iter().rev() {
            i(req.clone(), &mut new_res.0, &mut new_res.1);
        }

        *res = Ok(new_res.0);
        MiddleResult::Continue
    }
}

impl ServeStatic {
    /// Make a new Static File Server
    /// ## Example
    /// ```rust,no_run
    /// // Import Library
    /// use afire::{Server, extension::ServeStatic, Middleware};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
    ///
    /// // Make a new static file server and attach it to the afire server
    /// ServeStatic::new("data/static").attach(&mut server);
    ///
    /// server.run().unwrap();
    /// ```
    pub fn new(data_path: impl AsRef<str>) -> Self {
        Self {
            serve_path: normalize_path("/".to_owned()),
            data_dir: data_path.as_ref().to_string(),
            disabled_files: Vec::new(),
            middleware: Vec::new(),
            not_found: |req, _| {
                Response::new()
                    .status(Status::NotFound)
                    .text(format!("The page `{}` was not found...", req.path))
                    .header(HeaderType::ContentType, "text/plain")
            },
            types: Vec::new(),
        }
    }

    /// Disable serving a file
    /// Path is relative to the dir being served
    /// ## Example
    /// ```rust,no_run
    /// // Import Library
    /// use afire::{Server, extension::ServeStatic, Middleware};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
    ///
    /// // Make a new static server
    /// ServeStatic::new("data/static")
    ///     // Disable a file from being served
    ///     .disable("index.scss")
    ///     // Attach it to the afire server
    ///     .attach(&mut server);
    ///
    /// server.run().unwrap();
    /// ```
    pub fn disable(self, file_path: impl AsRef<str>) -> Self {
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
    /// ```rust,no_run
    /// // Import Library
    /// use afire::{Server, extension::ServeStatic, Middleware};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
    ///
    /// // Make a new static server
    /// ServeStatic::new("data/static")
    ///     // Disable a vec of files from being served
    ///     .disable_vec(&["index.scss", "index.css.map"])
    ///     // Attach it to the afire server
    ///     .attach(&mut server);
    ///
    /// server.run().unwrap();
    /// ```
    pub fn disable_vec(self, file_paths: &[impl AsRef<str>]) -> Self {
        let mut disabled = self.disabled_files;
        for i in file_paths {
            disabled.push(i.as_ref().to_owned());
        }

        Self {
            disabled_files: disabled,
            ..self
        }
    }

    /// Set the not found page.
    ///
    /// This will run if no file is found to serve or the file is disabled.
    /// The bool in the fn parameters is if the file is blocked.
    /// ## Example
    /// ```rust,no_run
    /// // Import Library
    /// use afire::{Response, Server, extension::ServeStatic, Middleware, Status};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
    ///
    /// // Make a new static server
    /// ServeStatic::new("data/static")
    ///     // Set a new file not found page
    ///     .not_found(|_req, _dis| Response::new().status(Status::NotFound).text("Page Not Found!"))
    ///     // Attach it to the afire server
    ///     .attach(&mut server);
    ///
    /// server.run().unwrap();
    /// ```
    pub fn not_found(self, f: fn(Arc<Request>, bool) -> Response) -> Self {
        Self {
            not_found: f,
            ..self
        }
    }

    /// Add a MIME type to the Static file Server
    ///
    /// This extension comes with a lot of builtin MIME types
    /// but if you need to add more thats what this is for
    ///
    /// The key is the file extension
    ///
    /// The value is the MIME type
    /// ## Example
    /// ```rust,no_run
    /// // Import Library
    /// use afire::{Server, extension::ServeStatic, Middleware};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
    ///
    /// // Make a new static server
    /// ServeStatic::new("data/static")
    ///     // Add a new MIME type
    ///     .mime_type(".3gp", "video/3gpp")
    ///     // Attach it to the afire server
    ///     .attach(&mut server);
    ///
    /// server.run().unwrap();
    /// ```
    pub fn mime_type(self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
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
    /// ```rust,no_run
    /// // Import Library
    /// use afire::{Server, extension::ServeStatic, Middleware};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
    ///
    /// // Make a new static server
    /// ServeStatic::new("data/static")
    ///     // Add a new MIME type
    ///     .mime_types(&[("3gp", "video/3gpp")])
    ///     // Attach it to the afire server
    ///     .attach(&mut server);
    ///
    /// server.run().unwrap();
    /// ```
    pub fn mime_types(self, new_types: &[(impl AsRef<str>, impl AsRef<str>)]) -> Self {
        let mut new_types = new_types
            .iter()
            .map(|x| (x.0.as_ref().to_owned(), x.1.as_ref().to_owned()))
            .collect();
        let mut types = self.types;

        types.append(&mut new_types);

        Self { types, ..self }
    }

    /// Add a middleware to the serve static extension.
    /// Middleware here works much differently to the normal afire middleware.
    /// The middleware priority is still by most recently defined.
    ///
    /// The middleware function takes 3 parameters: the request, the response, and weather the file was loaded successfully.
    /// In your middleware you can modify the response and the bool.
    pub fn middleware(
        self,
        f: impl Fn(Arc<Request>, &mut Response, &mut bool) + Send + Sync + 'static,
    ) -> Self {
        let mut middleware = self.middleware;
        middleware.push(Box::new(f));

        Self { middleware, ..self }
    }

    /// Set path to serve static files on
    ///
    /// Default is '/' (root)
    /// ## Example
    /// ```rust,no_run
    /// // Import Library
    /// use afire::{Server, extension::ServeStatic, Middleware};
    ///
    /// // Create a server for localhost on port 8080
    /// let mut server = Server::<()>::new("localhost", 8080);
    ///
    /// // Make a new static server
    /// ServeStatic::new("data/static")
    ///     // Set serve path
    ///     .path("/static")
    ///     // Attach it to the afire server
    ///     .attach(&mut server);
    ///
    /// server.run().unwrap();
    /// ```
    pub fn path(self, path: impl AsRef<str>) -> Self {
        Self {
            serve_path: normalize_path(path.as_ref().to_owned()),
            ..self
        }
    }
}

fn process_req(req: Arc<Request>, this: &ServeStatic) -> (Response, bool) {
    let mut path = format!(
        "{}/{}",
        this.data_dir,
        safe_path(req.path.strip_prefix(&this.serve_path).unwrap())
    );

    // Add Index.html if path ends with /
    if path.ends_with('/') {
        path.push_str("index.html");
    }

    // Also add '/index.html' if path dose not end with a file
    if !path.rsplit('/').next().unwrap_or_default().contains('.') {
        path.push_str("/index.html");
    }

    if this
        .disabled_files
        .contains(&path.splitn(2, &this.data_dir).last().unwrap().to_string())
    {
        return ((this.not_found)(req, true), false);
    }

    // Try to read File
    let ext = path.rsplit('.').next().unwrap_or_default();
    let file = match File::open(&path) {
        Ok(i) => i,
        Err(_) => return ((this.not_found)(req, false), false),
    };

    let content_type = get_type(ext, &TYPES)
        .or_else(|| this.types.iter().find(|x| x.0 == ext).map(|x| x.1.as_str()))
        .unwrap_or("application/octet-stream");

    let mut res = Response::new();
    if let Ok(i) = file.metadata() {
        res.headers
            .add(HeaderType::ContentLength, i.len().to_string());
    }

    (res.stream(file).header("Content-Type", content_type), true)
}

/// Prevents path traversals.
/// Ex: '/hello/../../../data.db' => '/data.db'
#[inline]
pub fn safe_path(path: &str) -> Cow<'_, str> {
    if !path.contains("..") {
        return Cow::Borrowed(path);
    }

    let mut out = Vec::new();
    for i in path.split(['/', '\\']) {
        match i {
            ".." => {
                out.pop();
            }
            _ => out.push(i),
        }
    }

    Cow::Owned(out.join("/"))
}

/// Common MIME Types (sorted in alphabetical order by extension)
///
/// Used by ServeStatic extension
pub const TYPES: [MIME; 56] = [
    MIME::new("7z", "application/x-7z-compressed"),
    MIME::new("aac", "audio/aac"),
    MIME::new("avi", "video/x-msvideo"),
    MIME::new("bin", "application/octet-stream"),
    MIME::new("bmp", "image/bmp"),
    MIME::new("bz", "application/x-bzip"),
    MIME::new("bz2", "application/x-bzip2"),
    MIME::new("cda", "application/x-cdf"),
    MIME::new("css", "text/css"),
    MIME::new("csv", "text/csv"),
    MIME::new("epub", "application/epub+zip"),
    MIME::new("gif", "image/gif"),
    MIME::new("gz", "application/gzip"),
    MIME::new("htm", "text/html"),
    MIME::new("html", "text/html"),
    MIME::new("ico", "image/x-icon"),
    MIME::new("ics", "text/calendar"),
    MIME::new("jar", "application/java-archive"),
    MIME::new("jpeg", "image/jpeg"),
    MIME::new("jpg", "image/jpeg"),
    MIME::new("js", "application/javascript"),
    MIME::new("json", "application/json"),
    MIME::new("jsonld", "application/ld+json"),
    MIME::new("mid", "audio/midi audio/x-midi"),
    MIME::new("midi", "audio/midi audio/x-midi"),
    MIME::new("mjs", "text/javascript"),
    MIME::new("mp3", "audio/mpeg"),
    MIME::new("mp4", "video/mp4"),
    MIME::new("mpeg", "video/mpeg"),
    MIME::new("oga", "audio/ogg"),
    MIME::new("ogv", "video/ogg"),
    MIME::new("ogx", "application/ogg"),
    MIME::new("opus", "audio/opus"),
    MIME::new("otf", "font/otf"),
    MIME::new("pdf", "application/pdf"),
    MIME::new("png", "image/png"),
    MIME::new("rar", "application/vnd.rar"),
    MIME::new("rtf", "application/rtf"),
    MIME::new("sh", "application/x-sh"),
    MIME::new("svg", "image/svg+xml"),
    MIME::new("swf", "application/x-shockwave-flash"),
    MIME::new("tar", "application/x-tar"),
    MIME::new("tif", "image/tiff"),
    MIME::new("tiff", "image/tiff"),
    MIME::new("ts", "text/x-typescript"),
    MIME::new("ttf", "font/ttf"),
    MIME::new("txt", "text/plain"),
    MIME::new("wav", "audio/wav"),
    MIME::new("weba", "audio/webm"),
    MIME::new("webm", "video/webm"),
    MIME::new("webp", "image/webp"),
    MIME::new("woff", "font/woff"),
    MIME::new("woff2", "font/woff2"),
    MIME::new("xhtml", "application/xhtml+xml"),
    MIME::new("xml", "application/xml"),
    MIME::new("zip", "application/zip"),
];

/// Struct to hold a file extension and its matching MIME type
#[derive(Debug, Clone)]
pub struct MIME {
    extension: &'static str,
    mime_type: &'static str,
}

impl MIME {
    /// Create a new MIME type
    pub const fn new(extension: &'static str, mime_type: &'static str) -> Self {
        Self {
            extension,
            mime_type,
        }
    }
}

/// Gets the MIME type from the specified file extension using a slice of MIME types.
/// If no type is found, None is returned.
pub fn get_type(ext: &str, extensions: &[MIME]) -> Option<&'static str> {
    extensions
        .binary_search_by(|x| x.extension.cmp(ext))
        .map(|x| TYPES[x].mime_type)
        .ok()
}
