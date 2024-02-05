//! Defined in [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110#field.range).

use std::{
    borrow::Cow,
    fmt::{self, Display},
    io::{self, Read},
    ops::{Deref, RangeInclusive},
};

use crate::{
    consts, prelude::MiddleResult, response::ResponseBody, HeaderName, Method, Middleware, Request,
    Response, Status,
};

/// Implements the HTTP Range header as defined in [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110#field.range).
pub struct Range {
    /// Will reject requests with invalid range headers (416 RangeNotSatisfiable) rather than returning the full response.
    reject_invalid: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum RangeError {
    MissingBytesPrefix,
    MalformedRange,
    InvalidNumber,
    EmptyRange,
    UnorderedRange,
    OverlappingRange,
    InvalidRange,
}

#[derive(Debug, PartialEq, Eq)]
struct Ranges {
    ranges: Vec<RangeInclusive<usize>>,
}

struct RangeResponse {
    entity_length: usize,
    parts: Ranges,
    data: ResponseBody,
    part: usize,
    byte: usize,
}

impl Middleware for Range {
    // Inject the Accept-Ranges header into the response.
    fn post(&self, req: &Request, res: &mut Response) -> MiddleResult {
        if req.method != Method::GET {
            return MiddleResult::Continue;
        }

        if let Some(ranges) = req.headers.get(HeaderName::Range) {
            self.handle_range(ranges, req, res);
        } else if !res.headers.has(HeaderName::AcceptRanges) {
            res.headers.push((HeaderName::AcceptRanges, "bytes").into());
        }
        MiddleResult::Continue
    }
}

impl Range {
    /// Create a new Range middleware with default settings.
    pub fn new() -> Self {
        Range {
            reject_invalid: false,
        }
    }

    /// Will reject requests with invalid range headers (416 RangeNotSatisfiable) rather than returning the full response.
    /// This is disabled by default.
    pub fn reject_invalid(mut self) -> Self {
        self.reject_invalid = true;
        self
    }

    fn handle_range(&self, ranges: &Cow<'static, str>, _req: &Request, res: &mut Response) {
        let ranges = match Ranges::from_header(ranges) {
            Ok(ranges) => ranges,
            Err(e) => {
                if self.reject_invalid {
                    *res = Response::new().status(Status::RangeNotSatisfiable).text(e);
                }
                return;
            }
        };

        let entity_length = match &res.data {
            ResponseBody::Empty => 0,
            ResponseBody::Static(data) => data.len(),
            ResponseBody::Stream(_) => res
                .headers
                .get(HeaderName::ContentLength)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
        };

        let range_response = RangeResponse::new(entity_length, ranges, res.data.take());
        *res = range_response.into();
    }
}

impl Ranges {
    fn from_header(raw: &str) -> Result<Self, RangeError> {
        // TODO: can you do 'bytes = '
        let Some(raw) = raw.strip_prefix("bytes=") else {
            return Err(RangeError::MissingBytesPrefix);
        };

        let mut ranges = Vec::new();
        for raw_range in raw.split(',') {
            let mut parts = raw_range.split('-');
            let mut parse = || {
                Ok(parts
                    .next()
                    .ok_or(RangeError::MalformedRange)?
                    .trim()
                    .parse::<usize>()
                    .or(Err(RangeError::InvalidNumber))?)
            };
            let (start, end) = (parse()?, parse()?);
            ranges.push(start..=end);
        }

        // Validate ranges.
        if ranges.is_empty() {
            return Err(RangeError::EmptyRange);
        }

        let mut last = ranges[0].clone();
        if *last.start() > *last.end() {
            return Err(RangeError::InvalidRange);
        }

        for range in ranges.iter().skip(1) {
            if *range.start() > *range.end() {
                return Err(RangeError::InvalidRange);
            }

            if *range.start() < *last.start() {
                return Err(RangeError::UnorderedRange);
            }

            if *range.start() <= *last.end() {
                return Err(RangeError::OverlappingRange);
            }

            last = range.clone();
        }

        Ok(Ranges { ranges })
    }
}

impl RangeResponse {
    fn new(entity_length: usize, parts: Ranges, data: ResponseBody) -> Self {
        RangeResponse {
            entity_length,
            parts,
            data,
            part: 0,
            byte: 0,
        }
    }

    fn is_single(&self) -> bool {
        self.parts.len() == 1
    }

    fn seek(&mut self, start: usize) -> bool {
        debug_assert!(start >= self.byte);
        if let ResponseBody::Stream(stream) = &mut self.data {
            let stream = stream.get_mut();
            let mut skip = start - self.byte;
            while skip > 0 {
                let mut buf = [0; consts::CHUNK_SIZE];
                let read = match stream.read(&mut buf[0..skip.min(consts::CHUNK_SIZE)]) {
                    Ok(read) => read,
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(_) => return false,
                };
                if read == 0 {
                    return false;
                }
                skip -= read;
            }
        }

        self.byte = start;
        true
    }

    fn read(&mut self, into: &mut [u8], end: usize) -> io::Result<usize> {
        let mut read = 0;
        while read < into.len() && self.byte < end {
            let to = end.min(self.byte + into.len());
            let copied = (self.byte..=to).count();
            match &mut self.data {
                ResponseBody::Stream(ref mut stream) => {
                    let stream = stream.get_mut();
                    let n = match stream.read(&mut into[read..read + copied]) {
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                        Err(e) => return Err(e),
                    };
                    read += n;
                    self.byte += n;
                }
                ResponseBody::Static(bytes) => {
                    into[read..read + copied].copy_from_slice(&bytes[self.byte..=to]);
                    read += copied;
                    self.byte += copied;
                }
                ResponseBody::Empty => return Ok(0),
            }
        }
        Ok(read)
    }
}

// TODO: Pass headers?
impl Into<Response> for RangeResponse {
    fn into(self) -> Response {
        if self.is_single() {
            singlepart_response(self)
        } else {
            multipart_response(self)
        }
    }
}

fn singlepart_response(res: RangeResponse) -> Response {
    let part = &res.parts[0];
    Response::new()
        .status(Status::PartialContent)
        .header((
            HeaderName::ContentLength,
            (part.end() - part.start()).to_string(),
        ))
        .header((
            HeaderName::ContentRange,
            format!(
                "bytes {}-{}/{}",
                part.start(),
                part.end(),
                res.entity_length
            ),
        ))
        .stream(res)
}

/*
HTTP/1.1 206 Partial Content
Content-Type: multipart/byteranges; boundary=3d6b6a416f9b5
Content-Length: 282

--3d6b6a416f9b5
Content-Type: text/html
Content-Range: bytes 0-50/1270

...DATA...
--3d6b6a416f9b5
Content-Type: text/html
Content-Range: bytes 100-150/1270

...DATA...
--3d6b6a416f9b5--
*/
fn multipart_response(_res: RangeResponse) -> Response {
    todo!()
}

impl Read for RangeResponse {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.part >= self.parts.len() {
            return Ok(0);
        }

        let mut part = self.parts[self.part].clone();
        let mut read = 0;
        while read < buf.len() {
            // Read until we reach the start of the part.
            if self.byte < *part.start() {
                self.seek(*part.start());
                return Ok(0);
            }

            // Go to next part if we've reached the end of this one.
            if self.byte >= *part.end() {
                self.part += 1;
                self.byte = 0;

                if self.part >= self.parts.len() {
                    break;
                } else {
                    part = self.parts[self.part].clone();
                    continue;
                }
            }

            let end = (*part.end()).min(self.byte + buf.len());
            let Ok(n) = self.read(&mut buf[read..], end) else {
                break;
            };
            read += n;
            self.byte += n;
        }

        Ok(read)
    }
}

impl Display for RangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Range header: ")?;
        f.write_str(match self {
            RangeError::MissingBytesPrefix => "Missing bytes prefix",
            RangeError::MalformedRange => "Malformed range",
            RangeError::InvalidNumber => "Invalid number",
            RangeError::EmptyRange => "No ranges specified",
            RangeError::UnorderedRange => "Ranges are not ordered",
            RangeError::OverlappingRange => "Ranges overlap",
            RangeError::InvalidRange => "Invalid range",
        })
    }
}

impl Deref for Ranges {
    type Target = Vec<RangeInclusive<usize>>;

    fn deref(&self) -> &Self::Target {
        &self.ranges
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_range_parse() {
        let range = Ranges::from_header("bytes=0-1023").unwrap();
        assert_eq!(range.ranges, vec![0..=1023]);

        let range = Ranges::from_header("bytes=0-50, 100-150").unwrap();
        assert_eq!(range.ranges, vec![0..=50, 100..=150]);
    }

    #[test]
    fn test_range_read_static() {
        let body = ResponseBody::Static(b"Hello, world!".to_vec());
        let mut range = RangeResponse::new(13, Ranges::from_header("bytes=0-5").unwrap(), body);

        let mut buf = [0; 6];
        let n = range.read(&mut buf, 5).unwrap();

        assert_eq!(&buf, b"Hello,");
        assert_eq!(n, 6);
    }

    #[test]
    fn test_range_read_static_empty() {
        let body = ResponseBody::Static(b"Hello, world!".to_vec());
        let mut range = RangeResponse::new(13, Ranges::from_header("bytes=0-0").unwrap(), body);

        let mut buf = [0; 6];
        let n = range.read(&mut buf, 0).unwrap();
        assert!(buf.iter().all(|&b| b == 0), "buf: {:?}", buf);
        assert_eq!(n, 0);
    }

    #[test]
    fn test_static_response() {
        let body = ResponseBody::Static(b"Hello, world!".to_vec());
        let mut range = RangeResponse::new(13, Ranges::from_header("bytes=0-5").unwrap(), body);

        let mut buf = [0; 6];
        let n = Read::read(&mut range, &mut buf).unwrap();
        assert_eq!(&buf, b"Hello,");
        assert_eq!(n, 6);
    }

    #[test]
    fn test_range_parse_errors() {
        assert_eq!(
            Ranges::from_header("bytes=0-50, 100-150, 100-200"),
            Err(RangeError::OverlappingRange)
        );
        assert_eq!(
            Ranges::from_header("bytes=0-50, 100-150, 50-200"),
            Err(RangeError::UnorderedRange)
        );
        assert_eq!(
            Ranges::from_header("bytes=100-150, 50-60"),
            Err(RangeError::UnorderedRange)
        );
    }
}
