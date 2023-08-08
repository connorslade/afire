use std::{
    borrow::Cow,
    cell::RefCell,
    io::Read,
    net::{Shutdown, TcpStream},
    ops::Deref,
    panic,
    rc::Rc,
    sync::{atomic::Ordering, Arc, Mutex},
};

use crate::{
    error::{HandleError, ParseError, Result, StreamError},
    internal::common::{any_string, ForceLockMutex},
    middleware::MiddleResult,
    response::ResponseFlag,
    server, trace, Content, Context, Error, Request, Response, Server, Status,
};

pub(crate) type Writeable = Box<RefCell<dyn Read + Send>>;

// https://open.spotify.com/track/50txng2W8C9SycOXKIQP0D

/// - Manages keep-alive sockets
/// - Lets Request::from_socket read the request
/// - Lets Response::write write the response to the socket
/// - Runs End Middleware
/// - Optionally closes the socket
pub(crate) fn handle<State>(stream: TcpStream, this: Arc<Server<State>>)
where
    State: 'static + Send + Sync,
{
    trace!(Level::Debug, "Opening socket {:?}", stream.peer_addr());
    stream.set_read_timeout(this.socket_timeout).unwrap();
    stream.set_write_timeout(this.socket_timeout).unwrap();
    let stream = Arc::new(Mutex::new(stream));
    loop {
        let mut keep_alive = false;
        let mut req = Request::from_socket(stream.clone());

        if let Ok(req) = &req {
            keep_alive = req.keep_alive();
            trace!(
                Level::Debug,
                "{} {} {{ keep_alive: {} }}",
                req.method,
                req.path,
                keep_alive
            );
        }

        let req = req.map(Rc::new).expect("Error with getting request");

        // Handle Route
        let ctx = Context::new(this.clone(), req.clone());
        let mut flag = ResponseFlag::None;
        for route in this.routes.iter().rev() {
            if let Some(params) = route.matches(req.clone()) {
                *req.path_params.borrow_mut() = params;
                let result = (route.handler)(&ctx);

                match result {
                    Ok(_) => {
                        flag = ctx.response.force_lock().flag;
                        let has_response = ctx.response_dirty.load(Ordering::Relaxed);
                        if !has_response {
                            unimplemented!("No response");
                        }

                        break;
                    }
                    Err(_) => unimplemented!("Route error"),
                };
            }
        }

        // unimplemented!("Route not found")

        if flag == ResponseFlag::End {
            trace!(Level::Debug, "Ending socket");
            break;
        }

        if !keep_alive || flag == ResponseFlag::Close || !this.keep_alive {
            trace!(Level::Debug, "Closing socket");
            if let Err(e) = stream.lock().unwrap().shutdown(Shutdown::Both) {
                trace!(Level::Debug, "Error closing socket: {:?}", e);
            }
            break;
        }
    }
}

/// Gets a response if there is an error.
/// Can handle Parse, Handle and IO errors.
pub fn error_response<State>(err: &Error, server: &Server<State>) -> Response
where
    State: 'static + Send + Sync,
{
    match err {
        Error::None | Error::Startup(_) => {
            unreachable!("None and Startup errors should not be here")
        }
        Error::Stream(e) => match e {
            StreamError::UnexpectedEof => Response::new().status(400).text("Unexpected EOF"),
        },
        Error::Parse(e) => Response::new().status(400).text(match e {
            ParseError::NoSeparator => "No separator",
            ParseError::NoMethod => "No method",
            ParseError::NoPath => "No path",
            ParseError::NoVersion => "No HTTP version",
            ParseError::NoRequestLine => "No request line",
            ParseError::InvalidQuery => "Invalid query",
            ParseError::InvalidHeader => "Invalid header",
            ParseError::InvalidMethod => "Invalid method",
        }),
        Error::Handle(e) => match e.deref() {
            HandleError::NotFound(method, path) => Response::new()
                .status(Status::NotFound)
                .text(format!("Cannot {method} {path}"))
                .content(Content::TXT),
            HandleError::Panic(r, e) => {
                (server.error_handler)(server.state.clone(), r, e.to_owned())
            }
        },
        Error::Io(e) => Response::new().status(500).text(e),
    }
}
