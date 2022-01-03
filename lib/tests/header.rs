use crate::Header;

#[test]
fn header_new() {
    let header = Header::new("Hello", "World");

    assert_eq!(header.name, "Hello");
    assert_eq!(header.value, "World");
}

#[test]
fn header_parse() {
    let header = Header::from_string("Name: Value").unwrap();

    assert_eq!(header.name, "Name");
    assert_eq!(header.value, "Value");
}

#[test]
fn header_parse_empty() {
    let header = Header::from_string("");

    assert!(header.is_none());
}

#[test]
fn header_format_debug() {
    let header = Header::new("Hello", "World");

    assert_eq!(
        format!("{:?}", header),
        r#"Header { name: "Hello", value: "World" }"#
    )
}

#[test]
fn header_format_display() {
    let header = Header::new("Hello", "World");

    assert_eq!(format!("{}", header), "Hello: World");
}
