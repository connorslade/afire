//!
//! Cookies!
//! This module provides a simple interface for setting and receiving cookies.

use std::fmt;

use crate::encoding::decode_url;

/// Represents a Cookie
#[derive(Clone, Hash, PartialEq, Eq)]
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
    /// Will only return None of the cookie string does not start with "Cookie:".
    /// If thare are any invalid cookies, they will be ignored.
    pub(crate) fn from_string(cookie_string: &str) -> Option<Vec<Cookie>> {
        if let Some(cookie_string) = cookie_string.strip_prefix("Cookie:") {
            let cookies = cookie_string.trim().split("; ").collect::<Vec<&str>>();
            let mut final_cookies = Vec::new();
            for i in cookies {
                let mut cookie_parts = i.splitn(2, '=');
                let name = match cookie_parts.next() {
                    Some(i) => i.trim(),
                    None => continue,
                };

                let value = match &cookie_parts.next() {
                    Some(i) => i.trim(),
                    None => continue,
                }
                .trim_end_matches(';');

                let name = decode_url(name).unwrap_or_else(|| name.to_owned());
                let value = decode_url(value).unwrap_or_else(|| value.to_owned());
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

// Impl ToString for Cookie
impl fmt::Display for Cookie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}={}", self.name, self.value)
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
        let cookie_string = "Cookie: name=value; name2=value2; name3=value3";
        let cookies = Cookie::from_string(cookie_string).unwrap();
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
        let cookie_string = "Cookie: name=value; name2 value2; name3=value3;";
        let cookies = Cookie::from_string(cookie_string).unwrap();
        assert_eq!(cookies.len(), 2);
        assert_eq!(cookies[0].name, "name");
        assert_eq!(cookies[0].value, "value");
        assert_eq!(cookies[1].name, "name3");
        assert_eq!(cookies[1].value, "value3");
    }

    #[test]
    fn test_invalid_cookie_parse() {
        let cookie_string = "Cookies: name=value; name2=value2; name3=value3";
        let cookies = Cookie::from_string(cookie_string);
        assert_eq!(cookies, None);
    }
}
