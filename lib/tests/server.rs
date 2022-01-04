use std::time::Duration;

use afire::{Header, Server, VERSION};

#[test]
fn server_new() {
    let server = Server::new("localhost", 8080);

    assert_eq!(server.port, 8080);
    assert_eq!(server.ip.octets(), [127, 0, 0, 1]);
}

#[test]
fn server_ip_string() {
    let server = Server::new("localhost", 8080);
    let server2 = Server::new("1.2.3.4", 8080);

    assert_eq!(server.ip.to_string(), "127.0.0.1");
    assert_eq!(server2.ip.to_string(), "1.2.3.4");
}

#[test]
fn server_buff_resize() {
    let server = Server::new("localhost", 8080).buffer(1000);

    assert_eq!(server.buff_size, 1000);
}

#[test]
fn server_default_headers() {
    let server = Server::new("localhost", 8080)
        .default_header(Header::new("Hello", "World"))
        .default_header(Header::new("Server", "Magic"));

    assert_eq!(
        server.default_headers,
        vec![
            Header::new("Server", format!("afire/{}", VERSION)),
            Header::new("Hello", "World"),
            Header::new("Server", "Magic")
        ]
    )
}

#[test]
fn server_socket_timeout() {
    let server = Server::new("localhost", 8080).socket_timeout(Duration::from_secs(10));

    assert_eq!(server.socket_timeout, Some(Duration::from_secs(10)));
}

#[test]
fn server_set_run() {
    let mut server = Server::new("localhost", 8080);
    server.set_run(false);

    assert!(!server.run);

    // Should Not block thread
    server.start();
}
