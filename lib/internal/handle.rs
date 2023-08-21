use std::{
    cell::RefCell,
    io::Read,
    net::{Shutdown, TcpStream},
    ops::Deref,
    sync::Arc,
};

use crate::{
    context::ContextFlag,
    error::{HandleError, ParseError, StreamError},
    internal::sync::ForceLockMutex,
    prelude::MiddleResult,
    response::ResponseFlag,
    route::RouteError,
    socket::Socket,
    trace, Content, Context, Error, Request, Response, Server, Status,
};

use super::sync::ForceLockRwLock;

pub(crate) type Writeable = Box<RefCell<dyn Read + Send>>;

// https://open.spotify.com/track/50txng2W8C9SycOXKIQP0D

pub(crate) fn handle<State>(stream: TcpStream, this: Arc<Server<State>>)
where
    State: 'static + Send + Sync,
{
    trace!(Level::Debug, "Opening socket {:?}", stream.peer_addr());
    stream.set_read_timeout(this.socket_timeout).unwrap();
    stream.set_write_timeout(this.socket_timeout).unwrap();
    let stream = Arc::new(Socket::new(stream));
    'outer: loop {
        let mut keep_alive = false;
        let mut req = Request::from_socket(stream.clone());

        for i in &this.middleware {
            match i.pre_raw(&mut req) {
                MiddleResult::Abort => break,
                MiddleResult::Continue => (),
                MiddleResult::Send(res) => {
                    write(stream.clone(), this.clone(), req.map(Arc::new), Ok(res));
                    if close(stream.clone(), keep_alive, this.clone()) {
                        break 'outer;
                    }
                    continue 'outer;
                }
            }
        }

        if let Ok(req) = &req {
            keep_alive = req.keep_alive() && this.keep_alive;
            trace!(
                Level::Debug,
                "{} {} {{ keep_alive: {} }}",
                req.method,
                req.path,
                keep_alive
            );
        }

        let req = match req.map(Arc::new) {
            Ok(req) => req,
            Err(e) => {
                write(stream.clone(), this.clone(), Err(e), Err(Error::None));
                if close(stream.clone(), keep_alive, this.clone()) {
                    break 'outer;
                }
                continue 'outer;
            }
        };

        let mut ctx = Context::new(this.clone(), req.clone());

        // Handle Route
        let (route, params) = match this
            .routes
            .iter()
            .rev()
            .find_map(|route| route.matches(req.clone()).map(|x| (route, x)))
        {
            Some(x) => x,
            None => {
                let err = HandleError::NotFound(req.method, req.path.to_owned()).into();
                write(stream.clone(), this.clone(), Ok(req), Err(err));
                continue 'outer;
            }
        };

        ctx.path_params = params;
        let result = (route.handler)(&ctx);
        let sent_response = ctx.flags.get(ContextFlag::ResponseSent);

        if let Err(e) = result {
            // TODO: account for guaranteed send
            // TODO: Run through `write` for middleware
            let error =
                RouteError::downcast_error(&e).unwrap_or_else(|| RouteError::from_error(&e));
            if let Err(e) = error
                .as_response()
                .write(req.socket.clone(), &this.default_headers)
            {
                trace!(Level::Debug, "Error writing error response: {:?}", e);
            }
        } else if sent_response {
        } else if ctx.flags.get(ContextFlag::GuaranteedSend) {
            let barrier = ctx.req.socket.barrier.clone();
            trace!(Level::Debug, "Waiting for response to be sent");
            barrier.wait();
            trace!(Level::Debug, "Response sent");
        } else {
            // TODO: Impl NotImplemented as an error
            let res = Response::new()
                .status(Status::NotImplemented)
                .text("No response was sent");
            write(stream.clone(), this.clone(), Ok(req), Ok(res));
        }

        if close(stream.clone(), keep_alive, this.clone()) {
            break 'outer;
        }
    }
}

fn write<State: 'static + Send + Sync>(
    socket: Arc<Socket>,
    server: Arc<Server<State>>,
    request: Result<Arc<Request>, Error>,
    mut response: Result<Response, Error>,
) {
    for i in &server.middleware {
        // TODO: Remove clone
        match i.post_raw(request.clone(), &mut response) {
            MiddleResult::Abort => break,
            MiddleResult::Continue => (),
            MiddleResult::Send(res) => {
                response = Ok(res);
                break;
            }
        }
    }

    let mut response = match response {
        Ok(i) => i,
        Err(e) => {
            if let Err(r) = request {
                error_response(&r, server.clone())
            } else {
                error_response(&e, server.clone())
            }
        }
    };

    socket.set_flag(response.flag);
    if let Err(e) = response.write(socket.clone(), &server.default_headers) {
        trace!(Level::Debug, "Error writing response: {:?}", e);
        socket.set_flag(ResponseFlag::End);
    }
}

fn close<State: 'static + Send + Sync>(
    stream: Arc<Socket>,
    keep_alive: bool,
    this: Arc<Server<State>>,
) -> bool {
    let flag = stream.flag();
    if flag == ResponseFlag::End {
        trace!(Level::Debug, "Ending socket");
        return true;
    }

    if !keep_alive || flag == ResponseFlag::Close || !this.keep_alive {
        trace!(Level::Debug, "Closing socket");
        if let Err(e) = stream.force_lock().shutdown(Shutdown::Both) {
            trace!(Level::Debug, "Error closing socket: {:?}", e);
        }
        return true;
    }

    false
}

/// Gets a response if there is an error.
/// Can handle Parse, Handle and IO errors.
pub fn error_response<State>(err: &Error, server: Arc<Server<State>>) -> Response
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
        Error::Parse(ParseError::InvalidHttpVersion) => Response::new()
            .status(Status::HTTPVersionNotSupported)
            .text("HTTP version not supported. Only HTTP/1.0 and HTTP/1.1 are supported."),
        Error::Parse(e) => Response::new().status(400).text(match e {
            ParseError::NoSeparator => "No separator",
            ParseError::NoMethod => "No method",
            ParseError::NoPath => "No path",
            ParseError::NoVersion => "No HTTP version",
            ParseError::NoRequestLine => "No request line",
            ParseError::InvalidQuery => "Invalid query",
            ParseError::InvalidHeader => "Invalid header",
            ParseError::InvalidMethod => "Invalid method",
            ParseError::NoHostHeader => "No Host header",
            ParseError::InvalidHttpVersion => unreachable!(),
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
