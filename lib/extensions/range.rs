use crate::{prelude::MiddleResult, HeaderName, Middleware, Request, Response, Status};

pub struct Range;

impl Middleware for Range {
    // Inject the Accept-Ranges header into the response.
    fn post(&self, req: &Request, res: &mut Response) -> MiddleResult {
        if let Some(ranges) = req.headers.get(HeaderName::Range) {
            let Some(ranges) = Ranges::from_header(ranges) else {
                return MiddleResult::Continue;
            };

            return MiddleResult::Continue;
        } else if !res.headers.has(HeaderName::AcceptRanges) {
            res.headers.push((HeaderName::AcceptRanges, "bytes").into());
        }
        MiddleResult::Continue
    }
}

struct Ranges {
    ranges: Vec<(u64, u64)>,
}

struct RangeResponse {
    entity_length: u64,
    parts: Vec<RangePart>,
}

struct RangePart {
    start: u64,
    end: u64,
    data: Vec<u8>,
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
            ranges.push((start, end));
        }

        Some(Ranges { ranges })
    }
}

// TODO: Pass headers?
impl Into<Response> for RangeResponse {
    fn into(self) -> Response {
        if self.parts.len() == 1 {
            let part = self.parts.into_iter().next().unwrap();
            Response::new()
                .status(Status::PartialContent)
                .header((HeaderName::ContentLength, part.data.len().to_string()))
                .header((
                    HeaderName::ContentRange,
                    format!("bytes {}-{}/{}", part.start, part.end, self.entity_length),
                ))
        } else {
            unimplemented!("multipart/byteranges")
        }
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
