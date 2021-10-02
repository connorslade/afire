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
    pub fn new(query: &str) -> Query {
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
    pub fn from_body(body: String) -> Query {
        let mut data = Vec::new();

        let parts: Vec<&str> = body.split('&').collect();
        for i in parts {
            let sub: Vec<&str> = i.split('=').collect();
            if sub.len() < 2 {
                continue;
            }

            let key: String = sub[0].to_string();
            let value: String = sub[1].to_string();

            data.push([key, value])
        }

        Query { data }
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
    /// let query = Query::new("?foo=bar&nose=dog");
    ///
    /// assert_eq!(query.get("foo"), Some("bar".to_string()));
    /// ```
    pub fn get(&self, key: &str) -> Option<String> {
        for i in self.data.clone() {
            if i[0] == key {
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
