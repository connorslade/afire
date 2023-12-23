use std::sync::Arc;

use afire::{headers::Vary, route::RouteError, Content, HeaderName, Request, Response, Server};
use serde_json::json;

use crate::app::App;

pub fn error_handler(_server: Arc<Server<App>>, req: Arc<Request>, error: RouteError) -> Response {
    if req
        .headers
        .get(HeaderName::Accept)
        .map(|x| x == "application/json")
        .unwrap_or(false)
    {
        Response::new()
            .text(json!({
                "message": error.message,
                "location": error.location.map(|x| x.to_string()),
                "error": error.error.map(|x| format!("{x:?}")),
            }))
            .content(Content::JSON)
    } else {
        Response::new()
            .text(format!(
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
    }
    .header(Vary::headers([HeaderName::Accept]))
    .status(error.status)
    .headers(error.headers)
}
