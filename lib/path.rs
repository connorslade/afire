#[derive(Debug, PartialEq, Eq)]
pub struct Path {
    pub raw: String,
    pub parts: Vec<PathPart>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PathPart {
    // Normal Path Segment (/hi)
    Normal(String),

    // Path param (/{name})
    Param(String),

    // Literly Anything (E)
    Any,
}

impl Path {
    pub(crate) fn new(mut path: String) -> Path {
        let mut out = Vec::new();

        // Normalize Path
        #[cfg(feature = "ignore_trailing_path_slash")]
        if path.ends_with('/') {
            path.pop();
        }

        if !path.starts_with('/') {
            path.insert(0, '/');
        }

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

    #[cfg(feature = "path_patterns")]
    pub(crate) fn match_path(&self, path: String) -> Option<Vec<(String, String)>> {
        let mut out = Vec::new();

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

    #[cfg(not(feature = "path_patterns"))]
    pub(crate) fn match_path(&self, path: String) -> Option<()> {
        if self.raw == path {
            return Some(());
        }
        None
    }
}

impl PathPart {
    #[cfg(feature = "path_patterns")]
    pub(crate) fn from_segment(seg: &str) -> PathPart {
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
