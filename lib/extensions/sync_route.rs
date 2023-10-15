//! Lets you make routes that return responses synchronously.
//! This is similar to how routes used to work in pre 3.0.0 versions of afire.

use crate::{error::AnyResult, route::Route, trace::emoji, Context, Method, Response, Server};

/// Lets you make routes that return responses synchronously.
pub trait SyncRoute<State> {
    /// Creates a new route on the server.
    /// Unlike [`Server::route`], you send your response by returning a Result<Response, Error> from your handler.
    /// Because of this, you should refrain from using [`Context::send`] in your handler as it will throw an error when trying to send a response twice.]
    ///
    /// Refer to the [`Server::route`] docs for more information on making routes.
    ///
    /// ## Example
    /// ```rust
    /// # use afire::{Server, Response, Method, extensions::SyncRoute};
    /// # fn test(server: &mut Server<()>) {
    /// server.sync_route(Method::GET, "/hello/{name}", |ctx| {
    ///     let name = ctx.param("name");
    ///     Ok(Response::new().text(format!("Hello, {name}!")))
    /// });
    /// # }
    /// ```
    fn sync_route(
        &mut self,
        method: Method,
        path: impl AsRef<str>,
        handler: impl Fn(&Context<State>) -> AnyResult<Response> + Send + Sync + 'static,
    ) -> &mut Self;
}

impl<State: Send + Sync> SyncRoute<State> for Server<State> {
    fn sync_route(
        &mut self,
        method: Method,
        path: impl AsRef<str>,
        handler: impl Fn(&Context<State>) -> AnyResult<Response> + Send + Sync + 'static,
    ) -> &mut Self {
        trace!("{}Adding Route {} {}", emoji("🚗"), method, path.as_ref());

        let handler = move |ctx: &Context<State>| Ok(ctx.with_response(handler(ctx)?).send()?);

        self.routes.push(
            Route::new(method, path.as_ref(), Box::new(handler)).expect("Error creating route."),
        );
        self
    }
}
