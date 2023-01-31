//! HTTP Path stuff

use crate::common;

/// Http Path
#[derive(Debug, PartialEq, Eq)]
pub struct Path {
    /// Raw Path String
    pub raw: String,

    /// Path Segments
    pub parts: Vec<PathPart>,
}

/// Segment of a path
///
/// Ex: `/hello/{name}` => [Normal::("hello"), Param::("name")]
#[derive(Debug, PartialEq, Eq)]
pub enum PathPart {
    /// Normal Path Segment (/hi)
    Normal(String),

    /// Path param (/{name})
    Param(String),

    /// Match anything for self and after
    AnyAfter,

    /// Literally Anything (E)
    Any,
}

impl Path {
    /// Make a new path
    pub fn new(path: String) -> Path {
        let path = normalize_path(path);
        let mut out = Vec::new();

        // Split off into Path Parts
        for i in path.split('/') {
            out.push(PathPart::from_segment(i));
        }

        Path {
            raw: path,
            parts: out,
        }
    }

    /// Match Path
    pub fn match_path(&self, path: String) -> Option<Vec<(String, String)>> {
        let path = normalize_path(path);
        let mut out = Vec::new();

        let path = path.split('/');

        for (i, j) in self.parts.iter().zip(path.clone()) {
            match i {
                PathPart::Normal(x) => {
                    if x != j {
                        return None;
                    }
                }
                PathPart::Param(x) => out.push((x.to_owned(), common::decode_url(j.to_owned()))),
                PathPart::AnyAfter => return Some(out),
                PathPart::Any => {}
            }
        }

        if path.count() != self.parts.len() {
            return None;
        }

        Some(out)
    }
}

impl PathPart {
    /// Decode Path Segment into PathPart
    pub fn from_segment(seg: &str) -> PathPart {
        match seg {
            "*" => PathPart::Any,
            "**" => PathPart::AnyAfter,
            x if x.starts_with('{') && x.ends_with('}') => PathPart::Param(
                x.strip_prefix('{')
                    .unwrap()
                    .strip_suffix('}')
                    .unwrap()
                    .to_owned(),
            ),
            _ => PathPart::Normal(seg.to_owned()),
        }
    }
}

/// Normalize a Path
///
/// Removes loading and trailing slashes
pub fn normalize_path(mut path: String) -> String {
    while path.ends_with('/') {
        path.pop();
    }

    while path.starts_with('/') {
        path.remove(0);
    }

    path
}
