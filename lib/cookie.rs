use std::fmt;

/// Represents a Cookie
pub struct Cookie {
    pub name: String,
    pub value: String,
}

struct SetCookie {
    pub cookie: Cookie,

    pub expires: Option<String>,
    pub max_age: Option<i64>,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub secure: bool,
}

impl Cookie {
    /// Make a new Cookie
    pub fn new(name: &str, value: &str) -> Cookie {
        Cookie {
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    pub fn from_string(cookie_string: &str) -> Option<Vec<Cookie>> {
        if cookie_string.starts_with("Cookie: ") {
            let cookie_string = &cookie_string[8..];
            let cookies = cookie_string.split("; ").collect::<Vec<&str>>();
            let mut final_cookies = Vec::new();
            for i in cookies {
                let mut cookie_parts = i.splitn(2, '=');
                let name = cookie_parts.next()?;
                let value = cookie_parts.next()?;
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

// // Impl Debug
// impl fmt::Debug for Cookie {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         f.debug_struct("Cookie")
//             .field("name", &self.name)
//             .field("value", &self.value)
//             .field("expires", &self.expires)
//             .field("max_age", &self.max_age)
//             .field("domain", &self.domain)
//             .field("path", &self.path)
//             .field("secure", &self.secure)
//             .finish()
//     }
// }

// // Impl Clone
// impl Clone for Cookie {
//     fn clone(&self) -> Cookie {
//         Cookie {
//             name: self.name.clone(),
//             value: self.value.clone(),
//             expires: self.expires.clone(),
//             max_age: self.max_age.clone(),
//             domain: self.domain.clone(),
//             path: self.path.clone(),
//             secure: self.secure,
//         }
//     }
// }
