// Import STD libraries
use std::io::Read;
use std::net::TcpStream;

// Feature Imports
#[cfg(feature = "panic_handler")]
use std::panic;

// Import local files
use crate::common::{any_string, reason_phrase, trim_end_bytes};
use crate::content_type::Content;
use crate::header::{headers_to_string, Header};
use crate::http;
use crate::method::Method;
use crate::middleware::{MiddleRequest, MiddleResponse};
use crate::request::Request;
use crate::response::Response;
use crate::server::Server;

/// Handle a request
pub(crate) fn handle_connection(mut stream: &TcpStream, this: &Server) -> (Request, Response) {
    // Init (first) Buffer
    let mut buffer = vec![0; this.buff_size];

    if this.socket_timeout.is_some() {
        stream.set_read_timeout(this.socket_timeout).unwrap();
        stream.set_write_timeout(this.socket_timeout).unwrap();
    }

    // Read stream into buffer
    match stream.read(&mut buffer) {
        Ok(_) => {}
        Err(_) => {
            return (
                Request::new_empty(),
                Response::new()
                    .status(500)
                    .text("Error Reading Stream")
                    .content(Content::TXT),
            )
        }
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

            if new_buffer_size > this.buff_size {
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
    for middleware in this.middleware.iter().rev() {
        #[cfg(feature = "panic_handler")]
        {
            let result =
                panic::catch_unwind(panic::AssertUnwindSafe(|| middleware.pre(req.clone())));

            match result {
                Ok(i) => match i {
                    MiddleRequest::Continue => {}
                    MiddleRequest::Add(i) => req = i,
                    MiddleRequest::Send(i) => return (req, i),
                },
                Err(e) => {
                    return (
                        req.clone(),
                        (this.error_handler)(req.clone(), any_string(e)),
                    )
                }
            }
        }

        #[cfg(not(feature = "panic_handler"))]
        {
            let result = this.middleware.borrow_mut().pre(req.clone());
            match result {
                MiddleRequest::Continue => {}
                MiddleRequest::Add(i) => req = i,
                MiddleRequest::Send(i) => return (req, i),
            }
        }
    }

    // Loop through all routes and check if the request matches
    for route in this.routes.iter().rev() {
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
                let result =
                    panic::catch_unwind(panic::AssertUnwindSafe(|| (&route.handler)(req.clone())));
                let err = match result {
                    Ok(i) => return (req, i),
                    Err(e) => any_string(e),
                };

                return (req.clone(), (this.error_handler)(req.clone(), err));
            }

            #[cfg(not(feature = "panic_handler"))]
            {
                return (req.clone(), (route.handler)(req));
            }
        }
    }

    // If no route was found, return a default 404
    (
        req.clone(),
        Response::new()
            .status(404)
            .text(format!("Cannot {} {}", req.method, req.path))
            .header("Content-Type", "text/plain"),
    )
}

pub(crate) fn response_http(this: &Server, req: Request, mut res: Response) -> Vec<u8> {
    for middleware in this.middleware.iter().rev() {
        #[cfg(feature = "panic_handler")]
        {
            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                middleware.post(req.clone(), res.clone())
            }));

            match result {
                Ok(i) => match i {
                    MiddleResponse::Continue => {}
                    MiddleResponse::Add(i) => res = i,
                    MiddleResponse::Send(i) => {
                        res = i;
                        break;
                    }
                },
                Err(e) => res = (this.error_handler)(req.clone(), any_string(e)),
            }
        }

        #[cfg(not(feature = "panic_handler"))]
        {
            let result = middleware.post(req.clone(), res.clone());
            match result {
                MiddleResponse::Continue => {}
                MiddleResponse::Add(i) => res = i,
                MiddleResponse::Send(i) => {
                    res = i;
                    break;
                }
            }
        }
    }
    // Add default headers to response
    let mut headers = res.headers;
    headers.append(&mut this.default_headers.clone());

    // Add content-length header to response
    headers.push(Header::new("Content-Length", &res.data.len().to_string()));

    // Convert the response to a string
    // TODO: Use Bytes instead of String
    let status = res.status;
    let mut response = format!(
        "HTTP/1.1 {} {}\r\n{}\r\n\r\n",
        status,
        res.reason.unwrap_or_else(|| reason_phrase(status)),
        headers_to_string(headers)
    )
    .as_bytes()
    .to_vec();

    // Add Bytes of data to response
    response.append(&mut res.data);

    response
}
