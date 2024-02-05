use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use afire::{
    error::AnyResult,
    extensions::{Logger, RealIp, ServeStatic},
    middleware::Middleware,
    route::RouteError,
    trace::{set_log_level, Level},
    Content, Context, Method, Request, Response, Server,
};
use rand::Rng;
use serde::Deserialize;
use serde_json::json;

// The App struct will be used as the state for our server.
// A reference to the state is passed to middleware and route handlers.
struct App {
    // Records the number of times each page has been visited.
    analytics: RwLock<HashMap<String, u64>>,
}

fn main() -> AnyResult<()> {
    // Show some helpful information during startup.
    // afire log level is global and will affect all afire servers in your application
    // (although there is usually only one)
    set_log_level(Level::Trace);

    // Create a new afire server on localhost:8080 with 4 worker threads, our custom error handler, and the App state.
    let mut server = Server::<App>::new("localhost", 8080)
        .error_handler(error_handler)
        .workers(4)
        .state(App::new());

    // Add extension to serve static files from the ./static directory.
    ServeStatic::new("./static").attach(&mut server);

    // Add middleware to log requests to the console.
    Logger::new().attach(&mut server);

    // Add our analytics middleware.
    Analytics::new(server.app()).attach(&mut server);

    // Add a route at GET /greet/{name} that will greet the user.
    server.route(Method::GET, "/greet/{name}", |ctx| {
        let name = ctx.param("name");
        ctx.text(format!("Hello, {}!", name)).send()?;
        Ok(())
    });

    // Add a route to respond with the client's IP address.
    server.route(Method::GET, "/ip", |ctx| {
        // Get the client's IP address, even if they are behind a reverse proxy.
        let ip = ctx.real_ip();
        ctx.text(format!("Your IP is: {}", ip)).send()?;
        Ok(())
    });

    // Add an api route to pick a random number.
    server.route(Method::GET, "/api/random", |ctx| {
        #[derive(Deserialize)]
        struct Request {
            min: u32,
            max: u32,
        }

        let data = serde_json::from_slice::<Request>(&ctx.req.body)?;
        let num = rand::thread_rng().gen_range(data.min..=data.max);

        ctx.text(json!({ "number": num }))
            .content(Content::JSON)
            .send()?;
        Ok(())
    });

    server.route(Method::GET, "/api/analytics", |ctx| {
        let res = json!(*ctx.app().analytics.read().unwrap());
        ctx.text(res).content(Content::JSON).send()?;
        Ok(())
    });

    server.run()?;
    Ok(())
}

struct Analytics {
    // Store a reference to the app state so we can access it in the middleware.
    app: Arc<App>,
}

impl Middleware for Analytics {
    // Pre middleware is run before the route handler.
    // You can modify the request, return a response or continue to the next middleware or route handler.
    fn end(&self, req: Arc<Request>, res: &Response) {
        // Only record successful requests.
        if !res.status.is_success() {
            return;
        }

        // Update the analytics with the path of the request.
        let mut analytics = self.app.analytics.write().unwrap();
        analytics
            .entry(req.path.to_owned())
            .and_modify(|x| *x += 1)
            .or_insert(1);
    }
}

/// Custom error handler that returns JSON for API routes and plain text for other routes.
/// Note: This is just an example, in your own application you should consider making use of the location and error fields of RouteError.
fn error_handler(ctx: &Context<App>, error: RouteError) -> AnyResult<()> {
    if ctx.req.path.starts_with("/api") {
        ctx.text(json!({
            "message": error.message,
        }))
        .content(Content::JSON)
    } else {
        ctx.text(format!("Internal Server Error\n{}", error.message))
            .content(Content::TXT)
    }
    .status(error.status)
    .headers(error.headers)
    .send()?;
    Ok(())
}

impl App {
    fn new() -> Self {
        Self {
            analytics: RwLock::new(HashMap::new()),
        }
    }
}

impl Analytics {
    fn new(app: Arc<App>) -> Self {
        Self { app }
    }
}
