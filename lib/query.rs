use std::{fmt, str::FromStr};

/// Struct for holding query data, from request paths and form posts.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Query(pub Vec<[String; 2]>);

/// Implementation for Query
impl Query {
    /// Get a value from a key.
    /// This will return None if the key does not exist.
    /// # Example
    /// ```
    /// # use afire::Query;
    /// # use std::str::FromStr;
    /// let query = Query::from_str("foo=bar&nose=dog").unwrap();
    /// assert_eq!(query.get("foo"), Some("bar"));
    /// ```
    pub fn get<T>(&self, key: T) -> Option<&str>
    where
        T: AsRef<str>,
    {
        let key = key.as_ref().to_owned();

        for i in &self.0 {
            if *i.first()? == key {
                return Some(&i[1]);
            }
        }
        None
    }
}

impl FromStr for Query {
    type Err = ();

    /// Create a new Query from a Form POST body
    /// # Example
    /// ```
    /// # use afire::Query;
    /// # use std::str::FromStr;
    /// let query = Query::from_str("foo=bar&nose=dog");
    /// ```
    fn from_str(body: &str) -> Result<Self, Self::Err> {
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

        Ok(Query(data))
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
