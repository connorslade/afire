use std::{
    fmt,
    ops::{Deref, DerefMut},
};

/// Collection of query parameters.
/// Can be made from the query string of a URL, or the body of a POST request.
/// Similar to [`crate::headers::Headers`].
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Query(Vec<[String; 2]>);

impl Deref for Query {
    type Target = Vec<[String; 2]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Query {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Query {
    /// Checks if the specified key exists in the query.
    /// # Example
    /// ```rust
    /// # use afire::Query;
    /// # use std::str::FromStr;
    /// # let query = Query::from_body("foo=bar&nose=dog");
    /// # assert!(query.has("foo"));
    /// if query.has("foo") {
    ///    println!("foo exists");
    /// }
    /// ```
    pub fn has(&self, key: impl AsRef<str>) -> bool {
        let key = key.as_ref().to_owned();
        self.iter().any(|i| *i[0] == key)
    }

    /// Get a value from a key.
    /// This will return None if the key does not exist.
    /// # Example
    /// ```
    /// # use afire::Query;
    /// # use std::str::FromStr;
    /// let query = Query::from_body("foo=bar&nose=dog");
    /// assert_eq!(query.get("foo"), Some("bar"));
    /// ```
    pub fn get(&self, key: impl AsRef<str>) -> Option<&str> {
        let key = key.as_ref().to_owned();
        self.iter()
            .find(|i| *i[0] == key)
            .map(|i| &i[1])
            .map(|x| x.as_str())
    }

    /// Gets a value of the specified key as a mutable reference.
    /// This will return None if the key does not exist.
    /// See [`Query::get`] for the non-mutable version.
    pub fn get_mut(&mut self, key: impl AsRef<str>) -> Option<&mut String> {
        let key = key.as_ref().to_owned();
        self.iter_mut().find(|i| *i[0] == key).map(|i| &mut i[1])
    }

    /// Get the key-value pair of the specified key as a mutable reference.
    /// If the key does not exist, this will return None.
    pub fn get_query_mut(&mut self, key: impl AsRef<str>) -> Option<&mut [String; 2]> {
        let key = key.as_ref().to_owned();
        self.iter_mut().find(|i| *i[0] == key)
    }

    /// Create a new Query from a Form POST body
    /// # Example
    /// ```
    /// # use afire::Query;
    /// let query = Query::from_body("foo=bar&nose=dog");
    /// ```
    pub fn from_body(body: &str) -> Self {
        let mut data = Vec::new();

        let parts = body.split('&');
        for i in parts {
            let mut sub = i.splitn(2, '=');

            let key = match sub.next() {
                Some(i) => i.to_owned(),
                None => continue,
            };

            let value = match sub.next() {
                Some(i) => i.to_owned(),
                None => continue,
            };

            data.push([key, value])
        }

        Query(data)
    }
}

// Implement fmt::Display for Query
impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::from("?");
        for i in self.0.clone() {
            output.push_str(&format!("{}={}&", i[0], i[1]));
        }
        output.pop();
        write!(f, "{}", output)
    }
}

#[cfg(test)]
mod test {
    use super::Query;

    #[test]
    fn test_from_str() {
        let query = Query::from_body("foo=bar&nose=dog");
        assert_eq!(query.get("foo"), Some("bar"));
        assert_eq!(query.get("nose"), Some("dog"));
        assert_eq!(query.get("bar"), None);
    }

    #[test]
    fn test_get() {
        let query = Query::from_body("foo=bar&nose=dog");
        assert_eq!(query.get("foo"), Some("bar"));
        assert_eq!(query.get("nose"), Some("dog"));
        assert_eq!(query.get("bar"), None);
    }

    #[test]
    fn test_get_mut() {
        let mut query = Query::from_body("foo=bar&nose=dog");
        assert_eq!(query.get_mut("bar"), None);
        query.get_mut("foo").unwrap().push_str("bar");
        assert_eq!(query.get("foo"), Some("barbar"));
    }
}
