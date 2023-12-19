//! Query parameters for HTTP requests.

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use crate::internal::encoding::url;

/// Collection of query parameters.
/// Can be made from the query string of a URL, or the body of a POST request.
/// Similar to [`crate::header::Headers`].
#[derive(Hash, PartialEq, Eq, Clone)]
pub struct Query {
    inner: Vec<QueryParameter>,
}

/// An individual query parameter.
/// Key and value are both automatically url decoded.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct QueryParameter {
    /// The key of the query parameter.
    pub key: String,
    /// The value of the query parameter.
    pub value: String,
}

impl Query {
    /// Checks if the specified key exists in the query.
    /// ## Example
    /// ```rust
    /// # use afire::Query;
    /// # use std::str::FromStr;
    /// # let query = Query::from_str("foo=bar&nose=dog");
    /// # assert!(query.has("foo"));
    /// if query.has("foo") {
    ///    println!("foo exists");
    /// }
    /// ```
    pub fn has(&self, key: impl AsRef<str>) -> bool {
        let key = key.as_ref();
        self.inner.iter().any(|i| i.key == key)
    }

    /// Adds a new key-value pair to the collection with the specified key and value.
    /// See [`Query::add_query`] for adding a key-value pair with a `[String; 2]`.
    /// ## Example
    /// ```rust
    /// # use afire::Query;
    /// # use std::str::FromStr;
    /// # fn test(query: &mut Query) {
    /// query.add("foo", "bar");
    /// # }
    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.inner.push(QueryParameter {
            key: key.into(),
            value: value.into(),
        });
    }

    /// Get a value from a key.
    /// This will return None if the key does not exist.
    /// ## Example
    /// ```
    /// # use afire::Query;
    /// # use std::str::FromStr;
    /// let query = Query::from_str("foo=bar&nose=dog");
    /// assert_eq!(query.get("foo"), Some("bar"));
    /// ```
    pub fn get(&self, key: impl AsRef<str>) -> Option<&str> {
        let key = key.as_ref();
        self.inner
            .iter()
            .find(|i| i.key == key)
            .map(|i| i.value.as_ref())
    }

    /// Gets a value of the specified key as a mutable reference.
    /// This will return None if the key does not exist.
    /// See [`Query::get`] for the non-mutable version.
    pub fn get_mut(&mut self, key: impl AsRef<str>) -> Option<&mut String> {
        let key = key.as_ref();
        self.inner
            .iter_mut()
            .find(|i| i.key == key)
            .map(|i| &mut i.value)
    }

    /// Adds a new parameter to the collection with a QueryParameter struct.
    /// See [`Query::add`] for adding a key-value pair with string keys and values.
    /// ## Example
    /// ```rust
    /// # use afire::proto::http::query::{QueryParameter, Query};
    /// # fn test(query: &mut Query) {
    /// query.add_query(QueryParameter {
    ///     key: "foo".into(),
    ///     value: "bar".into(),
    /// });
    /// # }
    pub fn add_query(&mut self, query: QueryParameter) {
        self.inner.push(query);
    }

    /// Gets the key-value pair of the specified key.
    /// If the key does not exist, this will return None.
    pub fn get_query(&self, key: impl AsRef<str>) -> Option<&QueryParameter> {
        let key = key.as_ref();
        self.inner.iter().find(|i| i.key == key)
    }

    /// Get the key-value pair of the specified key as a mutable reference.
    /// If the key does not exist, this will return None.
    pub fn get_query_mut(&mut self, key: impl AsRef<str>) -> Option<&mut QueryParameter> {
        let key = key.as_ref();
        self.inner.iter_mut().find(|i| i.key == key)
    }

    /// Create a new Query from a Form POST body
    /// ## Example
    /// ```
    /// # use afire::Query;
    /// let query = Query::from_str("foo=bar&nose=dog");
    /// ```
    pub fn from_str(body: &str) -> Self {
        let mut data = Vec::new();

        for i in body.split('&') {
            let mut sub = i.splitn(2, '=');

            let Some(key) = sub.next().map(url::decode) else {
                continue;
            };

            let Some(value) = sub.next().map(url::decode) else {
                continue;
            };

            data.push(QueryParameter {
                key: key.into(),
                value: value.into(),
            });
        }

        Query { inner: data }
    }
}

impl Deref for Query {
    type Target = Vec<QueryParameter>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Query {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_empty() {
            return f.write_str("");
        }

        let mut output = String::from("?");
        for i in &self.inner {
            output.push_str(&format!("{}={}&", i.key, i.value));
        }

        f.write_str(&output[..output.len() - 1])
    }
}

impl fmt::Debug for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Query").field("inner", &self.inner).finish()
    }
}

#[cfg(test)]
mod test {
    use super::Query;

    #[test]
    fn test_from_str() {
        let query = Query::from_str("foo=bar&nose=dog");
        assert_eq!(query.get("foo"), Some("bar"));
        assert_eq!(query.get("nose"), Some("dog"));
        assert_eq!(query.get("bar"), None);
    }

    #[test]
    fn test_get() {
        let query = Query::from_str("foo=bar&nose=dog");
        assert_eq!(query.get("foo"), Some("bar"));
        assert_eq!(query.get("nose"), Some("dog"));
        assert_eq!(query.get("bar"), None);
    }

    #[test]
    fn test_get_mut() {
        let mut query = Query::from_str("foo=bar&nose=dog");
        assert_eq!(query.get_mut("bar"), None);
        query.get_mut("foo").unwrap().push_str("bar");
        assert_eq!(query.get("foo"), Some("barbar"));
    }
}
