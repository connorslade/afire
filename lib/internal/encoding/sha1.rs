//! SHA1 hash function.
//! NOTE: This is not a secure hash function, and should not be used unless necessary.

/// SHA1 hash function.
/// NOTE: This is not a secure hash function, and should not be used unless necessary.
pub fn hash(message: &[u8]) -> [u8; 20] {
    let mut h0 = 0x67452301u32;
    let mut h1 = 0xefcdab89u32;
    let mut h2 = 0x98badcfeu32;
    let mut h3 = 0x10325476u32;
    let mut h4 = 0xc3d2e1f0u32;

    let mut padded_message = vec![0u8; message.len() + 64 - message.len() % 64];
    padded_message[..message.len()].copy_from_slice(message);
    padded_message[message.len()] = 0x80;

    let bit_len = (message.len() as u64) * 8;
    let msg_len = padded_message.len();
    padded_message[msg_len - 8..].copy_from_slice(&bit_len.to_be_bytes());

    for chunk in padded_message.chunks(64) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..80 {
            w[i] = w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16];
            w[i] = w[i].rotate_left(1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        for i in 0..80 {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5a827999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1u32),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1bbcdcu32),
                60..=79 => (b ^ c ^ d, 0xca62c1d6u32),
                _ => unreachable!(),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let mut out = [0u8; 20];
    out[0..4].copy_from_slice(&h0.to_be_bytes());
    out[4..8].copy_from_slice(&h1.to_be_bytes());
    out[8..12].copy_from_slice(&h2.to_be_bytes());
    out[12..16].copy_from_slice(&h3.to_be_bytes());
    out[16..20].copy_from_slice(&h4.to_be_bytes());

    out
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hash() {
        assert_eq!(
            hash(b"hello world"),
            [
                42, 174, 108, 53, 201, 79, 207, 180, 21, 219, 233, 95, 64, 139, 156, 233, 30, 232,
                70, 237
            ]
        );
    }

    #[test]
    fn test_hash_empty() {
        assert_eq!(
            hash(b""),
            [
                218, 57, 163, 238, 94, 107, 75, 13, 50, 85, 191, 239, 149, 96, 24, 144, 175, 216,
                7, 9
            ]
        );
    }
}
