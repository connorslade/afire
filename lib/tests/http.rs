use crate::http;
use crate::Method;

#[test]
fn get_request_method() {
    assert_eq!(http::get_request_method("GET /path HTTP/1.1"), Method::GET);
    assert_eq!(http::get_request_method("post /path HTTP/1.1"), Method::POST);
    assert_eq!(http::get_request_method("OptIoNs /path HTTP/1.1"), Method::OPTIONS);
}
