use std::error;

use afire::{
    extensions::{RedirectResponse, RouteShorthands},
    header::Vary,
    route::RouteContext,
    Content, Context, HeaderName, Query, Server, Status,
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
        // Get the content type from the request headers
        let content_type = ctx.req.headers.get(HeaderName::ContentType);

        // The vary header tells clients and proxies what headers can change the response
        ctx.header(Vary::headers([HeaderName::ContentType]));
        match content_type {
            // Depending on the content type, call the correct handler
            Some(c) if c == MIME_JSON => post_api(ctx),
            Some(c) if c == MIME_FORM => post_form(ctx),
            // Fall back to an error message
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
    // Define the request structure
    #[derive(Deserialize)]
    struct Request {
        name: String,
        paste: String,
    }

    // Get the request body and parse it as json
    let req = serde_json::from_str::<Request>(&ctx.req.body_str())?;
    // Create a new paste in the database
    let uuid = ctx.app().db.new_paste(&req.paste, &req.name)?;
    println!("[+] Created new paste `{}`", uuid);

    // Send the uuid back to the client
    ctx.text(json!({ "uuid": uuid.to_string() }))
        .content(Content::JSON)
        .send()?;
    Ok(())
}

fn post_form(ctx: &Context<App>) -> Result<(), Box<dyn error::Error>> {
    // Parse the query from the request body
    let query = Query::from_body(&ctx.req.body_str());
    // Pull out the name and paste from the query
    // (automatically url-decoded)
    let name = query.get("title").context("Missing name")?;
    let paste = query.get("paste").context("Missing paste")?;

    // Create a new paste in the database
    let uuid = ctx.app().db.new_paste(paste, name)?;
    println!("[+] Created new paste `{}`", uuid);

    // Redirect the client to the new paste
    ctx.text(format!("Created new paste `{}`", uuid))
        .content(Content::TXT)
        .redirect(format!("/paste/{}", uuid))
        .send()?;
    Ok(())
}
