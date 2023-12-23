use afire::{error::AnyResult, headers::Vary, route::RouteError, Content, Context, HeaderName};
use serde_json::json;

use crate::app::App;

pub fn error_handler(ctx: &Context<App>, error: RouteError) -> AnyResult<()> {
    if ctx
        .req
        .headers
        .get(HeaderName::Accept)
        .map(|x| x == "application/json")
        .unwrap_or(false)
    {
        ctx.text(json!({
            "message": error.message,
            "location": error.location.map(|x| x.to_string()),
            "error": error.error.map(|x| format!("{x:?}")),
        }))
        .content(Content::JSON)
    } else {
        ctx.text(format!(
            "Internal Server Error\n{}{}{}",
            error.message,
            error
                .error
                .map(|x| format!("\n{:?}", x))
                .unwrap_or_default(),
            error
                .location
                .map(|x| format!("\n{}", x))
                .unwrap_or_default(),
        ))
        .content(Content::TXT)
    };

    ctx.header(Vary::headers([HeaderName::Accept]))
        .status(error.status)
        .headers(error.headers)
        .send()?;

    Ok(())
}
