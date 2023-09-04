use std::{borrow::Cow, fmt::Display};

pub struct Mime {
    r#type: Cow<'static, str>,
    subtype: Cow<'static, str>,
}

impl Display for Mime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.r#type, self.subtype)
    }
}

impl From<&str> for Mime {
    fn from(value: &str) -> Self {
        let mut iter = value.split('/');
        let r#type = iter.next().unwrap_or_default();
        let subtype = iter.next().unwrap_or_default();
        Mime {
            r#type: Cow::Owned(r#type.to_owned()),
            subtype: Cow::Owned(subtype.to_owned()),
        }
    }
}

macro_rules! mime {
    ($_type:ident / $_subtype:ident) => {
        Mime {
            r#type: Cow::Borrowed(stringify!($_type)),
            subtype: Cow::Borrowed(stringify!($_subtype)),
        }
    };
}

macro_rules! define_mimes {
    ($($name:ident => $_type:ident / $_subtype:ident),*) => {
        $( pub const $name: Mime = mime!($_type / $_subtype); )*
    };
}

define_mimes! {
    HTML => text/html,
    TEXT => text/plain,
    CSV  => text/csv,
    JSON => application/json,
    XML  => application/xml
}
