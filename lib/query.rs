use crate::common;
use std::fmt;

/// Struct for holding query data
pub struct Query {
    pub(crate) data: Vec<[String; 2]>,
}

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
        T: fmt::Display,
    {
        // Remove the leading '?' if it exists
        let mut body = query.to_string();
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

        Some(Query { data })
    }

    /// Create a new blank query
    /// # Example
    /// ```
    /// use afire::Query;
    /// let query = Query::new_empty();
    /// ```
    pub fn new_empty() -> Query {
        Query { data: Vec::new() }
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
        T: fmt::Display,
    {
        let key = key.to_string();

        for i in self.data.clone() {
            if *i.first()? == key {
                return Some(i[1].clone());
            }
        }
        None
    }
}

// Impl Clone for Query
impl Clone for Query {
    fn clone(&self) -> Query {
        Query {
            data: self.data.clone(),
        }
    }
}

// Implement fmt::Display for Query
impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::from("?");
        for i in self.data.clone() {
            output.push_str(&format!("{}={}&", i[0], i[1]));
        }
        output.pop();
        write!(f, "{}", output)
    }
}

/// Implement the fmt::Display trait for Query
impl fmt::Debug for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Query").field("data", &self.data).finish()
    }
}
