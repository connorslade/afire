//! HTTP Path stuff

use super::encoding::url;

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
    /// Tokenize a new path
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

    /// Match Path, returns None if it doesn't match and the path params if it does
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
                PathPart::Param(x) => {
                    out.push((x.to_owned(), url::decode(j).unwrap_or_else(|| j.to_owned())))
                }
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

#[cfg(test)]
mod test {
    use super::{normalize_path, Path, PathPart};

    #[test]
    fn test_path_new() {
        assert_eq!(
            Path::new("/".to_owned()),
            Path {
                parts: vec![PathPart::Normal("".to_owned())],
                raw: "".to_owned()
            }
        );

        assert_eq!(
            Path::new("/cool/{bean}".to_owned()),
            Path {
                parts: vec![
                    PathPart::Normal("cool".to_owned()),
                    PathPart::Param("bean".to_owned())
                ],
                raw: "cool/{bean}".to_owned()
            }
        );

        assert_eq!(
            Path::new("idk/*".to_owned()),
            Path {
                parts: vec![PathPart::Normal("idk".to_owned()), PathPart::Any],
                raw: "idk/*".to_owned()
            }
        )
    }

    #[test]
    fn test_match_path_normal() {
        assert_eq!(
            Path::new("/".to_owned()).match_path("/".to_owned()),
            Some(vec![])
        );

        assert_eq!(
            Path::new("/".to_owned()).match_path("".to_owned()),
            Some(vec![])
        );
    }

    #[test]
    fn test_match_path_param() {
        assert_eq!(
            Path::new("/cool/{bean}".to_owned()).match_path("/Cool/Bean".to_owned()),
            None
        );

        assert_eq!(
            Path::new("/cool/{bean}".to_owned()).match_path("/cool/Bean".to_owned()),
            Some(vec![("bean".to_owned(), "Bean".to_owned())])
        );
    }

    #[test]
    fn test_match_path_any() {
        assert_eq!(
            Path::new("idk/*".to_owned()).match_path("/idk/Cool Beans".to_owned()),
            Some(vec![])
        );

        assert_eq!(
            Path::new("idk/*".to_owned()).match_path("/idk/Cool/Beans".to_owned()),
            None
        );
    }

    #[test]
    fn test_path_part_from_normal() {
        assert_eq!(
            PathPart::from_segment("coolbeans"),
            PathPart::Normal("coolbeans".to_owned())
        );

        assert_eq!(PathPart::from_segment(""), PathPart::Normal("".to_owned()));
    }

    #[test]
    fn test_path_part_from_param() {
        assert_eq!(
            PathPart::from_segment("{bean}"),
            PathPart::Param("bean".to_owned())
        );

        assert_eq!(PathPart::from_segment("{}"), PathPart::Param("".to_owned()));
    }

    #[test]
    fn test_path_part_from_any() {
        assert_eq!(PathPart::from_segment("*"), PathPart::Any);
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(
            normalize_path("/COOL/BEANS/".to_owned()),
            "COOL/BEANS".to_owned()
        );

        assert_eq!(
            normalize_path("////COOL/BEANS////".to_owned()),
            "COOL/BEANS".to_owned()
        );
    }
}
