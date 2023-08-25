//! Url encoding and decoding.

use std::str::Chars;

/// Decode a url encoded string.
/// Supports `+` and `%` encoding.
/// If invalid encoding is found, it will be ignored.
pub fn decode(url: &str) -> String {
    fn decode_hex(chars: &mut Chars<'_>) -> Option<char> {
        let mut hex = String::new();
        hex.push(chars.next()?);
        hex.push(chars.next()?);
        u8::from_str_radix(&hex, 16).ok().map(char::from)
    }

    let mut chars = url.chars();
    let mut out = String::with_capacity(url.len());

    while let Some(i) = chars.next() {
        match i {
            '+' => out.push(' '),
            '%' => {
                if let Some(chr) = decode_hex(&mut chars) {
                    out.push(chr);
                }
            }
            _ => out.push(i),
        }
    }

    out
}

/// Encodes a string with url encoding.
/// Uses `%20` for spaces not `+`.
/// Allowed characters are `A-Z`, `a-z`, `0-9`, `-`, `.`, `_` and `~`.
pub fn encode(url: &str) -> String {
    const ALLOWED_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                   abcdefghijklmnopqrstuvwxyz\
                                   0123456789-._~";

    let mut out = String::with_capacity(url.len());

    for i in url.chars() {
        if i.is_ascii() && ALLOWED_CHARS.contains(&(i as u8)) {
            out.push(i);
            continue;
        }
        out.push_str(&format!("%{:02X}", i as u8));
    }

    out
}

#[cfg(test)]
mod test {
    use super::{decode, encode};

    #[test]
    fn test_url_decode() {
        assert_eq!(decode("hello+world"), "hello world");
        assert_eq!(decode("hello%20world"), "hello world");
        assert_eq!(
            decode("%3C%3E%22%23%25%7B%7D%7C%5C%5E~%5B%5D%60"),
            "<>\"#%{}|\\^~[]`"
        );
    }

    #[test]
    fn test_url_decode_fail() {
        assert_eq!(decode("hello%20world%"), "hello world");
        assert_eq!(decode("hello%20world%2"), "hello world");
        assert_eq!(decode("hello%20world%2G"), "hello world");
    }

    #[test]
    fn test_url_encode() {
        assert_eq!(encode("hello world"), "hello%20world");
        assert_eq!(encode("hello%20world"), "hello%2520world");
        assert_eq!(
            encode("<>\"#%{}|\\^~[]`"),
            "%3C%3E%22%23%25%7B%7D%7C%5C%5E~%5B%5D%60"
        );
    }
}
