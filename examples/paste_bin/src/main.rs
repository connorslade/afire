use std::process;

use afire::{
    extensions::{Date, Head, Logger},
    Middleware, Server,
};
use anyhow::Result;

use app::App;
mod app;
mod config;
mod database;
mod pages;
mod routes;

fn main() -> Result<()> {
    let app = App::new()?;

    let mut server = Server::new(&app.config.server.host, app.config.server.port)
        .workers(app.config.server.workers)
        .state(app);

    // Add some middleware
    Date.attach(&mut server);
    Head::new().attach(&mut server);
    Logger::new().attach(&mut server);

    // Add all the api routes and pages
    routes::attach(&mut server);
    pages::attach(&mut server);

    // Setup a ctrl-c handler to cleanup the database
    let ctrlc_app = server.app();
    ctrlc::set_handler(move || {
        ctrlc_app.cleanup().unwrap();
        process::exit(0);
    })
    .unwrap();

    // Start the server :tada:
    server.run()?;
    Ok(())
}
