//!
//! Cookies!
//! This module provides a simple interface for setting and receiving cookies.

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use crate::encoding::url;

/// Represents a Cookie
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Cookie {
    /// Cookie Key
    pub name: String,

    /// Cookie Value
    pub value: String,
}

/// Represents a Set-Cookie header.
/// Has more information than a normal Cookie (e.g. max-age, domain, path, secure).
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SetCookie {
    /// Base Cookie
    pub cookie: Cookie,

    /// Cookie Max-Age.
    /// Number of seconds until the cookie expires. A zero or negative number will expire the cookie immediately.
    pub max_age: Option<u64>,

    /// Cookie Domain
    pub domain: Option<String>,

    /// Cookie Path where the cookie is valid
    pub path: Option<String>,

    /// Cookie is secure
    pub secure: bool,
}

/// A collection of Cookies.
#[derive(Debug, Clone)]
pub struct CookieJar(pub(crate) Vec<Cookie>);

impl Cookie {
    /// Make a new Cookie from a name and a value.
    /// ## Example
    /// ```
    /// # use afire::Cookie;
    /// let cookie = Cookie::new("name", "value");
    /// ```
    pub fn new(name: impl AsRef<str>, value: impl AsRef<str>) -> Cookie {
        Cookie {
            name: name.as_ref().to_owned(),
            value: value.as_ref().to_owned(),
        }
    }

    /// Make a Vec of Cookies from a String.
    /// Intended for making Cookie Vec from HTTP Headers.
    pub fn from_string(cookie_string: &str) -> Vec<Cookie> {
        let mut out = Vec::new();
        for i in cookie_string.split(';') {
            let (name, value) = match i.split_once('=') {
                Some(i) => (i.0.trim(), i.1.trim()),
                None => continue,
            };

            let name = url::decode(name).unwrap_or_else(|| name.to_owned());
            let value = url::decode(value).unwrap_or_else(|| value.to_owned());
            out.push(Cookie::new(name, value));
        }

        out
    }
}

impl SetCookie {
    /// Make a new SetCookie from a name and a value.
    /// ## Example
    /// ```
    /// use afire::SetCookie;
    /// let cookie = SetCookie::new("name", "value");
    /// ```
    pub fn new(name: impl AsRef<str>, value: impl AsRef<str>) -> SetCookie {
        SetCookie {
            cookie: Cookie::new(name, value),
            max_age: None,
            domain: None,
            path: None,
            secure: false,
        }
    }

    /// Set the Max-Age field of a SetCookie.
    /// This is the number of seconds the cookie should be valid for.
    /// ## Example
    /// ```
    /// # use afire::SetCookie;
    /// let mut cookie = SetCookie::new("name", "value")
    ///     .max_age(10 * 60);
    ///
    /// assert_eq!(cookie.max_age, Some(10*60));
    /// ```
    pub fn max_age(self, max_age: u64) -> SetCookie {
        SetCookie {
            max_age: Some(max_age),
            ..self
        }
    }

    /// Set the Domain field of a SetCookie.
    /// ## Example
    /// ```
    /// # use afire::SetCookie;
    /// let mut cookie = SetCookie::new("name", "value")
    ///     .domain("domain");
    ///
    /// assert_eq!(cookie.domain, Some("domain".to_string()));
    /// ```
    pub fn domain(self, domain: impl AsRef<str>) -> SetCookie {
        SetCookie {
            domain: Some(domain.as_ref().to_owned()),
            ..self
        }
    }

    /// Set the Path field of a SetCookie.
    /// ## Example
    /// ```
    /// # use afire::SetCookie;
    /// let mut cookie = SetCookie::new("name", "value")
    ///     .path("path");
    ///
    /// assert_eq!(cookie.path, Some("path".to_string()));
    /// ```
    pub fn path(self, path: impl AsRef<str>) -> SetCookie {
        SetCookie {
            path: Some(path.as_ref().to_owned()),
            ..self
        }
    }

    /// Set the Secure field of a SetCookie.
    /// ## Example
    /// ```
    /// # use afire::SetCookie;
    /// let mut cookie = SetCookie::new("name", "value")
    ///     .secure(true);
    ///
    /// assert_eq!(cookie.secure, true);
    /// ```
    pub fn secure(self, secure: bool) -> SetCookie {
        let mut new = self;
        new.secure = secure;
        new
    }
}

impl CookieJar {
    /// Create a new empty CookieJar.
    pub fn new() -> CookieJar {
        CookieJar(Vec::new())
    }

    /// Create a new CookieJar from a Vec of Cookies.
    pub fn from_vec(cookies: Vec<Cookie>) -> CookieJar {
        CookieJar(cookies)
    }

    /// Check if the cookie jar contains a cookie with the given name.
    /// ## Example
    /// ```rust
    /// # use afire::cookie::CookieJar;
    /// # fn test(jar: &CookieJar) {
    /// if jar.has("Session") {
    ///     println!("Session cookie exists");
    /// }
    /// # }
    pub fn has(&self, name: &str) -> bool {
        self.iter().any(|i| i.name == name)
    }

    /// Adds a cookie to the jar with the given name and value.
    /// See [`CookieJar::add_cookie`] for a version that takes a [`Cookie`] directly.
    /// ## Example
    /// ```rust
    /// # use afire::cookie::CookieJar;
    /// # fn test(jar: &mut CookieJar) {
    /// jar.add("Session", "1234");
    /// //                  ^^^^
    /// // Very secure session ID
    /// # }
    pub fn add(&mut self, name: impl AsRef<str>, value: impl AsRef<str>) {
        self.0.push(Cookie::new(name, value));
    }

    /// Gets the value of a cookie with the given name.
    /// If the specified cookie does not exist, None is returned.
    /// ## Example
    /// ```rust
    /// # use afire::cookie::CookieJar;
    /// # fn test(jar: &CookieJar) {
    /// if let Some(session) = jar.get("Session") {
    ///     println!("Session cookie value: {}", session);
    /// }
    /// # }
    pub fn get(&self, name: &str) -> Option<&str> {
        self.iter()
            .find(|i| i.name == name)
            .map(|x| x.value.as_str())
    }

    /// Gets a mutable reference to the value of a cookie with the given name.
    /// If the specified cookie does not exist, None is returned.
    /// See [`CookieJar::get`] for a non-mutable version.
    /// ## Example
    /// ```rust
    /// # use afire::cookie::CookieJar;
    /// # fn test(jar: &mut CookieJar) {
    /// if let Some(session) = jar.get_mut("Session") {
    ///     *session = "new value".to_owned();
    /// }
    /// # }
    pub fn get_mut(&mut self, name: &str) -> Option<&mut String> {
        self.iter_mut()
            .find(|i| i.name == name)
            .map(|x| &mut x.value)
    }

    /// Adds the given cookie to the jar.
    /// See [`CookieJar::add`] for a version that takes a name and value directly.
    /// ## Example
    /// ```rust
    /// # use afire::cookie::{CookieJar, Cookie};
    /// # fn test(jar: &mut CookieJar) {
    /// jar.add_cookie(Cookie::new("Session", "1234"));
    /// # }
    pub fn add_cookie(&mut self, cookie: Cookie) {
        self.0.push(cookie);
    }

    /// Gets a reference to the Cookie struct of a cookie with the given name.
    /// If the specified cookie does not exist, None is returned.
    /// ## Example
    /// ```rust
    /// # use afire::cookie::CookieJar;
    /// # fn test(jar: &CookieJar) {
    /// if let Some(session) = jar.get_cookie("Session") {
    ///     println!("Session cookie value: {}", session.value);
    /// }
    /// # }
    pub fn get_cookie(&self, name: &str) -> Option<&Cookie> {
        self.iter().find(|i| i.name == name)
    }

    /// Gets a mutable reference to the Cookie struct of a cookie with the given name.
    /// If the specified cookie does not exist, None is returned.
    /// See [`CookieJar::get_cookie`] for a non-mutable version.
    /// ## Example
    /// ```rust
    /// # use afire::cookie::CookieJar;
    /// # fn test(jar: &mut CookieJar) {
    /// if let Some(session) = jar.get_cookie_mut("Session") {
    ///     session.value = "new value".to_owned();
    /// }
    /// # }
    pub fn get_cookie_mut(&mut self, name: &str) -> Option<&mut Cookie> {
        self.iter_mut().find(|i| i.name == name)
    }
}

impl Default for CookieJar {
    fn default() -> Self {
        CookieJar::new()
    }
}

impl Deref for CookieJar {
    type Target = Vec<Cookie>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CookieJar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Impl ToString for Cookie
impl fmt::Display for Cookie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}={}", self.name, self.value)
    }
}

// Impl Display for SetCookie
impl fmt::Display for SetCookie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut cookie_string = format!("{}={}; ", self.cookie.name, self.cookie.value);

        // Add max_age
        if self.max_age.is_some() {
            cookie_string.push_str(&format!("Max-Age={}; ", self.max_age.unwrap()));
        }

        // Add domain
        if self.domain.is_some() {
            cookie_string.push_str(&format!("Domain={}; ", self.domain.as_ref().unwrap()));
        }

        // Add path
        if self.path.is_some() {
            cookie_string.push_str(&format!("Path={}; ", self.path.as_ref().unwrap()));
        }

        // Add secure
        if self.secure {
            cookie_string.push_str("Secure; ");
        }

        f.write_str(cookie_string.trim_end())
    }
}

#[cfg(test)]
mod test {
    use super::Cookie;

    #[test]
    fn test_cookie_parse() {
        let cookie_string = "name=value; name2=value2; name3=value3";
        let cookies = Cookie::from_string(cookie_string);
        assert_eq!(cookies.len(), 3);
        assert_eq!(cookies[0].name, "name");
        assert_eq!(cookies[0].value, "value");
        assert_eq!(cookies[1].name, "name2");
        assert_eq!(cookies[1].value, "value2");
        assert_eq!(cookies[2].name, "name3");
        assert_eq!(cookies[2].value, "value3");
    }

    #[test]
    fn test_ignore_cookie_parse() {
        let cookie_string = "name=value; name2 value2; name3=value3;";
        let cookies = Cookie::from_string(cookie_string);
        assert_eq!(cookies.len(), 2);
        assert_eq!(cookies[0].name, "name");
        assert_eq!(cookies[0].value, "value");
        assert_eq!(cookies[1].name, "name3");
        assert_eq!(cookies[1].value, "value3");
    }
}
