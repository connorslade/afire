use std::{
    cell::RefCell,
    io::Read,
    net::{Shutdown, TcpStream},
    panic,
    sync::Arc,
};

use crate::{
    error::{HandleError, ParseError, Result},
    internal::common::any_string,
    middleware::MiddleResult,
    route::RouteType,
    trace, Content, Error, Method, Request, Response, Server,
};

pub(crate) type Writeable = Box<RefCell<dyn Read + Send>>;

pub(crate) fn handle<State>(stream: &mut TcpStream, this: &Server<State>)
where
    State: 'static + Send + Sync,
{
    trace!(
        Level::Debug,
        "Opening socket {}",
        stream.peer_addr().unwrap()
    );
    loop {
        let mut keep_alive = false;
        let req = Request::from_socket(stream);

        if let Ok(req) = &req {
            keep_alive = req.keep_alive();
            trace!(Level::Debug, "{} {} {}", req.method, req.path, keep_alive);
        }

        let (req, mut res) = get_response(req, this);

        let close = res.close;
        if let Err(e) = res.write(stream, &this.default_headers) {
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

        if !keep_alive || close {
            trace!(Level::Debug, "Closing socket");
            if let Err(e) = stream.shutdown(Shutdown::Both) {
                trace!(Level::Error, "Error closing socket: {:?}", e);
            }
            break;
        }
    }
}

fn get_response<State>(
    req: Result<Request>,
    server: &Server<State>,
) -> (Option<Arc<Request>>, Response)
where
    State: 'static + Send + Sync,
{
    let mut req = match req {
        Ok(req) => req,
        Err(e) => return (None, error_response(Error::None, e, server)),
    };

    // Pre Middleware
    // TODO: dont use an Arc here
    for i in server.middleware.iter().rev() {
        match panic::catch_unwind(panic::AssertUnwindSafe(|| i.pre(&mut req))) {
            Ok(MiddleResult::Abort(res)) => return (Some(Arc::new(req)), res),
            Ok(MiddleResult::Continue) => {}
            Err(e) => {
                let req = Arc::new(req);
                return (
                    Some(req.clone()),
                    error_response(
                        Error::None,
                        HandleError::Panic(Box::new(req), any_string(e)).into(),
                        server,
                    ),
                );
            }
        }
    }

    let req = Arc::new(req);
    let mut res = match handle_route(req.clone(), server) {
        Ok(res) => res,
        Err(e) => return (Some(req), error_response(Error::None, e, server)),
    };

    // Post Middleware
    for i in server.middleware.iter().rev() {
        match panic::catch_unwind(panic::AssertUnwindSafe(|| i.post(&req, &mut res))) {
            Ok(MiddleResult::Abort(res)) => return (Some(req), res),
            Ok(MiddleResult::Continue) => {}
            Err(e) => {
                return (
                    Some(req.clone()),
                    error_response(
                        Error::None,
                        HandleError::Panic(Box::new(req), any_string(e)).into(),
                        server,
                    ),
                )
            }
        }
    }

    (Some(req), res)
}

fn handle_route<State>(req: Arc<Request>, this: &Server<State>) -> Result<Response>
where
    State: 'static + Send + Sync,
{
    // Handle Route
    let path = req.path.to_owned();
    for route in this.routes.iter().rev() {
        let path_match = route.path.match_path(req.path.clone());
        if (req.method == route.method || route.method == Method::ANY) && path_match.is_some() {
            *req.path_params.borrow_mut() = path_match.unwrap_or_default();

            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| match &route.handler {
                RouteType::Stateless(i) => (i)(&req),
                RouteType::Statefull(i) => {
                    (i)(this.state.clone().expect("State not initialized"), &req)
                }
            }));

            let err = match result {
                Ok(i) => return Ok(i),
                Err(e) => any_string(e),
            };

            return Err(Error::Handle(Box::new(HandleError::Panic(
                Box::new(req),
                err,
            ))));
        }
    }

    Err(Error::Handle(Box::new(HandleError::NotFound(
        req.method, path,
    ))))
}

pub fn error_response<State>(req: Error, mut res: Error, server: &Server<State>) -> Response
where
    State: 'static + Send + Sync,
{
    if matches!(res, Error::None) {
        res = req;
    }

    match res {
        Error::Stream(_) | Error::Startup(_) => {
            unreachable!("Stream and Startup errors should not be here")
        }
        Error::Parse(e) => match e {
            ParseError::NoSeparator => Response::new().status(400).text("No separator"),
            ParseError::NoMethod => Response::new().status(400).text("No method"),
            ParseError::NoPath => Response::new().status(400).text("No path"),
            ParseError::NoVersion => Response::new().status(400).text("No HTTP version"),
            ParseError::NoRequestLine => Response::new().status(400).text("No request line"),
            ParseError::InvalidQuery => Response::new().status(400).text("Invalid query"),
            ParseError::InvalidHeader => Response::new().status(400).text("Invalid header"),
            ParseError::InvalidMethod => Response::new().status(400).text("Invalid method"),
        },
        Error::Handle(e) => match *e {
            HandleError::NotFound(method, path) => Response::new()
                .status(404)
                .text(format!("Cannot {} {}", method, path))
                .content(Content::TXT),
            #[cfg(feature = "panic_handler")]
            HandleError::Panic(r, e) => (server.error_handler)(*r, e),
            #[cfg(not(feature = "panic_handler"))]
            HandleError::Panic(_, _) => unreachable!(),
        },
        Error::Io(e) => Response::new().status(500).text(e),
        Error::None => unreachable!(),
    }
}
