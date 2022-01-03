use crate::common;

#[test]
fn remove_address_port() {
    assert_eq!(common::remove_address_port("127.0.0.1:6032"), "127.0.0.1");
    assert_eq!(common::remove_address_port("127.0.0.1"), "127.0.0.1");
    assert_eq!(common::remove_address_port(":"), "");
}

#[test]
fn decode_url() {
    assert_eq!(
        common::decode_url("/Page%20With%20Spaces".to_owned()),
        "/Page With Spaces"
    );

    assert_eq!(common::decode_url("%A9".to_owned()), "©");
}
