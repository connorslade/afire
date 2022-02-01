use crate::path;
use crate::path::{Path, PathPart};

#[test]
fn path_new() {
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
fn match_path_normal() {
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
fn match_path_param() {
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
fn match_path_any() {
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
fn path_part_from_normal() {
    assert_eq!(
        PathPart::from_segment("coolbeans"),
        PathPart::Normal("coolbeans".to_owned())
    );

    assert_eq!(PathPart::from_segment(""), PathPart::Normal("".to_owned()));
}

#[test]
fn path_part_from_param() {
    assert_eq!(
        PathPart::from_segment("{bean}"),
        PathPart::Param("bean".to_owned())
    );

    assert_eq!(PathPart::from_segment("{}"), PathPart::Param("".to_owned()));
}

#[test]
fn path_part_from_any() {
    assert_eq!(PathPart::from_segment("*"), PathPart::Any);
}

#[test]
fn normalize_path() {
    assert_eq!(
        path::normalize_path("/COOL/BEANS/".to_owned()),
        "COOL/BEANS".to_owned()
    );

    assert_eq!(
        path::normalize_path("////COOL/BEANS////".to_owned()),
        "COOL/BEANS".to_owned()
    );
}
