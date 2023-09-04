//! Multipart request parsing.

use std::{
    convert::TryFrom,
    error::Error,
    fmt::Display,
    io::BufRead,
    ops::{Deref, DerefMut},
};

use crate::{header::Headers, Header, Request};

/// A multipart request.
pub struct MultipartData<'a> {
    /// The content type of the request.
    pub content_type: &'a str,
    /// The lists of entries.
    entries: Vec<MultipartEntry<'a>>,
}

/// An entry in a multipart request.
pub struct MultipartEntry<'a> {
    /// The name of the entry.
    pub name: String,
    /// The name of the uploaded file, if applicable.
    pub filename: Option<String>,
    /// Other headers of the entry.
    pub headers: Headers,
    /// The data of the entry.
    pub data: &'a [u8],
}

/// Errors that can occur when parsing a multipart request.
#[derive(Debug)]
pub enum MultipartError {
    /// The request is not a multipart request.
    InvalidContentType,
    /// The request is a multipart request, no boundary is defined.
    InvalidBoundary,
    /// The request is a multipart request, but the boundary is missing.
    InvalidData,
    /// An entry is invalid.
    InvalidEntry,
}

impl<'a> MultipartData<'a> {
    /// Get an entry by name, returns `None` if the entry does not exist.
    pub fn get(&self, name: impl AsRef<str>) -> Option<&MultipartEntry> {
        self.entries.iter().find(|x| x.name == name.as_ref())
    }

    /// Gets a mutable reference to an entry by name, returns `None` if the entry does not exist.
    pub fn get_mut(&'a mut self, name: impl AsRef<str>) -> Option<&mut MultipartEntry> {
        self.entries.iter_mut().find(|x| x.name == name.as_ref())
    }
}

impl<'a> Deref for MultipartData<'a> {
    type Target = Vec<MultipartEntry<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

impl DerefMut for MultipartData<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries
    }
}

impl<'a> TryFrom<&'a Request> for MultipartData<'a> {
    type Error = MultipartError;

    fn try_from(req: &'a Request) -> Result<Self, Self::Error> {
        let content_type = req
            .headers
            .get_header("Content-Type")
            .ok_or(MultipartError::InvalidContentType)?
            .params();

        let body_type = &content_type.value;
        let boundary = content_type
            .get("boundary")
            .ok_or(MultipartError::InvalidBoundary)?;

        if *body_type != "multipart/form-data" {
            return Err(MultipartError::InvalidContentType);
        }

        let boundary = [b"--", boundary.as_bytes()].concat();
        let data = split_boundary(&req.body, &boundary);

        if data.len() < 3 {
            return Err(MultipartError::InvalidData);
        }

        let entries = data[1..data.len() - 1]
            .iter()
            .map(|entry| MultipartEntry::try_from(*entry))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            content_type: body_type,
            entries,
        })
    }
}

impl<'a> TryFrom<&'a [u8]> for MultipartEntry<'a> {
    type Error = MultipartError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        // Split the headers from the data.
        let index = value
            .windows(4)
            .position(|x| x == b"\r\n\r\n")
            .ok_or(MultipartError::InvalidEntry)?
            + 4;

        if index > value.len() {
            return Err(MultipartError::InvalidEntry);
        }

        let (raw_headers, data) = value.split_at(index);

        let mut headers = Vec::new();

        for i in raw_headers
            .lines()
            .map(|x| x.unwrap())
            .filter(|x| !x.is_empty())
        {
            let header = Header::from_string(&i)
                .ok()
                .ok_or(MultipartError::InvalidEntry)?;
            headers.push(header);
        }

        let headers = Headers(headers);
        let content = headers
            .get_header("Content-Disposition")
            .ok_or(MultipartError::InvalidEntry)?;
        let content_params = content.params();

        Ok(Self {
            name: content_params
                .get("name")
                .ok_or(MultipartError::InvalidEntry)?
                .strip_prefix('"')
                .and_then(|x| x.strip_suffix('"'))
                .ok_or(MultipartError::InvalidEntry)?
                .to_string(),
            filename: content_params.get("filename").map(|x| x.to_string()),
            headers,
            data,
        })
    }
}

fn split_boundary<'a>(data: &'a [u8], boundary: &[u8]) -> Vec<&'a [u8]> {
    let indexes = data
        .windows(boundary.len())
        .enumerate()
        .filter(|(_, x)| x == &boundary)
        .map(|(i, _)| (i, i + boundary.len()))
        .collect::<Vec<_>>();

    let mut out = Vec::with_capacity(indexes.len() + 1);
    let mut start = 0;

    for (s, e) in indexes {
        out.push(&data[start..s]);
        start = e;
    }

    out.push(&data[start..]);
    out
}

impl Display for MultipartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultipartError::InvalidContentType => write!(f, "Invalid content type"),
            MultipartError::InvalidBoundary => write!(f, "Invalid boundary"),
            MultipartError::InvalidData => write!(f, "Invalid data"),
            MultipartError::InvalidEntry => write!(f, "Invalid entry"),
        }
    }
}

impl Error for MultipartError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_boundary() {
        let data = b"gomangogotomato";
        let boundary = b"go";
        let out = split_boundary(data, boundary);

        assert_eq!(out.len(), 4);
        assert_eq!(out[0], b"");
        assert_eq!(out[1], b"man");
        assert_eq!(out[2], b"");
        assert_eq!(out[3], b"tomato");
    }
}
