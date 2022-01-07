//! HTTP Path stuff

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

    /// Literly Anything (E)
    Any,
}

impl Path {
    /// Make a new path
    pub fn new(path: String) -> Path {
        let path = normalize_path(path);
        let mut out = Vec::new();

        // Split off into Path Parts
        for i in path.split('/') {
            #[cfg(feature = "path_patterns")]
            out.push(PathPart::from_segment(i));

            #[cfg(not(feature = "path_patterns"))]
            out.push(PathPart::Normal(i.to_owned()));
        }

        Path {
            raw: path,
            parts: out,
        }
    }

    /// Match Path
    #[cfg(feature = "path_patterns")]
    pub fn match_path(&self, path: String) -> Option<Vec<(String, String)>> {
        let path = normalize_path(path);
        let mut out = Vec::new();

        if self.raw == "/**" {
            return Some(Vec::new());
        }

        let path = path.split('/');

        if path.clone().count() != self.parts.len() {
            return None;
        }

        for (i, j) in self.parts.iter().zip(path) {
            match i {
                PathPart::Normal(x) => {
                    if x != j {
                        return None;
                    }
                }
                PathPart::Param(x) => out.push((x.to_owned(), j.to_owned())),
                PathPart::Any => {}
            }
        }

        Some(out)
    }

    /// Match Path
    #[cfg(not(feature = "path_patterns"))]
    pub fn match_path(&self, path: String) -> Option<()> {
        if self.raw == path {
            return Some(());
        }
        None
    }
}

impl PathPart {
    /// Decode Path Segment into PathPart
    #[cfg(feature = "path_patterns")]
    pub fn from_segment(seg: &str) -> PathPart {
        if seg == "*" {
            return PathPart::Any;
        }

        if seg.starts_with('{') && seg.ends_with('}') {
            return PathPart::Param(
                seg.strip_prefix('{')
                    .unwrap()
                    .strip_suffix('}')
                    .unwrap()
                    .to_owned(),
            );
        }

        PathPart::Normal(seg.to_owned())
    }
}

/// Normalize a Path
///
/// Makes it start and optinaly end with a slash
pub fn normalize_path(mut path: String) -> String {
    #[cfg(feature = "ignore_trailing_path_slash")]
    if path.ends_with('/') {
        path.pop();
    }

    if !path.starts_with('/') {
        path.insert(0, '/');
    }

    path
}
