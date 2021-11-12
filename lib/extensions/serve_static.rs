use crate::Header;
use crate::Request;
use crate::Response;

pub struct ServeStatic {
    pub data_dir: String,
    pub disabled_files: Vec<String>,
    pub not_found: fn(Request, String) -> Response,
    pub middleware: Vec<fn(&Request, Vec<u8>) -> Option<Response>>,
}

impl ServeStatic {
    pub fn new<T>(path: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self {
            data_dir: path.to_string(),
            disabled_files: Vec::new(),
            not_found: |req, _| {
                Response::new()
                    .text(format!("The page `{}` was not found...", req.path))
                    .header(Header::new("Content-Type", "text/html"))
            },
            middleware: Vec::new(),
        }
    }

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
}
