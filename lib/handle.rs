// Import STD libraries
use std::cell::RefCell;
use std::io::Read;
use std::net::TcpStream;
use std::str;

// Feature Imports
#[cfg(feature = "panic_handler")]
use std::panic;

// Import local files
use crate::common::{any_string, trim_end_bytes};
use crate::content_type::Content;
use crate::header::Header;
use crate::http;
use crate::method::Method;
use crate::middleware::{MiddleRequest, Middleware};
use crate::request::Request;
use crate::response::Response;
use crate::route::Route;
use crate::server::ErrorHandler;

/// Handle a request
pub(crate) fn handle_connection(
    mut stream: &TcpStream,
    middleware: &[Box<RefCell<dyn Middleware>>],
    #[cfg(feature = "panic_handler")] error_handler: &ErrorHandler,
    routes: &[Route],
    buff_size: usize,
) -> (Request, Response) {
    // Init (first) Buffer
    let mut buffer = vec![0; buff_size];

    // Read stream into buffer
    match stream.read(&mut buffer) {
        Ok(_) => {}
        Err(_) => return quick_err("Error Reading Stream", 500),
    };

    #[cfg(feature = "dynamic_resize")]
    {
        // TODO: Make this not crash

        // Get Buffer as string for parseing content length header
        let stream_string = String::from_utf8_lossy(&buffer);

        // Get Content-Length header
        // If header shows thar more space is needed,
        // make a new buffer read the rest of the stream and add it to the first buffer
        if let Some(dyn_buf) = http::get_request_headers(&stream_string)
            .iter()
            .find(|x| x.name == "Content-Length")
        {
            let header_size = http::get_header_size(&stream_string);
            let content_length = dyn_buf.value.parse::<usize>().unwrap_or(0);
            let new_buffer_size = (content_length as i64 + header_size as i64) as usize;

            if new_buffer_size > buff_size {
                buffer.reserve(content_length + header_size);
            }

            trim_end_bytes(&mut buffer);
            let mut new_buffer = vec![0; new_buffer_size - buffer.len()];
            stream.read_exact(&mut new_buffer).unwrap();
            buffer.extend(new_buffer);
        };
    }

    // TODO: Parse Bytes
    // TODO: Have one mut HTTP string that is chipted away at theough parseing

    // Get Buffer as string for parseing Path, Method, Query, etc
    let stream_string = String::from_utf8_lossy(&buffer);

    // Make Request Object
    let req_method = http::get_request_method(&stream_string);
    let req_path = http::get_request_path(&stream_string);
    let req_query = http::get_request_query(&stream_string);
    let body = http::get_request_body(&buffer);
    let headers = http::get_request_headers(&stream_string);
    #[cfg(feature = "cookies")]
    let cookies = http::get_request_cookies(&stream_string);

    let mut req = Request {
        method: req_method,
        path: req_path,
        query: req_query,
        headers,
        #[cfg(feature = "cookies")]
        cookies,
        body,
        address: stream.peer_addr().unwrap().to_string(),
        raw_data: buffer,
        #[cfg(feature = "path_patterns")]
        path_params: Vec::new(),
    };

    // Use middleware to handle request
    // If middleware returns a `None`, the request will be handled by earlier middleware then the routes
    for middleware in middleware.iter().rev() {
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            middleware.borrow_mut().pre(req.clone())
        }));

        match result {
            Ok(i) => match i {
                MiddleRequest::Continue => {}
                MiddleRequest::Add(i) => req = i,
                MiddleRequest::Send(i) => return (req, i),
            },
            Err(e) => return (req.clone(), (error_handler)(req.clone(), any_string(e))),
        }
    }

    // Loop through all routes and check if the request matches
    for route in routes.iter().rev() {
        let path_match = route.path.match_path(req.path.clone());
        if (req.method == route.method || route.method == Method::ANY) && path_match.is_some() {
            // Set the Pattern params of the Request
            #[cfg(feature = "path_patterns")]
            {
                req.path_params = path_match.unwrap_or_default();
            }

            // Optionally enable automatic panic handling
            #[cfg(feature = "panic_handler")]
            {
                return error_handle(&route.handler, error_handler, req);
            }

            #[cfg(not(feature = "panic_handler"))]
            {
                return (route.handler)(req);
            }
        }
    }

    // If no route was found, return a default 404
    (
        req.clone(),
        Response::new()
            .status(404)
            .text(format!("Cannot {} {}", req.method, req.path))
            .header(Header::new("Content-Type", "text/plain")),
    )
}

pub(crate) fn error_handle<T, M>(fun: M, handle: T, req: Request) -> (Request, Response)
where
    T: Fn(Request, String) -> Response,
    M: Fn(Request) -> Response,
{
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| (fun)(req.clone())));
    let err = match result {
        Ok(i) => return (req, i),
        Err(e) => match e.downcast_ref::<&str>() {
            Some(err) => err,
            None => "",
        },
    };

    (req.clone(), (handle)(req.clone(), err.to_owned()))
}

/// Quick function to get a basic error response
fn quick_err(text: &str, code: u16) -> (Request, Response) {
    (
        Request::new_empty(),
        Response::new()
            .status(code)
            .text(text)
            .content(Content::TXT),
    )
}
