use std::error;

use afire::{
    extensions::{RedirectResponse, RouteShorthands},
    route::RouteContext,
    Content, Context, Query, Server, Status,
};
use anyhow::Result;
use serde::Deserialize;
use serde_json::json;

use crate::app::App;

const MIME_JSON: &str = "application/json";
const MIME_FORM: &str = "application/x-www-form-urlencoded";

const INVALID_CONTENT_TYPE: &str =
    "Invalid Content-Type. Must be application/json or application/x-www-form-urlencoded.";

pub fn attach(server: &mut Server<App>) {
    server.post("/paste", |ctx| {
        let content_type = ctx.req.headers.get("Content-Type");

        ctx.header(("Vary", "Content-Type"));
        match content_type {
            Some(c) if c == MIME_JSON => post_api(ctx),
            Some(c) if c == MIME_FORM => post_form(ctx),
            _ => {
                ctx.status(Status::BadRequest)
                    .text(INVALID_CONTENT_TYPE)
                    .content(Content::TXT)
                    .send()?;
                Ok(())
            }
        }
    });
}

fn post_api(ctx: &Context<App>) -> Result<(), Box<dyn error::Error>> {
    #[derive(Deserialize)]
    struct Request {
        name: String,
        paste: String,
    }

    let req = serde_json::from_str::<Request>(&ctx.req.body_str())?;
    let uuid = ctx.app().db.new_paste(&req.paste, &req.name)?;
    println!("[+] Created new paste `{}`", uuid);

    ctx.text(json!({ "uuid": uuid.to_string() }))
        .content(Content::JSON)
        .send()?;
    Ok(())
}

fn post_form(ctx: &Context<App>) -> Result<(), Box<dyn error::Error>> {
    let query = Query::from_body(&ctx.req.body_str());
    let name = query.get("title").context("Missing name")?;
    let paste = query.get("paste").context("Missing paste")?;

    let uuid = ctx.app().db.new_paste(paste, name)?;
    println!("[+] Created new paste `{}`", uuid);

    ctx.text(format!("Created new paste `{}`", uuid))
        .content(Content::TXT)
        .redirect(format!("/paste/{}", uuid))
        .send()?;
    Ok(())
}
