use std::{
    net::{IpAddr, SocketAddr},
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use crate::{
    error::{Result, StartupError},
    header::Headers,
    internal::{
        event_loop::{EventLoop, TcpEventLoop},
        misc::ToHostAddress,
        thread_pool::ThreadPool,
    },
    route::{DefaultErrorHandler, ErrorHandler},
    Header, HeaderName, VERSION,
};

use super::{config::ServerConfig, Server};

pub struct Builder<State> {
    host: Result<IpAddr>,
    port: u16,
    state: State,

    workers: usize,
    default_headers: Headers,
    socket_timeout: Option<Duration>,
    keep_alive: bool,

    event_loop: Box<dyn EventLoop<State> + Send + Sync>,
    error_handler: Box<dyn ErrorHandler<State> + Send + Sync>,
}

impl<State> Builder<State>
where
    State: Send + Sync + 'static,
{
    pub(super) fn new(host: impl ToHostAddress, port: u16, state: State) -> Self {
        Self {
            host: host.to_address(),
            port,
            state,

            workers: 16,
            default_headers: Headers(vec![Header::new(
                HeaderName::Server,
                format!("afire/{VERSION}"),
            )]),
            socket_timeout: None,
            keep_alive: false,

            event_loop: Box::new(TcpEventLoop),
            error_handler: Box::new(DefaultErrorHandler),
        }
    }

    pub fn build(self) -> Result<Server<State>> {
        if self.socket_timeout == Some(Duration::ZERO) {
            return Err(StartupError::InvalidSocketTimeout.into());
        }

        if let Some(i) = self.default_headers.iter().find(|x| x.is_forbidden()) {
            return Err(StartupError::ForbiddenDefaultHeader {
                header: i.clone().name,
            }
            .into());
        }

        Ok(Server {
            event_loop: Box::new(TcpEventLoop),
            routes: vec![],
            middleware: vec![],
            state: Arc::new(self.state),
            error_handler: Box::new(DefaultErrorHandler),
            thread_pool: ThreadPool::new(self.workers),
            config: Arc::new(ServerConfig {
                host: SocketAddr::new(self.host?, self.port),
                default_headers: self.default_headers,
                keep_alive: self.keep_alive,
                socket_timeout: self.socket_timeout,
                running: AtomicBool::new(true),
            }),
        })
    }

    /// Sets the number of worker threads to use, it will resize the threadpool immediately.
    /// The more threads you have, the more concurrent requests you can handle.
    /// The default is 1, which is probably too few for most use cases.
    pub fn workers(mut self, workers: usize) -> Self {
        self.workers = workers;
        self
    }

    /// Add a new default header to the server.
    /// This will be added to every response if it is not already present.
    ///
    /// This will be added to every response
    pub fn default_header(mut self, key: impl Into<HeaderName>, value: impl AsRef<str>) -> Self {
        self.default_headers.push(Header::new(key, value));
        self
    }

    /// Set the timeout for the socket.
    /// This will ensure that the server will not hang on a request for too long.
    /// By default there is no timeout.
    pub fn socket_timeout(mut self, socket_timeout: Duration) -> Self {
        self.socket_timeout = Some(socket_timeout);
        self
    }

    /// Set the keep alive state of the server.
    /// This will determine if the server will keep the connection alive after a request.
    /// By default this is true.
    /// If you aren't using a threadpool, you may want to set this to false.
    pub fn keep_alive(mut self, keep_alive: bool) -> Self {
        self.keep_alive = keep_alive;
        self
    }

    /// Change the server's event loop.
    /// The default is [`TcpEventLoop`], which uses the standard library's built-in TCP listener.
    ///
    /// The [afire_tls](https://github.com/connorslade/afire_tls) crate contains an event loop that uses rustls to handle TLS connections.
    pub fn event_loop(mut self, event_loop: impl EventLoop<State> + Send + Sync + 'static) -> Self {
        self.event_loop = Box::new(event_loop);
        self
    }

    /// Set the panic handler, which is called if a route or middleware panics.
    /// This is only available if the `panic_handler` feature is enabled.
    /// If you don't set it, the default response is 500 "Internal Server Error :/".
    /// Be sure that your panic handler wont panic, because that will just panic the whole application.
    pub fn error_handler(mut self, res: impl ErrorHandler<State> + Send + Sync + 'static) -> Self {
        self.error_handler = Box::new(res);
        self
    }
}
