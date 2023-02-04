use std::fmt;

use crate::error::{ParseError, Result};

/// Http header.
/// Has a name and a value.
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Header {
    /// Name of the Header
    pub name: String,

    /// Value of the Header
    pub value: String,
}

impl Header {
    /// Make a new header from a name and a value, which bolth mut implement AsRef<str>.
    /// ## Example
    /// ```rust
    /// # use afire::Header;
    /// let header1 = Header::new("Content-Type", "text/html");
    /// let header2 = Header::new("Access-Control-Allow-Origin", "*");
    /// ```
    pub fn new(name: impl AsRef<str>, value: impl AsRef<str>) -> Header {
        Header {
            name: name.as_ref().to_owned(),
            value: value.as_ref().to_owned(),
        }
    }

    /// Convert a string to a header.
    /// String must be in the format `name: value`, or an error will be returned.
    /// ## Example
    /// ```rust
    /// # use afire::Header;
    /// let header1 = Header::new("Content-Type", "text/html");
    /// let header2 = Header::from_string("Content-Type: text/html").unwrap();
    ///
    /// assert!(header2 == header1);
    /// ```
    pub fn from_string(header: impl AsRef<str>) -> Result<Header> {
        let header = header.as_ref();
        let mut split_header = header.splitn(2, ':');
        if split_header.clone().count() != 2 {
            return Err(ParseError::InvalidHeader.into());
        }

        let name = match split_header.next() {
            Some(i) => i.trim().to_string(),
            None => return Err(ParseError::InvalidHeader.into()),
        };

        let value = match split_header.next() {
            Some(i) => i.trim().to_string(),
            None => return Err(ParseError::InvalidHeader.into()),
        };

        Ok(Header { name, value })
    }
}

impl fmt::Display for Header {
    /// Convert a header to a string
    /// In format: `name: value`.
    /// ## Example
    /// ```rust
    /// # use afire::Header;
    /// let header1 = Header::new("Content-Type", "text/html");
    /// assert_eq!(header1.to_string(), "Content-Type: text/html");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

/// Stringify a Vec of headers.
/// Each header is in the format `name: value` amd separated by a carrage return and newline (`\r\n`).
pub(crate) fn headers_to_string(headers: &[Header]) -> String {
    let out = headers
        .iter()
        .map(Header::to_string)
        .fold(String::new(), |acc, i| acc + &i + "\r\n");

    out[..out.len() - 2].to_owned()
}
