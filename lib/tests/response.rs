use crate::Response;

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
fn response_close() {
    let response = Response::new().close();

    assert!(response.close);
}
