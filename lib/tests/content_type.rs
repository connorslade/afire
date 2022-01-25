use crate::Content;

#[test]
fn content_type() {
    // Builtin Types
    assert_eq!(Content::HTML.as_type(), "text/html");
    assert_eq!(Content::TXT.as_type(), "text/plain");
    assert_eq!(Content::CSV.as_type(), "text/csv");
    assert_eq!(Content::JSON.as_type(), "application/json");
    assert_eq!(Content::XML.as_type(), "application/xml");

    // Custom Type
    assert_eq!(Content::Custom("Hello").as_type(), "Hello");
}
