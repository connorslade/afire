use std::{
    collections::HashMap,
    error,
    sync::{Arc, RwLock},
};

use afire::{
    extensions::{Logger, RealIp, ServeStatic},
    middleware::Middleware,
    trace::{set_log_level, Level},
    Content, Method, Request, Response, Server,
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

fn main() -> Result<(), Box<dyn error::Error>> {
    // Show some helpful information during startup.
    set_log_level(Level::Trace);

    // Create a new afire server on localhost:8080 with 4 worker threads and the App state.
    let mut server = Server::<App>::new("localhost", 8080)
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
    server.route(Method::GET, "/random", |ctx| {
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

    server.route(Method::GET, "/analytics", |ctx| {
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
