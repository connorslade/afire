use crate::{Context, HeaderType, Status};

/// Types of redirects that can be sent.
/// This type can be passed to [`RedirectResponse::redirect_type`] to send a redirect with a specific status code, or to the plain [`Context::header`] where you will need to manually define the 'Location' header.
///
/// | Name             | Status | Permanence | Cacheable      | Request Method Subsequent Request |
/// | :--------------- | :----- | :--------- | :------------- | :-------------------------------- |
/// | MovedPermanently | 301    | Permanent  | Yes            | GET or POST may change            |
/// | Found            | 302    | Temporary  | Not by default | GET or POST may change            |
/// | SeeOther         | 303    | Temporary  | Never          | Always GET                        |
/// | Temporary        | 307    | Temporary  | Not by default | May not change                    |
/// | Permanent        | 308    | Permanent  | By default     | May not change                    |
/// (Source: [Redirects & SEO](https://audisto.com/guides/redirects))
///
/// ## Examples
/// ```
/// # use afire::prelude::*;
/// # use afire::extensions::{RedirectResponse, RedirectType};
/// # fn test(server: &mut Server) {
/// // Use the default 302 Found redirect
/// server.route(Method::GET, "/redirect", |ctx| {
///     ctx.redirect("/").send()?;
///     Ok(())
/// });
///
/// // Use a 301 Moved Permanently redirect
/// server.route(Method::GET, "/redirect_permanent", |ctx| {
///     ctx.redirect_type(RedirectType::MovedPermanently, "/").send()?;
///     Ok(())
/// });
///
/// // Set the Location header and status code manually
/// server.route(Method::GET, "/redirect_manual", |ctx| {
///     ctx.header(HeaderType::Location, "/").status(RedirectType::Found).send()?;
///     Ok(())
/// });
/// # }
/// ```
pub enum RedirectType {
    /// **302 Found**
    Found,
    /// **301 Moved Permanently**
    MovedPermanently,
    /// **308 Permanent Redirect**
    Permanent,
    /// **307 Temporary Redirect**
    Temporary,
    /// **303 See Other**
    SeeOther,
}

pub trait RedirectResponse<State: Send + Sync> {
    fn redirect(&self, url: impl AsRef<str>) -> &Context<State>;

    fn redirect_type(&self, redirect_type: RedirectType, url: impl AsRef<str>) -> &Context<State>;
}

impl<State: Send + Sync> RedirectResponse<State> for Context<State> {
    fn redirect(&self, url: impl AsRef<str>) -> &Context<State> {
        self.status(Status::Found).header(HeaderType::Location, url)
    }

    fn redirect_type(&self, redirect_type: RedirectType, url: impl AsRef<str>) -> &Context<State> {
        self.status(redirect_type).header(HeaderType::Location, url)
    }
}

impl From<RedirectType> for Status {
    fn from(redirect_type: RedirectType) -> Self {
        match redirect_type {
            RedirectType::Found => Status::Found,
            RedirectType::MovedPermanently => Status::MovedPermanently,
            RedirectType::Permanent => Status::PermanentRedirect,
            RedirectType::Temporary => Status::TemporaryRedirect,
            RedirectType::SeeOther => Status::SeeOther,
        }
    }
}
