use std::str::FromStr;

use afire::{extensions::RouteShorthands, route::RouteContext, Content, Server};
use uuid::Uuid;

use crate::app::App;

pub fn attach(server: &mut Server<App>) {
    server.get("/paste/{id}", |ctx| {
        // Get the paste id from the url
        // You could also use ctx.param("id") here,
        // but that can be slower as it has to loop through all params and check if the name matches
        let id = ctx.param_idx(0);
        let uuid = Uuid::from_str(id).context("Invalid UUID")?;

        // Get the paste from the database and send it to the client
        let paste = ctx.app().db.get_paste(uuid)?;
        ctx.text(paste.paste)
            .header(("X-Paste-Name", paste.name))
            .header(("X-Paste-Date", paste.date.to_string()))
            .content(Content::TXT)
            .send()?;
        Ok(())
    });
}
