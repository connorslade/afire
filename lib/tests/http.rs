use crate::http;
use crate::Method;

#[test]
fn get_request_method() {
    assert_eq!(http::get_request_method("GET /path HTTP/1.1"), Method::GET);
    assert_eq!(
        http::get_request_method("post /path HTTP/1.1"),
        Method::POST
    );
    assert_eq!(
        http::get_request_method("OptIoNs /path HTTP/1.1"),
        Method::OPTIONS
    );
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
