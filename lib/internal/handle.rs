use std::{
    cell::RefCell,
    io::Read,
    net::{Shutdown, TcpStream},
    ops::Deref,
    panic,
    rc::Rc,
    sync::Mutex,
};

use crate::{
    error::{HandleError, ParseError, Result, StreamError},
    internal::common::any_string,
    middleware::MiddleResult,
    route::RouteType,
    trace, Content, Error, Request, Response, Server, Status,
};

pub(crate) type Writeable = Box<RefCell<dyn Read + Send>>;

/// - Manages keep-alive sockets
/// - Lets Request::from_socket read the request
/// - Lets Response::write write the response to the socket
/// - Runs End Middleware
/// - Optionally closes the socket
pub(crate) fn handle<State>(stream: TcpStream, this: &Server<State>)
where
    State: 'static + Send + Sync,
{
    trace!(Level::Debug, "Opening socket {:?}", stream.peer_addr());
    stream.set_read_timeout(this.socket_timeout).unwrap();
    stream.set_write_timeout(this.socket_timeout).unwrap();
    let stream = Rc::new(Mutex::new(stream));
    loop {
        let mut keep_alive = false;
        let req = Request::from_socket(stream.clone());

        if let Ok(req) = &req {
            keep_alive = req.keep_alive();
            trace!(Level::Debug, "{} {} {}", req.method, req.path, keep_alive);
        }

        let (req, mut res) = get_response(req, this);

        let close = res.close;
        if let Err(e) = res.write(stream.clone(), &this.default_headers) {
            trace!(Level::Error, "Error writing to socket: {:?}", e);
        }

        // End Middleware
        if let Some(req) = req {
            for i in this.middleware.iter().rev() {
                if let Err(e) = panic::catch_unwind(panic::AssertUnwindSafe(|| i.end(&req, &res))) {
                    trace!(Level::Error, "Error running end middleware: {:?}", e);
                }
            }
        }

        if !keep_alive || close || !this.keep_alive {
            trace!(Level::Debug, "Closing socket");
            if let Err(e) = stream.lock().unwrap().shutdown(Shutdown::Both) {
                trace!(Level::Error, "Error closing socket: {:?}", e);
            }
            break;
        }
    }
}

/// Gets the response from a request.
/// Will call middleware, route handlers and error handlers if needed.
fn get_response<State>(
    mut req: Result<Request>,
    server: &Server<State>,
) -> (Option<Rc<Request>>, Response)
where
    State: 'static + Send + Sync,
{
    let mut res = Err(Error::None);
    let handle_error = |error, req: Result<_>, server| {
        let err = HandleError::Panic(Box::new(req.clone()), any_string(error).into_owned()).into();
        (req.ok(), error_response(&err, server))
    };

    // Pre Middleware
    for i in server.middleware.iter().rev() {
        match panic::catch_unwind(panic::AssertUnwindSafe(|| i.pre_raw(&mut req))) {
            Ok(MiddleResult::Send(this_res)) => {
                res = Ok(this_res);
                break;
            }
            Ok(MiddleResult::Abort) => break,
            Ok(MiddleResult::Continue) => {}
            Err(e) => return handle_error(e, req.map(Rc::new), server),
        }
    }

    let req = req.map(Rc::new);
    if res.is_err() {
        if let Ok(req) = req.clone() {
            res = handle_route(req, server);
        }
    }

    // Post Middleware
    for i in server.middleware.iter().rev() {
        match panic::catch_unwind(panic::AssertUnwindSafe(|| {
            i.post_raw(req.clone(), &mut res)
        })) {
            Ok(MiddleResult::Send(res)) => return (req.ok(), res),
            Ok(MiddleResult::Abort) => break,
            Ok(MiddleResult::Continue) => {}
            Err(e) => return handle_error(e, req, server),
        }
    }

    let res = match res {
        Ok(res) => res,
        Err(e) => {
            let error = match req {
                Err(ref err) => err,
                Ok(_) => &e,
            };

            return (None, error_response(error, server));
        }
    };

    (req.ok(), res)
}

/// Tries to find a route that matches the request.
/// If it finds one, it will call the handler and return the result (assuming it doesn't panic).
/// If it doesn't find one, it will return an Error of HandleError::NotFound.
fn handle_route<State>(req: Rc<Request>, this: &Server<State>) -> Result<Response>
where
    State: 'static + Send + Sync,
{
    // Handle Route
    let path = req.path.to_owned();
    for route in this.routes.iter().rev() {
        if let Some(params) = route.matches(req.clone()) {
            *req.path_params.borrow_mut() = params;
            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| match &route.handler {
                RouteType::Stateless(i) => (i)(&req),
                RouteType::Stateful(i) => {
                    (i)(this.state.clone().expect("State not initialized"), &req)
                }
            }));

            let err = match result {
                Ok(i) => return Ok(i),
                Err(e) => any_string(e),
            };

            return Err(Error::Handle(Box::new(HandleError::Panic(
                Box::new(Ok(req)),
                err.into_owned(),
            ))));
        }
    }

    Err(Error::Handle(Box::new(HandleError::NotFound(
        req.method, path,
    ))))
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
                .text(format!("Cannot {} {}", method, path))
                .content(Content::TXT),
            HandleError::Panic(r, e) => {
                (server.error_handler)(server.state.clone(), r, e.to_owned())
            }
        },
        Error::Io(e) => Response::new().status(500).text(e),
    }
}
