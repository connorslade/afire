use crate::route::Route;
use crate::{path::Path, Method, Response};

#[test]
fn route_new() {
    let route = Route::<()>::new(Method::GET, "/".to_owned(), Box::new(|_| Response::new()));

    assert_eq!(route.method, Method::GET);
    assert_eq!(route.path, Path::new("/".to_owned()));
}
