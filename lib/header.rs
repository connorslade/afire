use std::fmt;

/// Http header
///
/// Has a name and a value.
#[derive(Hash, PartialEq, Eq)]
pub struct Header {
    /// Name of the Header
    pub name: String,

    /// Value of the Header
    pub value: String,
}

impl Header {
    /// Make a new header
    /// ## Example
    /// ```rust
    /// // Import Modules
    /// use afire::Header;
    ///
    /// let header1 = Header::new("Content-Type", "text/html");
    /// let header2 = Header::new("Access-Control-Allow-Origin", "*");
    /// ```
    pub fn new<T, M>(name: T, value: M) -> Header
    where
        T: fmt::Display,
        M: fmt::Display,
    {
        Header {
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    /// Convert a header ref to a header
    /// ## Example
    /// ```rust
    /// // Import Modules
    /// use afire::Header;
    ///
    /// let header1 = &Header::new("Content-Type", "text/html");
    /// let header2 = Header::new("Content-Type", "text/html");
    ///
    /// assert!(header1.copy() == header2);
    /// ```
    pub fn copy(&self) -> Header {
        Header {
            name: self.name.clone(),
            value: self.value.clone(),
        }
    }

    /// Convert a string to a header
    ///
    /// String must be in the format `name: value`
    /// ## Example
    /// ```rust
    /// // Import Modules
    /// use afire::Header;
    ///
    /// let header1 = Header::new("Content-Type", "text/html");
    /// let header2 = Header::from_string("Content-Type: text/html").unwrap();
    ///
    /// assert!(header2 == header1);
    /// ```
    pub fn from_string<T>(header: T) -> Option<Header>
    where
        T: fmt::Display,
    {
        let header = header.to_string();
        let mut split_header = header.splitn(2, ':');
        if split_header.clone().count() != 2 {
            return None;
        }
        Some(Header {
            name: split_header.next()?.trim().to_string(),
            value: split_header.next()?.trim().to_string(),
        })
    }
}

impl fmt::Display for Header {
    /// Convert a header to a string
    ///
    /// Im format: `name: value`
    /// ## Example
    /// ```rust
    /// // Import Modules
    /// use afire::Header;
    ///
    /// let header1 = Header::new("Content-Type", "text/html");
    ///
    /// assert_eq!(header1.to_string(), "Content-Type: text/html");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

/// Stringify a Vec of headers
///
/// Each header is in the format `name: value`
///
/// Every header is separated by a newline (`\r\n`)
pub fn headers_to_string(headers: Vec<Header>) -> String {
    let headers_string: Vec<String> = headers.iter().map(|header| header.to_string()).collect();
    headers_string.join("\r\n")
}

// Impl clone for Header
impl Clone for Header {
    fn clone(&self) -> Header {
        Header {
            name: self.name.clone(),
            value: self.value.clone(),
        }
    }
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Header")
            .field("name", &self.name)
            .field("value", &self.value)
            .finish()
    }
}
