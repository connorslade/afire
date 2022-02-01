use crate::http;
use crate::{Cookie, Header, Method, Query};

const HTTP: &str = "GET /path HTTP/1.1\r
Hello: World\r
Cookie: name=value\r
\r
B O D Y";

const HTTP2: &str = "post /spooky?Hello=World HTTP/1.1\r
Cool: Beans\r
\r
C O O L   B E A N S
";

const HTTP3: &str = "OptIoNs /Page%20With%20Spaces HTTP/1.1\r
Nose: Dog 🐶\r
\r
";

#[test]
fn get_request_method() {
    assert_eq!(http::get_request_method(HTTP), Method::GET);
    assert_eq!(http::get_request_method(HTTP2), Method::POST);
    assert_eq!(http::get_request_method(HTTP3), Method::OPTIONS);
}

#[test]
fn get_request_path() {
    assert_eq!(http::get_request_path(HTTP), "/path");
    assert_eq!(http::get_request_path(HTTP2), "/spooky");
    assert_eq!(http::get_request_path(HTTP3), "/Page With Spaces");
}

#[test]
fn get_request_query() {
    assert_eq!(http::get_request_query(HTTP), Query(Vec::new()));
    assert_eq!(
        http::get_request_query(HTTP2),
        Query(vec![["Hello".to_owned(), "World".to_owned()]])
    );
    assert_eq!(http::get_request_query(HTTP3), Query(Vec::new()));
    assert_eq!(http::get_request_query(""), Query(Vec::new()));
}

#[test]
fn get_request_body() {
    assert_eq!(
        http::get_request_body(HTTP.as_bytes()),
        vec![66, 32, 79, 32, 68, 32, 89]
    );
    assert_eq!(
        http::get_request_body(HTTP2.as_bytes()),
        vec![67, 32, 79, 32, 79, 32, 76, 32, 32, 32, 66, 32, 69, 32, 65, 32, 78, 32, 83, 10]
    );
    assert_eq!(http::get_request_body(HTTP3.as_bytes()), vec![]);
    assert_eq!(http::get_request_body(&vec![]), vec![]);
}

#[test]
fn get_request_headers() {
    assert_eq!(
        http::get_request_headers(HTTP),
        vec![
            Header::new("Hello", "World"),
            Header::new("Cookie", "name=value")
        ]
    );
    assert_eq!(
        http::get_request_headers(HTTP2),
        vec![Header::new("Cool", "Beans")]
    );
    assert_eq!(
        http::get_request_headers(HTTP3),
        vec![Header::new("Nose", "Dog 🐶")]
    );
}

#[cfg(feature = "cookies")]
#[test]
fn get_request_cookies() {
    assert_eq!(
        http::get_request_cookies(HTTP),
        vec![Cookie::new("name", "value")]
    );
    assert_eq!(http::get_request_cookies(HTTP2), vec![]);
    assert_eq!(http::get_request_cookies(HTTP3), vec![]);
}

#[cfg(feature = "dynamic_resize")]
#[test]
fn get_header_size() {
    assert_eq!(http::get_header_size(HTTP), 56);
    assert_eq!(http::get_header_size(HTTP2), 50);
    assert_eq!(http::get_header_size(HTTP3), 58);
    assert_eq!(http::get_header_size("abcd"), 4);
}

// extern crate test;
// use test::Bencher;
//
// #[bench]
// // 114ns +-19
// // 60ns +-6 --y
// fn bench_request_method(b: &mut Bencher) {
//     b.iter(|| test::black_box(http::get_request_method("GET /path HTTP/1.1")));
// }
//
// #[bench]
// // 349ns +-116
// // 114 +- 18 (No URL Decode)
// // 79 +- 2 (No URL Decode) --
// fn bench_request_path(b: &mut Bencher) {
//     b.iter(|| test::black_box(http::get_request_path("GET /path HTTP/1.1")));
// }
//
// #[bench]
// // 99ns +-84
// // 22ns +-0
// fn bench_request_query(b: &mut Bencher) {
//     b.iter(|| test::black_box(http::get_request_query("GET /path HTTP/1.1")));
// }
//
// // 328ns +-8
// // 36ns +-2
// #[bench]
// fn bench_request_headers(b: &mut Bencher) {
//     b.iter(|| {
//         test::black_box(http::get_request_headers(
//             r#"GET /path HTTP/1.1
// Hello: World
//
// B O D Y"#,
//         ))
//     });
// }
//
// #[bench]
// fn bench_request_body(b: &mut Bencher) {
//     b.iter(|| {
//         test::black_box(http::get_request_body(
//             r#"GET /path HTTP/1.1
// Hello: World
//
// B O D Y"#
//                 .as_bytes(),
//         ))
//     });
// }
//
// #[bench]
// fn bench_request_size(b: &mut Bencher) {
//     b.iter(|| {
//         test::black_box(http::get_header_size(
//             r#"GET /path HTTP/1.1
// Hello: World
//
// B O D Y"#,
//         ))
//     });
// }
