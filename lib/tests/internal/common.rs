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

#[test]
fn trim_end_bytes() {
    let mut inp = vec![1, 2, 3, 4, 0, 0, 0, 0];
    common::trim_end_bytes(&mut inp);
    assert_eq!(inp, vec![1, 2, 3, 4])
}
