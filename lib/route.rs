use std::{
    any,
    error::Error,
    fmt::{self, Debug, Display},
    panic,
    sync::Arc,
};

use crate::{
    error::AnyResult, path::Path, Content, Context, Header, HeaderType, Method, Request, Response,
    Status,
};

type Handler<State> = Box<dyn Fn(&Context<State>) -> AnyResult<()> + 'static + Send + Sync>;

/// Defines a route.
///
/// You should not use this directly.
/// It will be created automatically when using [`crate::Server::route`] or [`crate::Server::stateful_route`].
pub struct Route<State: 'static + Send + Sync> {
    /// Route Method (GET, POST, ANY, etc.)
    method: Method,

    /// Route path, in its tokenized form.
    path: Path,

    /// Route Handler, either stateless or stateful.
    pub(crate) handler: Handler<State>,
}

// Thanks breon for the idea - https://github.com/rcsc/amplitude/blob/main/amplitude/src/error.rs
/// Error returned by a route handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteError {
    status: Status,
    message: String,
    headers: Vec<Header>,
}

pub trait RouteContext<T> {
    #[track_caller]
    fn context(self, body: impl Display) -> Result<T, RouteError>;
    #[track_caller]
    fn with_context<D: Display>(self, body: impl Fn() -> D) -> Result<T, RouteError>;
}

pub trait AdditionalRouteContext<T> {
    fn status(self, status: Status) -> Result<T, RouteError>;
    fn with_status(self, status: impl Fn() -> Status) -> Result<T, RouteError>;

    fn header(self, name: impl Into<HeaderType>, value: impl AsRef<str>) -> Result<T, RouteError>;
    fn with_header(
        self,
        name: impl Into<HeaderType>,
        value: impl Fn() -> String,
    ) -> Result<T, RouteError>;
}

impl<State: 'static + Send + Sync> Route<State> {
    /// Creates a new route.
    pub(crate) fn new(method: Method, path: String, handler: Handler<State>) -> Self {
        Self {
            method,
            path: Path::new(path),
            handler,
        }
    }

    /// Checks if a Request matches the route.
    /// Returns the path parameters if it does.
    pub(crate) fn matches(&self, req: Arc<Request>) -> Option<Vec<(String, String)>> {
        if self.method != Method::ANY && self.method != req.method {
            return None;
        }
        self.path.match_path(req.path.clone())
    }
}

impl RouteError {
    pub fn as_response(&self) -> Response {
        Response::new()
            .status(self.status)
            .text(self.message.clone())
            .content(Content::TXT)
    }

    pub fn downcast_error(e: &Box<dyn Error>) -> Option<RouteError> {
        e.downcast_ref::<RouteError>().cloned()
    }

    pub fn from_error(e: &Box<dyn Error>) -> Self {
        Self {
            status: Status::InternalServerError,
            message: format!("{:?}", e),
            ..Default::default()
        }
    }
}

impl<T, E: Debug> RouteContext<T> for Result<T, E> {
    fn context(self, body: impl Display) -> Result<T, RouteError> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(RouteError {
                status: Status::InternalServerError,
                message: format!("{body}\n[{}]: {e:?}", panic::Location::caller()),
                ..Default::default()
            }),
        }
    }

    fn with_context<D: Display>(self, body: impl Fn() -> D) -> Result<T, RouteError> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(RouteError {
                status: Status::InternalServerError,
                message: format!("{}\n[{}]: {e:?}", body(), panic::Location::caller()),
                ..Default::default()
            }),
        }
    }
}

impl<T> AdditionalRouteContext<T> for Result<T, RouteError> {
    fn status(self, status: Status) -> Result<T, RouteError> {
        match self {
            Ok(x) => Ok(x),
            Err(mut e) => {
                e.status = status;
                Err(e)
            }
        }
    }

    fn with_status(self, status: impl Fn() -> Status) -> Result<T, RouteError> {
        match self {
            Ok(x) => Ok(x),
            Err(mut e) => {
                e.status = status();
                Err(e)
            }
        }
    }

    fn header(self, name: impl Into<HeaderType>, value: impl AsRef<str>) -> Result<T, RouteError> {
        match self {
            Ok(x) => Ok(x),
            Err(mut e) => {
                e.headers.push(Header::new(name, value));
                Err(e)
            }
        }
    }

    fn with_header(
        self,
        name: impl Into<HeaderType>,
        value: impl Fn() -> String,
    ) -> Result<T, RouteError> {
        match self {
            Ok(x) => Ok(x),
            Err(mut e) => {
                e.headers.push(Header::new(name, value()));
                Err(e)
            }
        }
    }
}

impl Error for RouteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl Default for RouteError {
    fn default() -> Self {
        Self {
            status: Status::InternalServerError,
            message: "Internal Server Error".to_string(),
            headers: Default::default(),
        }
    }
}

impl Display for RouteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.status.code(), self.message)
    }
}

#[cfg(test)]
mod test {
    use std::error::Error;

    use crate::Status;

    use super::RouteError;

    #[test]
    fn test_route_error_downcast() {
        let route_error = RouteError {
            status: Status::InternalServerError,
            message: "test".to_string(),
            ..Default::default()
        };
        let error = Box::new(route_error.clone()) as Box<dyn Error>;

        assert_eq!(RouteError::downcast_error(&error).unwrap(), route_error);
    }
}
