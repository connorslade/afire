//! Normalizes paths to a common format by removing trailing slashes and repeated slashes.

use crate::{
    middleware::{MiddleResult, Middleware},
    Request,
};

/// Normalizes paths to a common format by removing trailing slashes and repeated slashes.
/// ## Example
/// ```plain
/// `/hello/world/` -> `/hello/world`
/// `/hello//world` -> `/hello/world`
/// ```
pub struct PathNormalizer;

impl Middleware for PathNormalizer {
    fn pre(&self, req: &mut Request) -> MiddleResult {
        normalize(&mut req.path);
        MiddleResult::Continue
    }
}

fn normalize(path: &mut String) {
    let mut new_path = String::with_capacity(path.len());

    let mut last_char = '\0';
    for i in path.chars() {
        if i == '/' && last_char == '/' {
            continue;
        }

        new_path.push(i);
        last_char = i;
    }

    if new_path.ends_with('/') {
        new_path.pop();
    }

    *path = new_path;
}

#[cfg(test)]
mod test {
    use super::normalize;

    #[test]
    fn test_path_normalizer() {
        let mut path = "/hello/world/".to_string();
        normalize(&mut path);
        assert_eq!(path, "/hello/world");

        let mut path = "/hello//world/".to_string();
        normalize(&mut path);
        assert_eq!(path, "/hello/world");
    }
}
