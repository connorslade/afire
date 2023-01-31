use std::{io::Write, net::TcpStream, panic};

use crate::{
    error::{HandleError, ParseError, Result},
    internal::common::any_string,
    route::RouteType,
    Content, Error, Method, Request, Response, Server,
};

pub(crate) fn handle<State>(stream: &mut TcpStream, this: &Server<State>)
where
    State: 'static + Send + Sync,
{
    trace!("Opening socket {}", stream.peer_addr().unwrap());
    loop {
        let mut keep_alive = false;
        let res = match Request::from_socket(stream).and_then(|req| {
            keep_alive = req.keep_alive();
            trace!("{} {} {}", req.method, req.path, keep_alive);
            handle_route(req, this)
        }) {
            Ok(req) => req,
            Err(Error::Stream(_)) => break,
            Err(e) => error_response(Error::None, e, this),
        };

        let close = res.close;
        let res = res.to_bytes(&this.default_headers);

        if let Err(e) = stream.write_all(&res) {
            trace!("Error writing to stream: {}", e);
            break;
        }

        if !keep_alive || close {
            trace!("Closeing socket");
            break;
        }
    }
}

fn handle_route<State>(mut req: Request, this: &Server<State>) -> Result<Response>
where
    State: 'static + Send + Sync,
{
    trace!("{:?}", req);
    // let req = Arc::new(req);
    for route in this.routes.iter().rev() {
        let path_match = route.path.match_path(req.path.clone());
        if (req.method == route.method || route.method == Method::ANY) && path_match.is_some() {
            req.path_params = path_match.unwrap_or_default();

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
                Box::new(Ok(req)),
                err,
            ))));
        }
    }

    Err(Error::Handle(Box::new(HandleError::NotFound(
        req.method, req.path,
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
        Error::Stream(_) => unreachable!(),
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
