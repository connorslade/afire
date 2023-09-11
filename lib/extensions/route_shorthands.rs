use crate::{error::AnyResult, Context, Server};

macro_rules! route_shorthands {
    ($($name: ident => Method::$method: ident,)+) => {
        /// Shorthands for defining routes.
        /// Instead of using the method enum with the [`Server::route`] method, you can use `server.<method>(path, handler)`.
        /// ## Examples
        /// ```
        /// # use afire::prelude::*;
        /// use afire::extensions::RouteShorthands;
        ///
        /// # fn test(server: &mut Server) {
        /// server.get("/", |ctx| {
        ///     ctx.text("Hello World!").send()?;
        ///     Ok(())
        /// });
        /// # }
        pub trait RouteShorthands<State> {
            $(
                #[doc = concat!("A shorthand for for `server.route(Method::", stringify!($method), ", <path>, <handler>)`.")]
                fn $name(
                    &mut self,
                    path: impl AsRef<str>,
                    handler: impl Fn(&Context<State>) -> AnyResult<()> + Send + Sync + 'static,
                );
            )+
        }

        impl<State: Send + Sync> RouteShorthands<State> for Server<State> {
            $(
                fn $name(
                    &mut self,
                    path: impl AsRef<str>,
                    handler: impl Fn(&Context<State>) -> AnyResult<()> + Send + Sync + 'static,
                ) {
                    self.route($crate::Method::$method, path, handler);
                }
            )+
        }
    };
}

route_shorthands! {
    any     => Method::ANY,
    get     => Method::GET,
    post    => Method::POST,
    put     => Method::PUT,
    delete  => Method::DELETE,
    head    => Method::HEAD,
    options => Method::OPTIONS,
    trace   => Method::TRACE,
    patch   => Method::PATCH,
}
