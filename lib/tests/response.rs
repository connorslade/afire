use crate::{Response, Status};

#[test]
fn response_status() {
    let response = Response::new().status(200);

    assert_eq!(response.status, Status::Ok);
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
