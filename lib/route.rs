//! Stuff for defining routes and error handling in routes.
//! Holds [`RouteContext`] and [`AdditionalRouteContext`], which are used to add context to errors.
//! The context can include a message, status code, and headers.

use std::{
    borrow::Cow,
    error::Error,
    fmt::{self, Debug, Display},
    panic::Location,
    sync::Arc,
};

use crate::{
    error::{self, AnyResult},
    internal::router::PathParameters,
    router::Path,
    Content, Context, Header, HeaderName, Method, Request, Response, Server, Status,
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

impl<State: Send + Sync> Debug for Route<State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{} {}", self.method, self.path))
    }
}

pub trait ErrorHandler<State: 'static + Send + Sync> {
    fn handle(&self, server: Arc<Server<State>>, error: RouteError) -> Response;
}

pub struct DefaultErrorHandler;

impl<State: 'static + Send + Sync> ErrorHandler<State> for DefaultErrorHandler {
    fn handle(&self, _server: Arc<Server<State>>, error: RouteError) -> Response {
        let message = match error.location {
            Some(location) => format!(
                "Internal Server Error\n{}\nat {} {}:{}",
                error.message,
                location.file(),
                location.line(),
                location.column()
            ),
            None => format!("Internal Server Error\n{}", error.message),
        };

        let mut res = Response::new()
            .status(error.status)
            .text(message)
            .headers(error.headers);

        if !res.headers.has(HeaderName::ContentType) {
            res = res.content(Content::TXT);
        }

        res
    }
}

// Thanks breon for the idea - https://github.com/rcsc/amplitude/blob/main/amplitude/src/error.rs
/// Error returned by a route handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteError {
    pub location: Option<&'static Location<'static>>,
    // TODO: Add error reference
    // error: Option<Box<dyn Error + Send + Sync>>,
    pub status: Status,
    pub message: Cow<'static, str>,
    pub headers: Vec<Header>,
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
    fn header(self, name: impl Into<HeaderName>, value: impl AsRef<str>) -> Result<T, RouteError>;
    /// Add a header to a `Result<T, RouteError>` with a lazy-evaluated value.
    fn with_header(
        self,
        name: impl Into<HeaderName>,
        value: impl Fn() -> String,
    ) -> Result<T, RouteError>;
}

impl<State: 'static + Send + Sync> Route<State> {
    /// Creates a new route.
    pub(crate) fn new(method: Method, path: &str, handler: Handler<State>) -> error::Result<Self> {
        Ok(Self {
            method,
            path: Path::new(path)?,
            handler,
        })
    }

    /// Checks if a Request matches the route.
    /// Returns the path parameters if it does.
    pub(crate) fn matches(&self, req: Arc<Request>) -> Option<PathParameters> {
        if self.method != Method::ANY && self.method != req.method {
            return None;
        }

        self.path.matches(&req.path)
    }
}

impl RouteError {
    /// Tries to downcast a `Box<dyn Error>` into a RouteError.
    /// If that doesn't work, it will create a new RouteError with the error message.
    pub fn downcast_error(e: Box<dyn Error>) -> RouteError {
        e.downcast_ref::<RouteError>()
            .cloned()
            .unwrap_or_else(|| Self::from_error(e))
    }

    /// Converts any error type (`Box<dyn Error>`) into a RouteError.
    fn from_error(e: Box<dyn Error>) -> Self {
        Self {
            status: Status::InternalServerError,
            message: format!("{:?}", e).into(),
            ..Default::default()
        }
    }
}

impl<T, E: Debug> RouteContext<T> for Result<T, E> {
    fn context(self, body: impl Display) -> Result<T, RouteError> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(RouteError {
                location: Some(Location::caller()),

                status: Status::InternalServerError,
                message: body.to_string().into(),
                ..Default::default()
            }),
        }
    }

    fn with_context<D: Display>(self, body: impl Fn() -> D) -> Result<T, RouteError> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(RouteError {
                location: Some(Location::caller()),
                status: Status::InternalServerError,
                message: body().to_string().into(),
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
                location: Some(Location::caller()),
                status: Status::InternalServerError,
                message: body.to_string().into(),
                ..Default::default()
            }),
        }
    }

    fn with_context<D: Display>(self, body: impl Fn() -> D) -> Result<T, RouteError> {
        match self {
            Some(x) => Ok(x),
            None => Err(RouteError {
                location: Some(Location::caller()),
                status: Status::InternalServerError,
                message: body().to_string().into(),
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

    fn header(self, name: impl Into<HeaderName>, value: impl AsRef<str>) -> Result<T, RouteError> {
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
        name: impl Into<HeaderName>,
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

impl<T> AdditionalRouteContext<T> for Result<T, Box<dyn Error>> {
    fn status(self, status: Status) -> Result<T, RouteError> {
        self.map_err(RouteError::from_error).status(status)
    }

    fn with_status(self, status: impl Fn() -> Status) -> Result<T, RouteError> {
        self.map_err(RouteError::from_error).with_status(status)
    }

    fn header(self, name: impl Into<HeaderName>, value: impl AsRef<str>) -> Result<T, RouteError> {
        self.map_err(RouteError::from_error).header(name, value)
    }

    fn with_header(
        self,
        name: impl Into<HeaderName>,
        value: impl Fn() -> String,
    ) -> Result<T, RouteError> {
        self.map_err(RouteError::from_error)
            .with_header(name, value)
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
            location: None,
            status: Status::InternalServerError,
            message: "Internal Server Error".into(),
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
            message: "test".into(),
            ..Default::default()
        };
        let error = Box::new(route_error.clone()) as Box<dyn Error>;

        assert_eq!(RouteError::downcast_error(error), route_error);
    }
}
