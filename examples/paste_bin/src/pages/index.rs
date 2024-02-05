use afire::{extensions::RouteShorthands, Content, Server};
use askama::Template;

use crate::{app::App, database::Paste};

#[derive(Template)]
#[template(path = "index.html")]
struct PageTemplate {
    recent_pastes: Vec<Paste>,
}

pub fn attach(server: &mut Server<App>) {
    server.get("/", |ctx| {
        // Get the 10 most recent pastes from the database
        let recent_pastes = ctx.app().db.recent_pastes(10)?;
        let page = PageTemplate { recent_pastes };

        // Render the page and send it to the client
        ctx.text(page.render()?).content(Content::HTML).send()?;
        Ok(())
    })
}

// These filters are used in the askama templates (see templates/index.html)
mod filters {
    use afire::proto::http::date::imp_date;
    use askama::Result;

    pub fn is_empty<T>(s: &[T]) -> Result<bool> {
        Ok(s.is_empty())
    }

    pub fn or_untitled(s: &str) -> Result<&str> {
        Ok(if s.is_empty() { "Untitled" } else { s })
    }

    pub fn readable_time(time: &u64) -> Result<String> {
        Ok(imp_date(*time))
    }
}
