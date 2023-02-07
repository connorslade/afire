use crate::header::{self, Header, HeaderType};

#[test]
fn header_new() {
    let header = Header::new("Hello", "World");

    assert_eq!(
        header,
        Header {
            name: HeaderType::Custom("Hello".to_owned()),
            value: "World".to_owned(),
        }
    );
}

#[test]
fn header_parse() {
    let header = Header::from_string("Name: Value").unwrap();

    assert_eq!(
        header,
        Header {
            name: HeaderType::Custom("Name".to_owned()),
            value: "Value".to_owned(),
        }
    );
}

#[test]
fn header_parse_empty() {
    let header = Header::from_string("");

    assert!(header.is_err());
}

#[test]
fn header_format_display() {
    let header = Header::new("Hello", "World");

    assert_eq!(format!("{}", header), "Hello: World");
}

#[test]
fn headers_to_string() {
    let headers = vec![Header::new("Hello", "World"), Header::new("Name", "Value")];

    assert_eq!(
        header::headers_to_string(&headers),
        "Hello: World\r\nName: Value".to_owned()
    );
}
