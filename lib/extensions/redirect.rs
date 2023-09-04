//! Shorthand methods for sending redirects.

use crate::{header::Location, Context, Status};

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
///
/// (Source: [Redirects & SEO](https://audisto.com/guides/redirects))
///
/// ## Examples
/// ```
/// # use afire::prelude::*;
/// # use afire::extensions::{RedirectResponse, RedirectType};
/// # use afire::header::Location;
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
///     ctx.header(Location::new("/")).status(RedirectType::Found).send()?;
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

/// Shorthand methods for sending redirects.
pub trait RedirectResponse<State: Send + Sync> {
    /// Creates a redirect response with the default 302 Found status code.
    /// No body is added to the response.
    /// ## Example
    /// ```
    /// # use afire::prelude::*;
    /// # use afire::extensions::RedirectResponse;
    /// # fn test(server: &mut Server) {
    /// server.route(Method::GET, "/redirect", |ctx| {
    ///     ctx.redirect("/").send()?;
    ///     Ok(())
    /// });
    /// # }
    fn redirect(&self, url: impl ToString) -> &Context<State>;

    /// Creates a redirect response with the specified redirect type.
    /// No body is added to the response.
    /// ## Example
    /// ```
    /// # use afire::prelude::*;
    /// # use afire::extensions::{RedirectResponse, RedirectType};
    /// # fn test(server: &mut Server) {
    /// server.route(Method::GET, "/redirect_permanent", |ctx| {
    ///     ctx.redirect_type(RedirectType::MovedPermanently, "/").send()?;
    ///     Ok(())
    /// });
    /// # }
    /// ```
    fn redirect_type(&self, redirect_type: RedirectType, url: impl ToString) -> &Context<State>;
}

impl<State: Send + Sync> RedirectResponse<State> for Context<State> {
    fn redirect(&self, url: impl ToString) -> &Context<State> {
        self.status(Status::Found)
            .header(Location::new(url.to_string()))
    }

    fn redirect_type(&self, redirect_type: RedirectType, url: impl ToString) -> &Context<State> {
        self.status(redirect_type)
            .header(Location::new(url.to_string()))
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
