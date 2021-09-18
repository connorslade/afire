/*!
Cookies!

This module provides a simple interface for setting and receiving cookies.

It can be disabled with the `cookies` feature.
*/

use std::fmt;

/// Represents a Cookie
pub struct Cookie {
    /// Cookie Key
    pub name: String,

    /// Cookie Value
    pub value: String,
}

/// Represents a Client's Cookie
///
/// Has more information than a normal Cookie
/// (e.g. max-age, domain, path, secure)
pub struct SetCookie {
    pub cookie: Cookie,

    pub max_age: Option<u64>,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub secure: bool,
}

impl Cookie {
    /// Make a new Cookie
    /// ## Example
    /// ```
    /// use afire::Cookie;
    /// let cookie = Cookie::new("name", "value");
    /// ```
    pub fn new(name: &str, value: &str) -> Cookie {
        Cookie {
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    /// Make a Vec of Cookies from a String
    ///
    /// Intended for making Cookie Vec from HTTP Headers
    pub(crate) fn from_string(cookie_string: &str) -> Option<Vec<Cookie>> {
        if let Some(cookie_string) = cookie_string.strip_prefix("Cookie: ") {
            let cookies = cookie_string.split("; ").collect::<Vec<&str>>();
            let mut final_cookies = Vec::new();
            for i in cookies {
                let mut cookie_parts = i.splitn(2, '=');
                let name = cookie_parts.next()?;
                let value = &cookie_parts.next()?.trim_end_matches(';');
                final_cookies.push(Cookie::new(name, value));
            }
            return Some(final_cookies);
        }
        None
    }
}

// Impl Debug
impl fmt::Debug for Cookie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Cookie")
            .field("name", &self.name)
            .field("value", &self.value)
            .finish()
    }
}

// Impl Clone
impl Clone for Cookie {
    fn clone(&self) -> Cookie {
        Cookie {
            name: self.name.clone(),
            value: self.value.clone(),
        }
    }
}

// Impl ToString for Cookie
impl fmt::Display for Cookie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}={}", self.name, self.value)
    }
}

impl SetCookie {
    /// Make a new simple SetCookie
    /// ## Example
    /// ```
    /// use afire::SetCookie;
    /// let cookie = SetCookie::new("name", "value");
    /// ```
    pub fn new(name: &str, value: &str) -> SetCookie {
        SetCookie {
            cookie: Cookie::new(name, value),
            max_age: None,
            domain: None,
            path: None,
            secure: false,
        }
    }

    /// Make a new SetCookie with all fields
    /// ## Example
    /// ```
    /// use afire::{SetCookie, Cookie};
    /// let cookie = SetCookie::full_new(Cookie::new("name", "value"), 10*60, "domain", "path", true);
    /// ```
    pub fn full_new(
        cookie: Cookie,
        max_age: u64,
        domain: &str,
        path: &str,
        secure: bool,
    ) -> SetCookie {
        SetCookie {
            cookie,
            max_age: Some(max_age),
            domain: Some(domain.to_string()),
            path: Some(path.to_string()),
            secure,
        }
    }

    /// Set the Max-Age field of a SetCookie
    ///
    /// This is the number of seconds the cookie should be valid for.
    /// ## Example
    /// ```
    /// use afire::SetCookie;
    /// let mut cookie = SetCookie::new("name", "value")
    ///     .set_max_age(10 * 60);
    ///
    /// assert_eq!(cookie.max_age, Some(10*60));
    /// ```
    pub fn set_max_age(self, max_age: u64) -> SetCookie {
        let mut new = self;
        new.max_age = Some(max_age);
        new
    }

    /// Set the Domain field of a SetCookie
    /// ## Example
    /// ```
    /// use afire::SetCookie;
    /// let mut cookie = SetCookie::new("name", "value")
    ///     .set_domain("domain");
    ///
    /// assert_eq!(cookie.domain, Some("domain".to_string()));
    /// ```
    pub fn set_domain(self, domain: &str) -> SetCookie {
        let mut new = self;
        new.domain = Some(domain.to_string());
        new
    }

    /// Set the Path field of a SetCookie
    /// ## Example
    /// ```
    /// use afire::SetCookie;
    /// let mut cookie = SetCookie::new("name", "value")
    ///     .set_path("path");
    ///
    /// assert_eq!(cookie.path, Some("path".to_string()));
    /// ```
    pub fn set_path(self, path: &str) -> SetCookie {
        let mut new = self;
        new.path = Some(path.to_string());
        new
    }

    /// Set the Secure field of a SetCookie
    /// ## Example
    /// ```
    /// use afire::SetCookie;
    /// let mut cookie = SetCookie::new("name", "value")
    ///     .set_secure(true);
    ///
    /// assert_eq!(cookie.secure, true);
    /// ```
    pub fn set_secure(self, secure: bool) -> SetCookie {
        let mut new = self;
        new.secure = secure;
        new
    }
}

// Impl Debug for SetCookie
impl fmt::Debug for SetCookie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Cookie")
            .field("name", &self.cookie.name)
            .field("value", &self.cookie.value)
            .field("max_age", &self.max_age)
            .field("domain", &self.domain)
            .field("path", &self.path)
            .field("secure", &self.secure)
            .finish()
    }
}

// Impl Clone for SetCookie
impl Clone for SetCookie {
    fn clone(&self) -> SetCookie {
        SetCookie {
            cookie: self.cookie.clone(),
            max_age: self.max_age,
            domain: self.domain.clone(),
            path: self.path.clone(),
            secure: self.secure,
        }
    }
}

// Impl ToString for SetCookie
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
