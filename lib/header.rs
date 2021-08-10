/// Http header
///
/// Has a name and a value.
pub struct Header {
    name: String,
    value: String,
}

impl Header {
    /// Make a new header
    pub fn new(name: &str, value: &str) -> Header {
        Header {
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    /// Convert a header ref to a header
    pub fn copy(header: &Header) -> Header {
        Header {
            name: header.name.clone(),
            value: header.value.clone(),
        }
    }

    /// Convert a header to a string
    ///
    /// `name: value`
    pub fn to_string(&self) -> String {
        format!("{}: {}", self.name, self.value)
    }

    /// Convert a string to a header
    ///
    /// String must be in the format `name: value`
    pub fn from_string(header: &str) -> Option<Header> {
        let splitted_header: Vec<&str> = header.split(':').collect();
        if splitted_header.len() != 2 {
            return None;
        }
        Some(Header {
            name: splitted_header[0].trim().to_string(),
            value: splitted_header[1].trim().to_string(),
        })
    }
}

impl PartialEq for Header {
    fn eq(&self, other: &Header) -> bool {
        self.name == other.name && self.value == other.value
    }
}

/// Stringify a Vec of headers
///
/// Each header is in the format `name: value`
///
/// Every header is separated by a newline (`\r\n`)
pub fn headers_to_string(headers: Vec<Header>) -> String {
    let headers_string: Vec<String> = headers.iter().map(|header| header.to_string()).collect();
    format!("{}", headers_string.join("\r\n"))
}