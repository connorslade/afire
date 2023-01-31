use crate::{Header, Response, SetCookie};

#[test]
fn response_status() {
    let response = Response::new().status(100);

    assert_eq!(response.status, 100);
}

#[test]
fn response_reason() {
    let response = Response::new().reason("Good");

    assert_eq!(response.reason, Some("Good".to_owned()));
}

#[test]
fn response_header() {
    let response = Response::new()
        .header("Name", "Value")
        .header("Hello", "World");

    assert_eq!(
        response.headers,
        vec![Header::new("Name", "Value"), Header::new("Hello", "World")]
    );
}

#[test]
fn response_headers() {
    let response =
        Response::new().headers(&[Header::new("Name", "Value"), Header::new("Hello", "World")]);

    assert_eq!(
        response.headers,
        vec![Header::new("Name", "Value"), Header::new("Hello", "World")]
    );
}

#[test]
fn response_close() {
    let response = Response::new().close();

    assert!(response.close);
}

#[test]
fn response_cookie() {
    let response = Response::new().cookie(SetCookie::new("Name", "Value"));

    assert_eq!(
        response.headers,
        vec![Header::new("Set-Cookie", "Name=Value;")]
    );
}

#[test]
fn response_cookies() {
    let response = Response::new().cookies(&[
        SetCookie::new("Name", "Value"),
        SetCookie::new("Hello", "World"),
    ]);

    assert_eq!(
        response.headers,
        vec![
            Header::new("Set-Cookie", "Name=Value;"),
            Header::new("Set-Cookie", "Hello=World;")
        ]
    );
}
