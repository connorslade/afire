//! A router for matching request paths against routes.

use std::{
    fmt::{self, Display},
    ops::Range,
    sync::Arc,
};

use crate::error::{PathError, Result, StartupError};

/// A route path.
/// There are 5 types of segments that can be used to make up a path:
/// - Literal: A literal string that must match exactly.
/// - Parameter: A named parameter that can be matched to any string before the next separator, including empty strings.
/// - Any: A wildcard that matches any string before the next separator, including empty strings. Just like a parameter, but does not capture a value.
/// - AnyAfter: A wildcard that matches anything after the current segment. Must only ever be used as the last segment.
/// - Separator: A separator that must be matched exactly (/).
///
/// ## Examples
/// | Route           | Explanation                                                                                                              |
/// | --------------- | ------------------------------------------------------------------------------------------------------------------------ |
/// | `/hello/world`  | Matches `/hello/world` exactly. Will not match if path has a trailing separator.                                         |
/// | `/hello/world/` | Matches `/hello/world/`, will not match `/hello/world`                                                                   |
/// | `/hello/{name}` | Matches `/hello/` followed by any string before a separator. Will match `/hello/darren` and `/hello/` but not `/hello/`. |
/// | `/hello/*`      | Matches `/hello/` followed by any string. Will match `/hello/world` but not `/hello/world/test` or `/hello/world/`.      |
/// | `/**`           | Matches any path. This is useful for 404 pages.                                                                          |
#[derive(Debug)]
pub struct Path {
    segments: Vec<Segment>,
    parameters: Arc<[String]>,
}

/// A segment of a path.
#[derive(Debug)]
enum Segment {
    /// A separator that must be matched exactly (/).
    Separator,
    /// A literal string that must match exactly.
    Literal(String),
    /// A named parameter that can be matched to any string before the next separator, including empty strings.
    Parameter,
    /// A wildcard that matches any string before the next separator, including empty strings.
    /// Just like a parameter, but does not capture a value.
    Any,
    /// A wildcard that matches anything after the current segment.
    /// Must only ever be used as the last segment.
    AnyAfter,
}

/// A container for path parameters.
/// Created when a path matches a route with [`Path::matches`].
/// Values are automatically url-decoded.
#[derive(Debug, PartialEq, Eq)]
pub struct PathParameters {
    params: Vec<Range<usize>>,
    names: Arc<[String]>,
}

impl Path {
    /// Parse a raw path string into a `Path`.
    pub fn new(path: &str) -> Result<Path> {
        let mut tokenizer = tokenizer::Tokenizer::new(path);
        if let Err(e) = tokenizer.tokenize() {
            return Err(StartupError::Path {
                error: e,
                route: path.into(),
            }
            .into());
        };

        let path = tokenizer.into_path();

        let mut last_special = false;
        for i in &path.segments {
            if last_special && i.disallow_adjacent() {
                return Err(StartupError::Path {
                    error: PathError::AmbiguousPath,
                    route: path.to_string(),
                }
                .into());
            }

            last_special = i.disallow_adjacent();
        }

        Ok(path)
    }

    /// Try to match a path against this route.
    /// Returns `None` if the path does not match.
    /// Returns `Some` with a `PathParameters` if the path matches.
    pub fn matches<'a>(&'a self, path: &'a str) -> Option<PathParameters> {
        let mut matcher = matcher::Matcher::new(self, path);
        let names = self.parameters.clone();
        matcher
            .matches()
            .map(|params| PathParameters { params, names })
    }
}

impl Segment {
    fn disallow_adjacent(&self) -> bool {
        matches!(self, Segment::Any | Segment::AnyAfter | Segment::Parameter)
    }
}

impl PathParameters {
    /// Get a parameter by name.
    /// - Key is the name of the parameter (case sensitive).
    /// - Path is the path that was matched. It is used to get a reference to the parameter value.
    pub(crate) fn get<'a>(&self, key: &str, path: &'a str) -> Option<&'a str> {
        let index = self.names.iter().position(|i| i == key)?;
        let range = self.params[index].clone();
        Some(&path[range])
    }

    pub(crate) fn get_index<'a>(&self, index: usize, path: &'a str) -> Option<&'a str> {
        let range = self.params[index].clone();
        Some(&path[range])
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut pi = 0;
        for i in &self.segments {
            match i {
                Segment::Separator => f.write_str("/")?,
                Segment::Literal(l) => f.write_str(l)?,
                Segment::Parameter => {
                    f.write_fmt(format_args!("{{{}}}", self.parameters[pi]))?;
                    pi += 1
                }
                Segment::Any => f.write_str("*")?,
                Segment::AnyAfter => f.write_str("**")?,
            }
        }
        Ok(())
    }
}

mod matcher {
    use std::ops::Range;

    use super::{Path, Segment};

    pub struct Matcher<'a> {
        segments: &'a [Segment],
        path: Vec<char>,

        seg_index: usize,
        path_index: usize,
        params: Option<Vec<Range<usize>>>,
    }

    impl<'a> Matcher<'a> {
        pub(super) fn new(route: &'a Path, path: &'a str) -> Matcher<'a> {
            Matcher {
                segments: &route.segments,
                path: path.chars().collect(),

                seg_index: 0,
                path_index: 0,
                params: Some(Vec::with_capacity(route.parameters.len())),
            }
        }

        pub fn matches(&mut self) -> Option<Vec<Range<usize>>> {
            while self.seg_index < self.segments.len() {
                match &self.segments[self.seg_index] {
                    Segment::AnyAfter => return Some(self.params.take().unwrap()),
                    Segment::Literal(l) => self.take_str(l)?,
                    Segment::Any => self.match_any()?,
                    Segment::Parameter => self.match_parameter()?,
                    Segment::Separator => self.take('/')?,
                }

                self.seg_index += 1;
            }

            if self.path_index >= self.path.len() {
                return Some(self.params.take().unwrap());
            }

            None
        }
    }

    impl<'a> Matcher<'a> {
        fn take(&mut self, value: char) -> Option<()> {
            if *self.path.get(self.path_index)? != value {
                return None;
            }

            self.path_index += 1;
            Some(())
        }

        fn take_str(&mut self, values: &str) -> Option<()> {
            let mut values = values.chars();

            while self.path_index < self.path.len() {
                let Some(value) = values.next() else {
                    return Some(());
                };

                if self.path[self.path_index] != value {
                    return None;
                }

                self.path_index += 1;
            }

            values.next().is_none().then_some(())
        }

        fn next_separator(&self) -> usize {
            self.path
                .iter()
                .skip(self.path_index)
                .position(|i| *i == '/')
                .map(|i| i + self.path_index)
                .unwrap_or(self.path.len())
        }

        // Find last occurrence of next literal without passing any separators.
        fn match_around(&mut self) -> Option<Range<usize>> {
            let end_pos = match self.segments.get(self.seg_index + 1) {
                Some(Segment::Literal(l)) => {
                    let chars = l.chars().collect::<Vec<_>>();
                    let mut i = self.next_separator().saturating_sub(l.len());
                    let mut found = false;

                    // GE - Matches 0-length any / parameter segments
                    // G  - Does not match 0-length any / parameter segments
                    while i >= self.path_index {
                        if self.path[i..i + l.len()] != chars {
                            i -= 1;
                            continue;
                        }

                        found = true;
                        break;
                    }

                    if !found {
                        return None;
                    }

                    i
                }
                None | Some(Segment::Separator) => self.next_separator(),
                Some(Segment::Any | Segment::AnyAfter | Segment::Parameter) => unreachable!(),
            };

            Some(self.path_index..end_pos)
        }

        fn match_any(&mut self) -> Option<()> {
            let range = self.match_around()?;
            self.path_index = range.end;
            Some(())
        }

        fn match_parameter(&mut self) -> Option<()> {
            let range = self.match_around()?;
            self.path_index = range.end;
            self.params.as_mut().unwrap().push(range);
            Some(())
        }
    }
}

mod tokenizer {
    use crate::error::PathError;

    use super::{Path, Segment};

    pub(super) struct Tokenizer {
        chars: Vec<char>,
        index: usize,

        pub tokens: Vec<Segment>,
        pub parameters: Vec<String>,
        buffer: String,
    }

    impl Tokenizer {
        pub fn new(path: &str) -> Tokenizer {
            Tokenizer {
                chars: path.chars().collect(),
                index: 0,

                tokens: Vec::new(),
                parameters: Vec::new(),
                buffer: String::new(),
            }
        }

        pub fn tokenize(&mut self) -> Result<(), PathError> {
            while self.index < self.chars.len() {
                let chr = self.next();
                match chr {
                    '/' => {
                        self.flush_buffer();
                        self.tokens.push(Segment::Separator);
                    }
                    '{' => {
                        self.flush_buffer();
                        self.parse_parameter()?;
                    }
                    '*' if self.peek() == Some(&'*') => {
                        self.flush_buffer();
                        self.tokens.push(Segment::AnyAfter);
                        self.index += 1;

                        if self.index < self.chars.len() {
                            return Err(PathError::NonTerminatingAnyAfter);
                        }

                        return Ok(());
                    }
                    '*' => {
                        self.flush_buffer();
                        self.tokens.push(Segment::Any);
                    }
                    _ => {
                        self.buffer.push(chr);
                    }
                }
            }

            self.flush_buffer();
            Ok(())
        }

        pub fn into_path(self) -> Path {
            Path {
                segments: self.tokens,
                parameters: self.parameters.into(),
            }
        }
    }

    impl Tokenizer {
        fn next(&mut self) -> char {
            let chr = self.chars[self.index];
            self.index += 1;
            chr
        }

        fn peek(&self) -> Option<&char> {
            self.chars.get(self.index)
        }

        fn flush_buffer(&mut self) {
            if self.buffer.is_empty() {
                return;
            }

            self.tokens.push(Segment::Literal(self.buffer.clone()));
            self.buffer.clear();
        }

        fn parse_parameter(&mut self) -> Result<(), PathError> {
            while self.index < self.chars.len() {
                let chr = self.next();
                match chr {
                    '{' => {
                        return Err(PathError::NestedParameter);
                    }
                    '}' => {
                        self.tokens.push(Segment::Parameter);
                        self.parameters.push(self.buffer.clone());
                        self.buffer.clear();
                        return Ok(());
                    }
                    _ => {
                        self.buffer.push(chr);
                    }
                }
            }

            Err(PathError::UnterminatedParameter)
        }
    }
}

#[cfg(test)]
mod test {
    use super::Path;
    use std::collections::HashMap;

    macro_rules! match_result {
        [] => {
            HashMap::<String, String>::new()
        };
        [$($key:tt => $val:tt),*] => {
            {
                let mut map = HashMap::<String, String>::new();
                $(
                    map.insert($key.to_string(), $val.to_string());
                )*
                map
            }
        };
    }

    macro_rules! match_tests {
        {$(#[test($test_name:ident)] $(#[$meta:meta])* $($path:literal => [$($test:literal => $result:expr),+]),+),*} => {
            $(
                #[test]
                $(#[$meta])*
                fn $test_name() {
                    $(
                        let path = Path::new($path).unwrap();
                        $(
                            let res = path.matches($test);
                            println!("CASE: `{}`.matches(`{}`)", $path, $test);
                            let result: Option<HashMap<String, String>> = $result;
                            match (&res, &result) {
                                (None, None) => println!(" \\ PASSED"),
                                (Some(res), Some(result)) => {
                                    for (key, val) in result {
                                        assert_eq!(
                                            res.get(key, $test)
                                                .unwrap_or_else(|| panic!(" \\ Failed: Argument `{}` was not captured.", key)),
                                            val,
                                            " \\ Failed: Argument `{}` did not match.",
                                            key
                                        );
                                    }
                                    println!(" \\ PASSED")
                                },
                                (a, b) => panic!(" \\ FAILED: {:?} != {:?}", a, b)
                            }
                        )*
                    )+
                }
            )*
        };
    }

    /* spellchecker: disable */
    match_tests! {
        #[test(basic_1)]
        "/" => [
            "/"  => Some(match_result![]),
            "/a" => None,
            ""   => None
        ],
        #[test(basic_2)]
        "/send-2" => [
            "/send-2"  => Some(match_result![]),
            "/send-2/" => None,
            "/"        => None
        ],
        #[test(parameters_1)]
        "/hello{world}world/E" => [
            "/hellopeople!worldworld/E" => Some(match_result!["world" => "people!world"]),
            "/helloworld/E"             => Some(match_result!["world" => ""])
        ],
        #[test(parameters_2)]
        "/file/{name}.{ext}" => [
            "/file/hello.txt"  => Some(match_result!["name" => "hello", "ext" => "txt"]),
            "/file/hello"      => None,
            "/file/hello.txt/" => None
        ],
        #[test(parameters_2_trailing)]
        "/file/{name}.{ext}/" => [
            "/file/hello.txt"  => None,
            "/file/hello"      => None,
            "/file/hello.txt/" => Some(match_result!["name" => "hello", "ext" => "txt"])
        ],
        #[test(parameters_3)]
        "/api/get/{name}" => [
            "/api/get/john" => Some(match_result!["name" => "john"]),
            "/api/get/"     => Some(match_result!["name" => ""]),
            "/api/get"      => None
        ],
        #[test(wildcard_1)]
        "/hello*!" => [
            "/hello!"       => Some(match_result![]),
            "/hello"        => None,
            "/helloworld!"  => Some(match_result![]),
            "/helloworld"   => None,
            "/hello/world!" => None
        ],
        #[test(wildcard_2)]
        "/hello/*" => [
            "/hello/"      => Some(match_result![]),
            "/hello"       => None,
            "/hello/world" => Some(match_result![])
        ],
        #[test(any_after_1)]
        "/hello/**" => [
            "/hello"         => None,
            "/hello/"        => Some(match_result![]),
            "/hello/world"   => Some(match_result![]),
            "/hello/world/"  => Some(match_result![]),
            "/hello/world/!" => Some(match_result![])
        ]
    }
    /* spellchecker: enable */

    #[test]
    fn test_path_display() {
        const CASES: &[&str] = &[
            "/",
            "/hello",
            "/hello/world",
            "/hello/{name}",
            "/hello/{name}/",
        ];

        for i in CASES {
            let path = Path::new(i).unwrap();
            assert_eq!(path.to_string(), *i);
        }
    }
}
