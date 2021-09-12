/// Compares two Vectors
pub(crate) fn cmp_vec<T: std::cmp::PartialEq>(vec: &[T], vec2: &[T]) -> bool {
    if vec.len() != vec2.len() {
        return false;
    }

    for i in 0..vec.len() {
        if vec[i] != vec2[i] {
            return false;
        }
    }
    true
}

/// Decode a url encoded string
pub fn decode_url(url: String) -> String {
    // Convert input to Char array
    let url = url.chars().collect::<Vec<char>>();

    let mut res = String::new();
    let mut i = 0;
    while i < url.len() {
        if url[i] == '%' {
            let mut hex = String::new();
            try_push(&mut hex, url.get(i + 1));
            try_push(&mut hex, url.get(i + 2));
            res.push(u8::from_str_radix(&hex, 16).unwrap_or_default() as char);
            i += 3;
            continue;
        }
        try_push(&mut res, url.get(i));
        i += 1;
    }
    res
}

fn try_push(vec: &mut String, c: Option<&char>) {
    if let Some(c) = c {
        vec.push(*c);
    }
}
