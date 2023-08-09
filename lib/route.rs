use std::{
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

/// Convert any Result<T, E> into a Result<T, RouteError>.
/// This allows you to set the status code and message of the error.
pub trait RouteContext<T> {
    /// Adds additional context to an error.
    #[track_caller]
    fn context(self, body: impl Display) -> Result<T, RouteError>;
    /// Adds additional context to an error with a lazy-evaluated message.
    #[track_caller]
    fn with_context<D: Display>(self, body: impl Fn() -> D) -> Result<T, RouteError>;
}

/// Add additional context to a RouteError.
/// This can be the status code or headers.
pub trait AdditionalRouteContext<T> {
    /// Set the status code of a `Result<T, RouteError>`.
    fn status(self, status: Status) -> Result<T, RouteError>;
    /// Set the status code of a `Result<T, RouteError>` with a lazy-evaluated status code.
    fn with_status(self, status: impl Fn() -> Status) -> Result<T, RouteError>;

    /// Add a header to a `Result<T, RouteError>`.
    fn header(self, name: impl Into<HeaderType>, value: impl AsRef<str>) -> Result<T, RouteError>;
    /// Add a header to a `Result<T, RouteError>` with a lazy-evaluated value.
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
    /// Convert a RouteError into a Response.
    /// It will have the defined status code, message, and headers.
    /// If none is supplied, the content type will be text/plain.
    pub fn as_response(&self) -> Response {
        let mut res = Response::new()
            .status(self.status)
            .text(&self.message)
            .headers(&self.headers);

        if !res.headers.has(HeaderType::ContentType) {
            res = res.content(Content::TXT);
        }

        res
    }

    /// Tries to downcast a Box<dyn Error> into a RouteError.
    pub fn downcast_error(e: &Box<dyn Error>) -> Option<RouteError> {
        e.downcast_ref::<RouteError>().cloned()
    }

    /// Converts any error type (Box<dyn Error>) into a RouteError.
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

impl<T> RouteContext<T> for Option<T> {
    fn context(self, body: impl Display) -> Result<T, RouteError> {
        match self {
            Some(x) => Ok(x),
            None => Err(RouteError {
                status: Status::InternalServerError,
                message: format!("{body}\n[{}]", panic::Location::caller()),
                ..Default::default()
            }),
        }
    }

    fn with_context<D: Display>(self, body: impl Fn() -> D) -> Result<T, RouteError> {
        match self {
            Some(x) => Ok(x),
            None => Err(RouteError {
                status: Status::InternalServerError,
                message: format!("{}\n[{}]", body(), panic::Location::caller()),
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
