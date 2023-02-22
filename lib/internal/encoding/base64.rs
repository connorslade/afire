//! Base64 encoding and decoding.
//! - Reference: <https://renenyffenegger.ch/notes/development/Base64/Encoding-and-decoding-base-64-with-cpp>
//! - Reference: <https://dev.to/tiemen/implementing-base64-from-scratch-in-rust-kb1>

const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                       abcdefghijklmnopqrstuvwxyz\
                       0123456789+/";

/// Encodes a byte slice into a base64 string (with padding).
pub fn encode(inp: &[u8]) -> String {
    let end_len = (inp.len() + 2) / 3 * 4;
    let mut out = String::with_capacity(end_len);

    for i in (0..inp.len()).step_by(3) {
        out.push(CHARS[((inp[i] & 0xfc) >> 2) as usize] as char);

        if i + 1 < inp.len() {
            out.push(CHARS[(((inp[i] & 0x03) << 4) + ((inp[i + 1] & 0xf0) >> 4)) as usize] as char);

            if i + 2 < inp.len() {
                out.push(
                    CHARS[(((inp[i + 1] & 0x0f) << 2) + ((inp[i + 2] & 0xc0) >> 6)) as usize]
                        as char,
                );
                out.push(CHARS[(inp[i + 2] & 0x3f) as usize] as char);
                continue;
            }

            out.push(CHARS[((inp[i + 1] & 0x0f) << 2) as usize] as char);
            out.push('=');
            continue;
        }

        out.push(CHARS[((inp[i] & 0x03) << 4) as usize] as char);
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

    let out_size = (inp.len() / 4) * 3;
    let mut out = Vec::with_capacity(out_size);

    'o: for chunk in inp.as_bytes().chunks(4) {
        let mut decode = 0;

        for (i, e) in chunk.iter().enumerate() {
            match *e as char {
                'A'..='Z' => decode |= ((e - 65) as u32) << (6 * (3 - i)),
                'a'..='z' => decode |= ((e - 71) as u32) << (6 * (3 - i)),
                '0'..='9' => decode |= ((e + 4) as u32) << (6 * (3 - i)),
                '+' => decode |= 62 << (6 * i),
                '/' => decode |= 63 << (6 * i),
                '=' => {
                    out.extend_from_slice(&decode.to_be_bytes()[1..i]);
                    continue 'o;
                }
                _ => return None,
            }
        }

        out.extend_from_slice(&decode.to_be_bytes()[1..4]);
    }

    Some(out)
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
