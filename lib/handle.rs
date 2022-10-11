// Import STD libraries
use std::net::{SocketAddr, TcpStream};

// Feature Imports
#[cfg(not(feature = "panic_handler"))]
use crate::Middleware;
#[cfg(feature = "panic_handler")]
use std::panic;

// Import local files
use crate::{
    common::{any_string, has_header, reason_phrase, trim_end_bytes},
    content_type::Content,
    error::{Error, HandleError, ParseError, Result},
    header::{headers_to_string, Header},
    http,
    middleware::{MiddleRequest, MiddleResponse},
    route::RouteType,
    Method, Request, Response, Server,
};

/// Handle all aspects of a request
pub(crate) fn handle<State>(stream: &mut TcpStream, this: &Server<State>)
where
    State: 'static + Send + Sync,
{
    let (bytes, response, request) = handle_critical(stream, this);

    if response.close {
        return;
    }

    let _ = (this.socket_handler.socket_write)(stream, &bytes);
    let _ = (this.socket_handler.socket_flush)(stream);

    // Run end middleware
    for middleware in this.middleware.iter().rev() {
        #[cfg(feature = "panic_handler")]
        let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            middleware.end(&request, &response)
        }));

        #[cfg(not(feature = "panic_handler"))]
        middleware.end(&request, &response);
    }
}

fn handle_critical<State>(
    stream: &mut TcpStream,
    this: &Server<State>,
) -> (Vec<u8>, Response, Result<Request>)
where
    State: 'static + Send + Sync,
{
    let connection = match handle_connection(stream, this) {
        Ok(i) => i,
        Err(e) => {
            return {
                let out = response_http(this, Err(e));
                (out.0, out.1, Err(Error::None))
            }
        }
    };

    let req = Request::from_bytes(&connection.0, connection.1.to_string());
    let mut res = match req.clone() {
        Ok(i) => handle_request(this, i),
        Err(e) => Err(e),
    };

    for middleware in this.middleware.iter().rev() {
        #[cfg(feature = "panic_handler")]
        {
            let result =
                panic::catch_unwind(panic::AssertUnwindSafe(|| middleware.post(&req, &res)));

            match result {
                Ok(i) => match i {
                    MiddleResponse::Continue => {}
                    MiddleResponse::Add(i) => res = Ok(i),
                    MiddleResponse::Send(i) => {
                        res = Ok(i);
                        break;
                    }
                },
                Err(e) => {
                    res = Err(Error::Handle(Box::new(HandleError::Panic(
                        Box::new(req.clone()),
                        any_string(e),
                    ))))
                }
            }
        }

        #[cfg(not(feature = "panic_handler"))]
        {
            let result = middleware.post(req, res);
            match result {
                MiddleResponse::Continue => {}
                MiddleResponse::Add(i) => res = Ok(i),
                MiddleResponse::Send(i) => {
                    res = Ok(i);
                    break;
                }
            }
        }
    }

    let out = response_http(this, res);
    (out.0, out.1, req)
}

/// Handle a connection, outputting a byte vec
pub fn handle_connection<State>(
    stream: &mut TcpStream,
    this: &Server<State>,
) -> Result<(Vec<u8>, SocketAddr)>
where
    State: 'static + Send + Sync,
{
    // Init (first) Buffer
    let mut buffer = vec![0; this.buff_size];

    if this.socket_timeout.is_some() {
        stream.set_read_timeout(this.socket_timeout).unwrap();
        stream.set_write_timeout(this.socket_timeout).unwrap();
    }

    // Read stream into buffer
    match (this.socket_handler.socket_read)(stream, &mut buffer) {
        Ok(_) => {}
        Err(e) => return Err(Error::Io(e.kind())),
    }

    #[cfg(feature = "dynamic_resize")]
    {
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
            match (this.socket_handler.socket_read_exact)(stream, &mut new_buffer) {
                Ok(_) => {}
                Err(e) => return Err(Error::Io(e.kind())),
            }
            buffer.extend(new_buffer);
        };
    }

    // Remove trailing null bytes
    trim_end_bytes(&mut buffer);

    Ok((buffer, stream.peer_addr().unwrap()))
}

// Request::from_bytes(&data, addr.to_string())

pub fn handle_request<State>(server: &Server<State>, mut req: Request) -> Result<Response>
where
    State: 'static + Send + Sync,
{
    // Use middleware to handle request
    for middleware in server.middleware.iter().rev() {
        #[cfg(feature = "panic_handler")]
        {
            let result =
                panic::catch_unwind(panic::AssertUnwindSafe(|| middleware.pre(&Ok(req.clone()))));

            match result {
                Ok(i) => match i {
                    MiddleRequest::Continue => {}
                    MiddleRequest::Add(i) => req = i,
                    MiddleRequest::Send(i) => return Ok(i),
                },
                Err(e) => {
                    return Err(Error::Handle(Box::new(HandleError::Panic(
                        Box::new(Ok(req)),
                        any_string(e),
                    ))))
                }
            }
        }

        #[cfg(not(feature = "panic_handler"))]
        {
            let result = middleware.pre(&req);
            match result {
                MiddleRequest::Continue => {}
                MiddleRequest::Add(i) => req = i,
                MiddleRequest::Send(i) => return Ok(i),
            }
        }
    }

    // Loop through all routes and check if the request matches
    for route in server.routes.iter().rev() {
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
                    panic::catch_unwind(panic::AssertUnwindSafe(|| match &route.handler {
                        RouteType::Stateless(i) => (i)(req.clone()),
                        RouteType::Statefull(i) => (i)(
                            server.state.clone().expect("State not initialized"),
                            req.clone(),
                        ),
                    }));

                let err = match result {
                    Ok(i) => return Ok(i),
                    Err(e) => any_string(e),
                };

                return Err(Error::Handle(Box::new(HandleError::Panic(
                    Box::new(Ok(req)),
                    err,
                ))));
            }

            #[cfg(not(feature = "panic_handler"))]
            {
                match route.handler {
                    RouteType::Stateless(i) => return Ok((i)(req.clone())),
                    RouteType::Statefull(i) => {
                        return Ok((i)(server.state.expect("State not initialized"), req))
                    }
                }
            }
        }
    }

    Err(Error::Handle(Box::new(HandleError::NotFound(
        req.method, req.path,
    ))))
}

pub fn response_http<State>(this: &Server<State>, res: Result<Response>) -> (Vec<u8>, Response)
where
    State: 'static + Send + Sync,
{
    let res = match res {
        Ok(i) => i,
        Err(e) => error_response(e, Error::None, this),
    };

    // Add default headers to response
    // Only the ones that arent already in the response
    let mut headers = res.headers.to_vec();
    for i in &this.default_headers {
        if !has_header(&headers, &i.name) {
            headers.push(i.clone());
        }
    }

    // Add content-length header to response if it hasent already been deifned by the route or defult headers
    if !has_header(&headers, "Content-Length") {
        headers.push(Header::new("Content-Length", &res.data.len().to_string()));
    }

    // Convert the response to a string
    // TODO: Use Bytes instead of String
    let mut response = format!(
        "HTTP/1.1 {} {}\r\n{}\r\n\r\n",
        res.status,
        res.reason
            .to_owned()
            .unwrap_or_else(|| reason_phrase(res.status)),
        headers_to_string(headers)
    )
    .as_bytes()
    .to_vec();

    // Add Bytes of data to response
    response.extend(res.data.iter());

    (response, res)
}

pub fn error_response<State>(req: Error, mut res: Error, server: &Server<State>) -> Response
where
    State: 'static + Send + Sync,
{
    if res == Error::None {
        res = req;
    }

    match res {
        Error::Parse(e) => match e {
            ParseError::NoSeparator => Response::new().status(400).text("No separator"),
            ParseError::NoMethod => Response::new().status(400).text("No method"),
            ParseError::NoPath => Response::new().status(400).text("No path"),
            ParseError::NoVersion => Response::new().status(400).text("No HTTP version"),
            ParseError::NoRequestLine => Response::new().status(400).text("No request line"),
            ParseError::InvalidQuery => Response::new().status(400).text("Invalid query"),
            ParseError::InvalidHeader(i) => Response::new()
                .status(400)
                .text(format!("Invalid header #{}", i)),
        },
        Error::Handle(e) => match *e {
            HandleError::NotFound(method, path) => Response::new()
                .status(404)
                .text(format!(
                    "Cannot {} {}",
                    match method {
                        Method::CUSTOM(i) => i,
                        _ => method.to_string(),
                    },
                    path
                ))
                .content(Content::TXT),
            #[cfg(feature = "panic_handler")]
            HandleError::Panic(r, e) => (server.error_handler)(*r, e),
            #[cfg(not(feature = "panic_handler"))]
            HandleError::Panic(_, _) => unreachable!(),
        },
        Error::Io(e) => Response::new().status(500).text(format!("{:?}", e)),
        Error::None => unreachable!(),
    }
}
