//! Base64 encoding and decoding.
//! - Reference: https://renenyffenegger.ch/notes/development/Base64/Encoding-and-decoding-base-64-with-cpp
//! - Reference: https://dev.to/tiemen/implementing-base64-from-scratch-in-rust-kb1

const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789+/";

/// Encodes a byte slice into a base64 string (with padding).
pub fn encode(inp: &[u8]) -> String {
    let end_len = (inp.len() + 2) / 3 * 4;
    let mut out = String::with_capacity(end_len);

    for i in (0..inp.len()).step_by(3) {
        out.push(BASE64_CHARS[((inp[i] & 0xfc) >> 2) as usize] as char);

        if i + 1 < inp.len() {
            out.push(
                BASE64_CHARS[(((inp[i] & 0x03) << 4) + ((inp[i + 1] & 0xf0) >> 4)) as usize]
                    as char,
            );

            if i + 2 < inp.len() {
                out.push(
                    BASE64_CHARS[(((inp[i + 1] & 0x0f) << 2) + ((inp[i + 2] & 0xc0) >> 6)) as usize]
                        as char,
                );
                out.push(BASE64_CHARS[(inp[i + 2] & 0x3f) as usize] as char);
                continue;
            }

            out.push(BASE64_CHARS[((inp[i + 1] & 0x0f) << 2) as usize] as char);
            out.push('=');
            continue;
        }

        out.push(BASE64_CHARS[((inp[i] & 0x03) << 4) as usize] as char);
        out.push('=');
        out.push('=');
    }

    out
}

/// Decodes a base64 string into a byte slice.
pub fn decode(inp: &str) -> Option<Vec<u8>> {
    if inp.is_empty() {
        return Some(Vec::new());
    }

    let result = inp
        .chars()
        .collect::<Vec<_>>()
        .chunks(4)
        .map(|chunk| {
            chunk
                .iter()
                .filter(|character| *character != &'=')
                .map(|character| from_char(*character as u8).unwrap())
                .collect()
        })
        .flat_map(merge)
        .collect::<Vec<_>>();

    Some(result)
}

fn from_char(chr: u8) -> Option<u8> {
    Some(match chr as char {
        'A'..='Z' => chr as u8 - b'A',
        'a'..='z' => chr as u8 - b'a' + 26,
        '0'..='9' => chr as u8 - b'0' + 52,
        '+' => 62,
        '/' => 63,
        _ => return None,
    })
}

fn merge(bytes: Vec<u8>) -> Vec<u8> {
    let mut out = match bytes.len() {
        2 => vec![
            (bytes[0] & 0b00111111) << 2 | bytes[1] >> 4,
            (bytes[1] & 0b00001111) << 4,
        ],
        3 => vec![
            (bytes[0] & 0b00111111) << 2 | bytes[1] >> 4,
            (bytes[1] & 0b00001111) << 4 | bytes[2] >> 2,
            (bytes[2] & 0b00000011) << 6,
        ],
        4 => vec![
            (bytes[0] & 0b00111111) << 2 | bytes[1] >> 4,
            (bytes[1] & 0b00001111) << 4 | bytes[2] >> 2,
            (bytes[2] & 0b00000011) << 6 | bytes[3] & 0b00111111,
        ],
        _ => unreachable!(),
    };

    out.retain(|x| *x != 0);
    out
}

#[cfg(test)]
mod test {
    use super::{decode, encode};

    #[test]
    fn test_base64_encode() {
        assert_eq!(encode(b""), "");
        assert_eq!(encode(b"f"), "Zg==");
        assert_eq!(encode(b"fo"), "Zm8=");
        assert_eq!(encode(b"foo"), "Zm9v");
        assert_eq!(encode(b"foob"), "Zm9vYg==");
        assert_eq!(encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn test_base64_decode() {
        assert_eq!(decode("").unwrap(), b"");
        assert_eq!(decode("Zg==").unwrap(), b"f");
        assert_eq!(decode("Zm8=").unwrap(), b"fo");
        assert_eq!(decode("Zm9v").unwrap(), b"foo");
        assert_eq!(decode("Zm9vYg==").unwrap(), b"foob");
        assert_eq!(decode("Zm9vYmE=").unwrap(), b"fooba");
        assert_eq!(decode("Zm9vYmFy").unwrap(), b"foobar");
    }
}
