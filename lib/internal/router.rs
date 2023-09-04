//! A router for matching request paths against routes.
//! To find the route that should process a request, all added routes are checked in reverse order.
//! The first route that matches the request path is used.
//!
//!There are 5 types of segments that can be used to make up a path.
//! Note that 'Any string' includes empty strings.
//!
//!| Name      | Syntax | Description                                                                                        |
//!| --------- | :----: | -------------------------------------------------------------------------------------------------- |
//!| Separator | `/`    | Matches the slash character.                                                                       |
//!| Literal   | `...`  | A literal string that must match exactly.                                                          |
//!| Parameter | `{...}`| A named parameter that can be matched to any string before the next separator.                     |
//!| Wildcard  | `*`    | Matches any string before the next separator. Just like a parameter, but does not capture a value. |
//!| AnyAfter  | `**`   | Matches the rest of the path, regardless of its contents.                                          |
//!
//! ## Examples
//!| Route             | Explanation                                                                                                              |
//!| ----------------- | ------------------------------------------------------------------------------------------------------------------------ |
//!| `/hello/world`    | Matches `/hello/world` exactly. Will not match if path has a trailing slash.                                         |
//!| `/hello/world/`   | Matches `/hello/world/`, will not match `/hello/world`                                                                   |
//!| `/hello/{name}`   | Matches `/hello/` followed by any string before a separator. Will match `/hello/darren` and `/hello/` but not `/hello/`. |
//!| `/hello/*`        | Matches `/hello/` followed by any string. Will match `/hello/world` but not `/hello/world/test` or `/hello/world/`.      |
//!| `/hello/**/world` | Matches `/hello/` followed by anything, followed by `/world`. Will match `/hello/everybody/world`.                       |
//!| `/**`             | Matches any path. This is useful for 404 pages.    

use std::{
    fmt::{self, Display},
    ops::Range,
    sync::Arc,
};

use crate::error::{PathError, Result, StartupError};

/// A parsed route path.
/// String path can be matched against it with [`Path::matches`].
#[derive(Debug)]
pub struct Path {
    segments: Box<[Segment]>,
    parameters: Arc<[String]>,
}

/// A segment of a path.
#[derive(Debug)]
enum Segment {
    /// A separator that must be matched exactly (/).
    Separator,
    /// A literal string that must match exactly.
    Literal(Box<str>),
    /// A named parameter that can be matched to any string before the next separator, including empty strings.
    Parameter,
    /// A wildcard that matches any string before the next separator, including empty strings.
    /// Just like a parameter, but does not capture a value.
    Wildcard,
    /// Matches the rest of the path, regardless of its contents.
    AnyAfter,
}

/// A container for path parameters.
/// Because the parameters are stored in a slice, it also hold the names of each parameter if the user uses [`crate::Context::param`].
#[derive(Debug, PartialEq, Eq)]
pub struct PathParameters {
    params: Box<[Range<usize>]>,
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
        for i in path.segments.iter() {
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
        matcher.matches().map(|params| PathParameters {
            params: params.into(),
            names,
        })
    }
}

impl Segment {
    fn disallow_adjacent(&self) -> bool {
        matches!(
            self,
            Segment::Wildcard | Segment::AnyAfter | Segment::Parameter
        )
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
        for i in self.segments.iter() {
            match i {
                Segment::Separator => f.write_str("/")?,
                Segment::Literal(l) => f.write_str(l)?,
                Segment::Parameter => {
                    f.write_fmt(format_args!("{{{}}}", self.parameters[pi]))?;
                    pi += 1
                }
                Segment::Wildcard => f.write_str("*")?,
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
                    Segment::Literal(l) => self.take_str(l)?,
                    Segment::Separator => self.take('/')?,
                    Segment::Wildcard => self.match_wildcard()?,
                    Segment::Parameter => self.match_parameter()?,
                    Segment::AnyAfter => return Some(self.params.take().unwrap()),
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

        fn match_wildcard(&mut self) -> Option<()> {
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
                Some(Segment::Wildcard | Segment::AnyAfter | Segment::Parameter) => unreachable!(),
            };

            Some(self.path_index..end_pos)
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
                    }
                    '*' => {
                        self.flush_buffer();
                        self.tokens.push(Segment::Wildcard);
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
                segments: self.tokens.into(),
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

            let val = self.buffer.as_str().into();
            self.tokens.push(Segment::Literal(val));
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

    macro_rules! result {
        [] => {
            Some(HashMap::<String, String>::new())
        };
        [$($key:tt => $val:tt),*] => {
            {
                let mut map = HashMap::<String, String>::new();
                $(
                    map.insert($key.to_string(), $val.to_string());
                )*
                Some(map)
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
            "/"  => result![],
            "/a" => None,
            ""   => None
        ],
        #[test(basic_2)]
        "/send-2" => [
            "/send-2"  => result![],
            "/send-2/" => None,
            "/"        => None
        ],
        #[test(parameters_1)]
        "/hello{world}world/E" => [
            "/hellopeople!worldworld/E" => result!["world" => "people!world"],
            "/helloworld/E"             => result!["world" => ""]
        ],
        #[test(parameters_2)]
        "/file/{name}.{ext}" => [
            "/file/hello.txt"  => result!["name" => "hello", "ext" => "txt"],
            "/file/hello"      => None,
            "/file/hello.txt/" => None
        ],
        #[test(parameters_2_trailing)]
        "/file/{name}.{ext}/" => [
            "/file/hello.txt"  => None,
            "/file/hello"      => None,
            "/file/hello.txt/" => result!["name" => "hello", "ext" => "txt"]
        ],
        #[test(parameters_3)]
        "/api/get/{name}" => [
            "/api/get/john" => result!["name" => "john"],
            "/api/get/"     => result!["name" => ""],
            "/api/get"      => None
        ],
        #[test(wildcard_1)]
        "/hello*!" => [
            "/hello!"       => result![],
            "/hello"        => None,
            "/helloworld!"  => result![],
            "/helloworld"   => None,
            "/hello/world!" => None
        ],
        #[test(wildcard_2)]
        "/hello/*" => [
            "/hello/"      => result![],
            "/hello"       => None,
            "/hello/world" => result![]
        ],
        #[test(any_1)]
        "/hello/**" => [
            "/hello"         => None,
            "/hello/"        => result![],
            "/hello/world"   => result![],
            "/hello/world/"  => result![],
            "/hello/world/!" => result![]
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
