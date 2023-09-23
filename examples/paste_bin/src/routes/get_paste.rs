use std::str::FromStr;

use afire::{extensions::RouteShorthands, route::RouteContext, Content, Server};
use uuid::Uuid;

use crate::app::App;

pub fn attach(server: &mut Server<App>) {
    server.get("/paste/{id}", |ctx| {
        let id = ctx.param_idx(0);
        let uuid = Uuid::from_str(id).context("Invalid UUID")?;

        let paste = ctx.app().db.get_paste(uuid)?;
        ctx.text(paste.paste)
            .header(("X-Paste-Date", paste.date.to_string()))
            .content(Content::TXT)
            .send()?;
        Ok(())
    });
}
