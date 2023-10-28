use std::{
    borrow::Cow,
    io::{self, Read},
};

use crate::{
    consts, prelude::MiddleResult, response::ResponseBody, trace::Level, HeaderName, Middleware,
    Request, Response, Status,
};

pub struct Range;

impl Middleware for Range {
    // Inject the Accept-Ranges header into the response.
    fn post(&self, req: &Request, res: &mut Response) -> MiddleResult {
        if let Some(ranges) = req.headers.get(HeaderName::Range) {
            handle_range(ranges, req, res);
        } else if !res.headers.has(HeaderName::AcceptRanges) {
            res.headers.push((HeaderName::AcceptRanges, "bytes").into());
        }
        MiddleResult::Continue
    }
}

fn handle_range(ranges: &Cow<'static, str>, req: &Request, res: &mut Response) {
    let Some(ranges) = Ranges::from_header(ranges) else {
        trace!(Level::Debug, "Invalid range header: {ranges:?}");
        return;
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

    dbg!(&ranges, entity_length);
    let range_response = RangeResponse::new(entity_length, ranges, res.data.take());
    *res = range_response.into();
}

#[derive(Debug)]
struct Ranges {
    ranges: Vec<RangePart>,
}

struct RangeResponse {
    entity_length: usize,
    parts: Vec<RangePart>,
    data: ResponseBody,
    //
    part: usize,
    byte: usize,
}

#[derive(Debug, Clone, Copy)]
struct RangePart {
    start: usize,
    end: usize,
}

impl Ranges {
    fn from_header(raw: &str) -> Option<Self> {
        // TODO: can you do 'bytes = '
        let Some(raw) = raw.strip_prefix("bytes=") else {
            return None;
        };

        let mut ranges = Vec::new();
        for raw_range in raw.split(',') {
            let mut parts = raw_range.split('-');
            let start = parts.next()?.trim().parse().ok()?;
            let end = parts.next()?.trim().parse().ok()?;
            ranges.push(RangePart::new(start, end));
        }

        if ranges.is_empty() {
            return None;
        }

        Some(Ranges { ranges })
    }
}

impl RangeResponse {
    fn new(entity_length: usize, ranges: Ranges, data: ResponseBody) -> Self {
        RangeResponse {
            entity_length,
            parts: ranges.ranges,
            data,
            part: 0,
            byte: 0,
        }
    }

    fn is_single(&self) -> bool {
        self.parts.len() == 1
    }

    fn seek(&mut self, start: usize) -> bool {
        if let ResponseBody::Stream(stream) = &mut self.data {
            let stream = stream.get_mut();
            let mut skip = start - self.byte;
            while skip > 0 {
                let mut buf = [0; consts::CHUNK_SIZE];
                let Ok(read) = stream.read(&mut buf) else {
                    return false;
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

    fn read(&mut self, into: &mut [u8], max: usize) -> io::Result<usize> {
        let mut read = 0;
        while read < into.len() && self.byte < max {
            match &mut self.data {
                ResponseBody::Stream(ref mut stream) => {
                    let stream = stream.get_mut();
                    let n = stream.read(&mut into[read..])?;
                    if n == 0 {
                        break;
                    }
                    read += n;
                    self.byte += n;
                }
                ResponseBody::Static(bytes) => {
                    let [from, to] = [self.byte, max.min(self.byte + into.len() - read)];
                    into[read..].copy_from_slice(&bytes[from..to]);
                }
                ResponseBody::Empty => {
                    todo!()
                }
            }
        }
        Ok(read)
    }
}

impl RangePart {
    fn new(start: usize, end: usize) -> Self {
        RangePart { start, end }
    }

    fn len(&self) -> usize {
        self.end - self.start
    }
}

// TODO: Pass headers?
impl Into<Response> for RangeResponse {
    fn into(self) -> Response {
        if self.is_single() {
            return singlepart_response(self);
        }
        multipart_response(self)
    }
}

fn singlepart_response(res: RangeResponse) -> Response {
    let part = res.parts.into_iter().next().unwrap();
    Response::new()
        .status(Status::PartialContent)
        .header((HeaderName::ContentLength, part.len().to_string()))
        .header((
            HeaderName::ContentRange,
            format!("bytes {}-{}/{}", part.start, part.end, res.entity_length),
        ))
}

fn multipart_response(res: RangeResponse) -> Response {
    todo!()
}

impl Read for RangeResponse {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.part >= self.parts.len() {
            return Ok(0);
        }

        let mut part = self.parts[self.part];
        let mut read = 0;
        while read < buf.len() && self.byte < part.end {
            // Read until we reach the start of the part.
            if self.byte < part.start {
                if !self.seek(part.start) {
                    return Ok(0);
                }
            }

            // Go to next part if we've reached the end of this one.
            if self.byte >= part.end {
                self.part += 1;
                self.byte = 0;
                part = self.parts[self.part];
            }

            let max = part.end.min(self.byte + buf.len() - read);
            let Ok(n) = self.read(&mut buf[read..], max) else {
                break;
            };
            read += n;
            self.byte += n;
        }

        Ok(read)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_range_parse() {
        let range = Ranges::from_header("bytes=0-1023").unwrap();
        assert_eq!(range.ranges, vec![(0, 1023)]);

        let range = Ranges::from_header("bytes=0-50, 100-150").unwrap();
        assert_eq!(range.ranges, vec![(0, 50), (100, 150)]);
    }
}

/*
- bytes=0-1023
- bytes=0-50, 100-150

```
HTTP/1.1 206 Partial Content
Content-Range: bytes 0-1023/146515
Content-Length: 1024
…
(binary content)
```

```
HTTP/1.1 206 Partial Content
Content-Type: multipart/byteranges; boundary=3d6b6a416f9b5
Content-Length: 282

--3d6b6a416f9b5
Content-Type: text/html
Content-Range: bytes 0-50/1270

<!DOCTYPE html>
<html lang="en-US">
<head>
    <title>Example Do
--3d6b6a416f9b5
Content-Type: text/html
Content-Range: bytes 100-150/1270

eta http-equiv="Content-type" content="text/html; c
--3d6b6a416f9b5--
```
*/
