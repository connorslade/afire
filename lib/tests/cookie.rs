use crate::cookie::{Cookie, SetCookie};

#[test]
fn cookie_new() {
    assert_eq!(
        Cookie::new("Cool", "Beans"),
        Cookie {
            name: "Cool".to_owned(),
            value: "Beans".to_owned()
        }
    );
}

#[test]
fn cookie_from_string() {
    assert_eq!(
        Cookie::from_string("Cookie: name=value;"),
        Some(vec![Cookie {
            name: "name".to_owned(),
            value: "value".to_owned()
        }])
    );

    assert_eq!(
        Cookie::from_string("Cookie:name = value "),
        Some(vec![Cookie {
            name: "name".to_owned(),
            value: "value".to_owned()
        }])
    );

    assert_eq!(Cookie::from_string("Cookiez: name = value;"), None)
}

#[test]
fn set_cookie_new() {
    assert_eq!(
        SetCookie::new("Cool", "Beans"),
        SetCookie {
            cookie: Cookie {
                name: "Cool".to_owned(),
                value: "Beans".to_owned()
            },
            max_age: None,
            domain: None,
            path: None,
            secure: false,
        }
    )
}

#[test]
fn set_cookie_max_age() {
    assert_eq!(
        SetCookie::new("Cool", "Beans").max_age(100).max_age,
        Some(100)
    );
}

#[test]
fn set_cookie_domain() {
    assert_eq!(
        SetCookie::new("Cool", "Beans").domain("nose.net").domain,
        Some("nose.net".to_owned())
    );
}

#[test]
fn set_cookie_path() {
    assert_eq!(
        SetCookie::new("Cool", "Beans").path("/a/b/c").path,
        Some("/a/b/c".to_owned())
    );
}

#[test]
fn set_cookie_secure() {
    assert_eq!(SetCookie::new("Cool", "Beans").secure(true).secure, true);
    assert_eq!(SetCookie::new("Cool", "Beans").secure, false);
}
