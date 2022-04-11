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

        let path = path.split('/');

        let mut any_after = false;
        for (i, j) in self.parts.iter().zip(path.clone()) {
            if any_after {
                continue;
            }

            match i {
                PathPart::Normal(x) => {
                    if x != j {
                        return None;
                    }
                }
                PathPart::Param(x) => out.push((x.to_owned(), j.to_owned())),
                PathPart::AnyAfter => any_after = true,
                PathPart::Any => {}
            }
        }

        if !any_after && path.count() != self.parts.len() {
            return None;
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
