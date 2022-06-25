use crate::common;
use std::fmt;

/// Struct for holding query data
#[derive(Hash, PartialEq, Eq, Clone)]
pub struct Query(pub Vec<[String; 2]>);

/// Implementation for Query
impl Query {
    /// Create a new query from a string
    /// # Example
    /// ```
    /// use afire::Query;
    /// let query = Query::new("?foo=bar&nose=dog");
    /// ```
    pub fn new<T>(query: T) -> Option<Query>
    where
        T: AsRef<str>,
    {
        // Remove the leading '?' if it exists
        let mut body = query.as_ref().to_owned();
        if body.starts_with('?') {
            body = body.split_off(1);
        }

        // Decode the query string and split it into key/value pairs
        Query::from_body(common::decode_url(body))
    }

    /// Create a new Query from a Form POST body
    /// # Example
    /// ```
    /// use afire::Query;
    /// let query = Query::from_body("foo=bar&nose=dog".to_string());
    /// ```
    pub fn from_body(body: String) -> Option<Query> {
        let mut data = Vec::new();

        let parts = body.split('&');
        for i in parts {
            let mut sub = i.splitn(2, '=');
            if sub.clone().count() < 2 {
                continue;
            }

            let key: String = sub.next()?.to_string();
            let value: String = sub.next()?.to_string();

            data.push([key, value])
        }

        Some(Query(data))
    }

    /// Create a new blank query
    /// # Example
    /// ```
    /// use afire::Query;
    /// let query = Query::new_empty();
    /// ```
    pub fn new_empty() -> Query {
        Query(Vec::new())
    }

    /// Get a value from a key
    /// # Example
    /// ```
    /// use afire::Query;
    /// let query = Query::new("?foo=bar&nose=dog").unwrap();
    ///
    /// assert_eq!(query.get("foo"), Some("bar".to_string()));
    /// ```
    pub fn get<T>(&self, key: T) -> Option<String>
    where
        T: AsRef<str>,
    {
        let key = key.as_ref().to_owned();

        for i in self.0.clone() {
            if *i.first()? == key {
                return Some(i[1].clone());
            }
        }
        None
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

/// Implement the fmt::Display trait for Query
impl fmt::Debug for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Query").field("data", &self.0).finish()
    }
}
