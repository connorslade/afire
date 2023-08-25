use std::{
    collections::HashMap,
    fmt::{self, Display},
};

#[derive(Debug)]
pub struct Path {
    segments: Vec<Segment>,
}

#[derive(Debug)]
pub enum Segment {
    Separator,
    Literal(String),
    Parameter(String),
    Any,
    AnyAfter,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PathParameters {
    params: HashMap<String, String>,
}

impl Path {
    pub fn new(path: &str) -> Path {
        let mut tokenizer = tokenizer::Tokenizer::new(path);
        tokenizer.tokenize();

        let path = Path {
            segments: tokenizer.tokens,
        };

        let mut last_special = false;
        for i in &path.segments {
            if last_special && i.disallow_adjacent() {
                panic!("Any, AnyAfter, and Parameter segments cannot be adjacent as they make matching ambiguous. ({})", path);
            }

            last_special = i.disallow_adjacent();
        }

        path
    }

    pub fn matches(&self, path: &str) -> Option<PathParameters> {
        let mut matcher = matcher::Matcher::new(&self.segments, path);
        matcher.matches().map(|params| PathParameters { params })
    }
}

impl Segment {
    fn disallow_adjacent(&self) -> bool {
        matches!(
            self,
            Segment::Any | Segment::AnyAfter | Segment::Parameter(_)
        )
    }
}

impl PathParameters {
    pub(crate) fn new(params: HashMap<String, String>) -> PathParameters {
        PathParameters { params }
    }

    pub(crate) fn get(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|x| x.as_str())
    }
}

impl Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Segment::Separator => f.write_str("/"),
            Segment::Literal(l) => f.write_str(l),
            Segment::Parameter(p) => f.write_fmt(format_args!("{{{}}}", p)),
            Segment::Any => f.write_str("*"),
            Segment::AnyAfter => f.write_str("**"),
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in &self.segments {
            write!(f, "{}", i)?;
        }
        Ok(())
    }
}

mod matcher {
    use std::{collections::HashMap, ops::Range};

    use super::Segment;

    pub struct Matcher<'a> {
        segments: &'a [Segment],
        path: Vec<char>,

        seg_index: usize,
        path_index: usize,
        params: Option<HashMap<String, String>>,
    }

    impl<'a> Matcher<'a> {
        pub fn new(segments: &'a [Segment], path: &'a str) -> Matcher<'a> {
            Matcher {
                segments,
                path: path.chars().collect(),

                seg_index: 0,
                path_index: 0,
                params: Some(HashMap::new()),
            }
        }

        pub fn matches(&mut self) -> Option<HashMap<String, String>> {
            while self.seg_index < self.segments.len() {
                match self.next_seg() {
                    Segment::AnyAfter => return Some(self.params.take().unwrap()),
                    Segment::Any => self.match_any()?,
                    Segment::Literal(l) => {
                        let l = l.to_string();
                        self.take_str(&l)?;
                    }
                    Segment::Parameter(p) => {
                        let key = p.to_owned();
                        let val = self.match_parameter()?.to_string();
                        // TODO: Remove clones?
                        self.params.as_mut().unwrap().insert(key, val);
                    }
                    Segment::Separator => self.take('/')?,
                }
            }

            if self.path_index >= self.path.len() {
                return Some(self.params.take().unwrap());
            }

            None
        }
    }

    impl<'a> Matcher<'a> {
        fn next_seg(&mut self) -> &Segment {
            let seg = &self.segments[self.seg_index];
            self.seg_index += 1;
            seg
        }

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

            values.next().is_none().then(|| ())
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
            let end_pos = match self.segments.get(self.seg_index) {
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
                Some(Segment::Any | Segment::AnyAfter | Segment::Parameter(_)) => unreachable!(),
            };

            Some(self.path_index..end_pos)
        }

        fn match_any(&mut self) -> Option<()> {
            let range = self.match_around()?;
            self.path_index = range.end;
            Some(())
        }

        fn match_parameter(&mut self) -> Option<String> {
            let range = self.match_around()?;
            self.path_index = range.end;
            let val = &self.path[range];
            Some(val.iter().collect::<String>())
        }
    }
}

mod tokenizer {
    use super::Segment;

    pub struct Tokenizer {
        chars: Vec<char>,
        index: usize,

        pub tokens: Vec<Segment>,
        buffer: String,
    }

    impl Tokenizer {
        pub fn new(path: &str) -> Tokenizer {
            Tokenizer {
                chars: path.chars().collect(),
                index: 0,

                tokens: Vec::new(),
                buffer: String::new(),
            }
        }

        pub fn tokenize(&mut self) {
            while self.index < self.chars.len() {
                let chr = self.next();
                match chr {
                    '/' => {
                        self.flush_buffer();
                        self.tokens.push(Segment::Separator);
                    }
                    '{' => {
                        self.flush_buffer();
                        self.parse_parameter();
                    }
                    '*' if self.peek() == Some(&'*') => {
                        self.flush_buffer();
                        self.tokens.push(Segment::AnyAfter);
                        self.index += 1;

                        if self.index < self.chars.len() {
                            panic!("AnyAfter must be the last segment");
                        }

                        return;
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
            self.flush_buffer()
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

        fn parse_parameter(&mut self) {
            while self.index < self.chars.len() {
                let chr = self.next();
                match chr {
                    '{' => {
                        panic!("nested parameters are not allowed");
                    }
                    '}' => {
                        self.tokens.push(Segment::Parameter(self.buffer.clone()));
                        self.buffer.clear();
                        return;
                    }
                    _ => {
                        self.buffer.push(chr);
                    }
                }
            }

            panic!("unterminated parameter");
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Path, PathParameters};
    use std::collections::HashMap;

    macro_rules! match_result {
        [] => {
            PathParameters::new(HashMap::new())
        };
        [$($key:tt => $val:tt),*] => {
            {
                let mut map = HashMap::new();
                $(
                    map.insert($key.to_owned(), $val.to_owned());
                )*
                PathParameters::new(map)
            }
        };
    }

    macro_rules! match_tests {
        {$(#[test($test_name:ident)] $($path:literal => [$($test:literal => $result:expr),+]),+),*} => {
            $(
                #[test]
                fn $test_name() {
                    $(
                        let path = Path::new($path);
                        $(
                            assert_eq!(path.matches($test), $result, "`{}`.matches(`{}`)", $path, $test);
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
            "/hello!"       => Some(match_result![]), // FAILING
            "/hello"        => None,
            "/helloworld!"  => Some(match_result![]),
            "/helloworld"   => None,
            "/hello/world!" => None
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
}
