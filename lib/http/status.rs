//! HTTP status codes.
//! Used in [`crate::Request`] and [`crate::Response`].

/// HTTP status codes.
/// Used to indicate the status of an HTTP response.
/// Note: Methods that accept a [`Status`] will also accept any [`u16`] value, converting it to a Status::Custom if it is not a valid status code.
///
///  Supports Status:
/// - 100-101
/// - 200-206
/// - 300-307
/// - 400-417
/// - 500-505
///
/// From <https://developer.mozilla.org/en-US/docs/Web/HTTP/Status>
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Status {
    // == Informational ==
    /// HTTP 100 Continue.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/100)
    Continue,
    /// HTTP 101 Switching Protocols.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/101)
    SwitchingProtocols,
    /// HTTP 103 Early Hints.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/103)
    EarlyHints,

    // == Success ==
    /// HTTP 200 OK.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/200)
    Ok,
    /// HTTP 201 Created.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/201)
    Created,
    /// HTTP 202 Accepted.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/202)
    Accepted,
    /// HTTP 203 Non-Authoritative Information.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/203)
    NonAuthoritativeInformation,
    /// HTTP 204 No Content.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/204)
    NoContent,
    /// HTTP 205 Reset Content.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/205)
    ResetContent,
    /// HTTP 206 Partial Content.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/206)
    PartialContent,

    // == Redirection ==
    /// HTTP 300 Multiple Choices.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/300)
    MultipleChoices,
    /// HTTP 301 Moved Permanently.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/301)
    MovedPermanently,
    /// HTTP 302 Found.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/302)
    Found,
    /// HTTP 303 See Other.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/303)
    SeeOther,
    /// HTTP 304 Not Modified.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/304)
    NotModified,
    /// HTTP 305 Use Proxy.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/305)
    UseProxy,
    /// HTTP 307 Temporary Redirect.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/307)
    TemporaryRedirect,
    /// HTTP 308 Permanent Redirect.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/308)
    PermanentRedirect,

    // == Client Error ==
    /// HTTP 400 Bad Request.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/400)
    BadRequest,
    /// HTTP 401 Unauthorized.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401)
    Unauthorized,
    /// HTTP 402 Payment Required.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/402)
    PaymentRequired,
    /// HTTP 403 Forbidden.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/403)
    Forbidden,
    /// HTTP 404 Not Found.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404)
    NotFound,
    /// HTTP 405 Method Not Allowed.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/405)
    MethodNotAllowed,
    /// HTTP 406 Not Acceptable.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/406)
    NotAcceptable,
    /// HTTP 407 Proxy Authentication Required.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/407)
    ProxyAuthenticationRequired,
    /// HTTP 408 Request Time-out.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/408)
    RequestTimeOut,
    /// HTTP 409 Conflict.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/409)
    Conflict,
    /// HTTP 410 Gone.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/410)
    Gone,
    /// HTTP 411 Length Required.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/411)
    LengthRequired,
    /// HTTP 412 Precondition Failed.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/412)
    PreconditionFailed,
    /// HTTP 413 Payload Too Large.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/413)
    PayloadTooLarge,
    /// HTTP 414 URI Too Large.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/414)
    URITooLarge,
    /// HTTP 415 Unsupported Media Type.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/415)
    UnsupportedMediaType,
    /// HTTP 416 Range Not Satisfiable.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/416)
    RangeNotSatisfiable,
    /// HTTP 417 Expectation Failed.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/417)
    ExpectationFailed,
    /// HTTP 418 I'm a teapot.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/418)
    ImaTeapot,
    /// HTTP 421 Misdirected Request.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/421)
    MisdirectedRequest,
    /// HTTP 425 Too Early.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/425)
    TooEarly,
    /// HTTP 426 Upgrade Required.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/426)
    UpgradeRequired,
    /// HTTP 428 Precondition Required.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/428)
    PreconditionRequired,
    /// HTTP 429 Too Many Requests.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/429)
    TooManyRequests,
    /// HTTP 431 Request Header Fields Too Large.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/431)
    RequestHeaderFieldsTooLarge,
    /// HTTP 451 Unavailable For Legal Reasons.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/451)
    UnavailableForLegalReasons,

    // == Server Error ==
    /// HTTP 500 Internal Server Error.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/500)
    InternalServerError,
    /// HTTP 501 Not Implemented.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/501)
    NotImplemented,
    /// HTTP 502 Bad Gateway.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/502)
    BadGateway,
    /// HTTP 503 Service Unavailable.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/503)
    ServiceUnavailable,
    /// HTTP 504 Gateway Time-out.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/504)
    GatewayTimeOut,
    /// HTTP 505 HTTP Version Not Supported.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/505)
    HTTPVersionNotSupported,
    /// HTTP 506 Variant Also Negotiates.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/506)
    VariantAlsoNegotiates,
    /// HTTP 510 Not Extended.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/510)
    NotExtended,
    /// HTTP 511 Network Authentication Required.
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/511)
    NetworkAuthenticationRequired,

    // == Custom ==
    /// Custom status code
    Custom(u16),
}

impl Status {
    /// Gets the actual HTTP status code for the status.
    pub fn code(&self) -> u16 {
        match self {
            Status::Continue => 100,
            Status::SwitchingProtocols => 101,
            Status::EarlyHints => 103,

            Status::Ok => 200,
            Status::Created => 201,
            Status::Accepted => 202,
            Status::NonAuthoritativeInformation => 203,
            Status::NoContent => 204,
            Status::ResetContent => 205,
            Status::PartialContent => 206,

            Status::MultipleChoices => 300,
            Status::MovedPermanently => 301,
            Status::Found => 302,
            Status::SeeOther => 303,
            Status::NotModified => 304,
            Status::UseProxy => 305,
            Status::TemporaryRedirect => 307,
            Status::PermanentRedirect => 308,

            Status::BadRequest => 400,
            Status::Unauthorized => 401,
            Status::PaymentRequired => 402,
            Status::Forbidden => 403,
            Status::NotFound => 404,
            Status::MethodNotAllowed => 405,
            Status::NotAcceptable => 406,
            Status::ProxyAuthenticationRequired => 407,
            Status::RequestTimeOut => 408,
            Status::Conflict => 409,
            Status::Gone => 410,
            Status::LengthRequired => 411,
            Status::PreconditionFailed => 412,
            Status::PayloadTooLarge => 413,
            Status::URITooLarge => 414,
            Status::UnsupportedMediaType => 415,
            Status::RangeNotSatisfiable => 416,
            Status::ExpectationFailed => 417,
            Status::ImaTeapot => 418,
            Status::MisdirectedRequest => 421,
            Status::TooEarly => 425,
            Status::UpgradeRequired => 426,
            Status::PreconditionRequired => 428,
            Status::TooManyRequests => 429,
            Status::RequestHeaderFieldsTooLarge => 431,
            Status::UnavailableForLegalReasons => 451,

            Status::InternalServerError => 500,
            Status::NotImplemented => 501,
            Status::BadGateway => 502,
            Status::ServiceUnavailable => 503,
            Status::GatewayTimeOut => 504,
            Status::HTTPVersionNotSupported => 505,
            Status::VariantAlsoNegotiates => 506,
            Status::NotExtended => 510,
            Status::NetworkAuthenticationRequired => 511,

            Status::Custom(x) => *x,
        }
    }

    /// Gets the default reason phrase for the status.
    /// For responses you can use the [`crate::Response::reason`] method to set a custom reason phrase.
    pub fn reason_phrase(&self) -> &str {
        match self.code() {
            100 => "Continue",
            101 => "Switching Protocols",
            103 => "Early Hints",

            200 => "OK",
            201 => "Created",
            202 => "Accepted",
            203 => "Non-Authoritative Information",
            204 => "No Content",
            205 => "Reset Content",
            206 => "Partial Content",

            300 => "Multiple Choices",
            301 => "Moved Permanently",
            302 => "Found",
            303 => "See Other",
            304 => "Not Modified",
            305 => "Use Proxy",
            307 => "Temporary Redirect",
            308 => "Permanent Redirect",

            400 => "Bad Request",
            401 => "Unauthorized",
            402 => "Payment Required",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            406 => "Not Acceptable",
            407 => "Proxy Authentication Required",
            408 => "Request Time-out",
            409 => "Conflict",
            410 => "Gone",
            411 => "Length Required",
            412 => "Precondition Failed",
            413 => "Request Entity Too Large",
            414 => "Request-URI Too Large",
            415 => "Unsupported Media Type",
            416 => "Requested range not satisfiable",
            417 => "Expectation Failed",
            418 => "I'm a teapot",
            421 => "Misdirected Request",
            425 => "Too Early",
            426 => "Upgrade Required",
            428 => "Precondition Required",
            429 => "Too Many Requests",
            431 => "Request Header Fields Too Large",
            451 => "Unavailable For Legal Reasons",

            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            504 => "Gateway Time-out",
            505 => "HTTP Version not supported",
            506 => "Variant Also Negotiates",
            510 => "Not Extended",
            511 => "Network Authentication Required",
            _ => "OK",
        }
    }
}

impl From<u16> for Status {
    fn from(code: u16) -> Self {
        match code {
            100 => Status::Continue,
            101 => Status::SwitchingProtocols,
            103 => Status::EarlyHints,

            200 => Status::Ok,
            201 => Status::Created,
            202 => Status::Accepted,
            203 => Status::NonAuthoritativeInformation,
            204 => Status::NoContent,
            205 => Status::ResetContent,
            206 => Status::PartialContent,

            300 => Status::MultipleChoices,
            301 => Status::MovedPermanently,
            302 => Status::Found,
            303 => Status::SeeOther,
            304 => Status::NotModified,
            305 => Status::UseProxy,
            307 => Status::TemporaryRedirect,
            308 => Status::PermanentRedirect,

            400 => Status::BadRequest,
            401 => Status::Unauthorized,
            402 => Status::PaymentRequired,
            403 => Status::Forbidden,
            404 => Status::NotFound,
            405 => Status::MethodNotAllowed,
            406 => Status::NotAcceptable,
            407 => Status::ProxyAuthenticationRequired,
            408 => Status::RequestTimeOut,
            409 => Status::Conflict,
            410 => Status::Gone,
            411 => Status::LengthRequired,
            412 => Status::PreconditionFailed,
            413 => Status::PayloadTooLarge,
            414 => Status::URITooLarge,
            415 => Status::UnsupportedMediaType,
            416 => Status::RangeNotSatisfiable,
            417 => Status::ExpectationFailed,
            418 => Status::ImaTeapot,
            421 => Status::MisdirectedRequest,
            425 => Status::TooEarly,
            426 => Status::UpgradeRequired,
            428 => Status::PreconditionRequired,
            429 => Status::TooManyRequests,
            431 => Status::RequestHeaderFieldsTooLarge,
            451 => Status::UnavailableForLegalReasons,

            500 => Status::InternalServerError,
            501 => Status::NotImplemented,
            502 => Status::BadGateway,
            503 => Status::ServiceUnavailable,
            504 => Status::GatewayTimeOut,
            505 => Status::HTTPVersionNotSupported,
            506 => Status::VariantAlsoNegotiates,
            510 => Status::NotExtended,
            511 => Status::NetworkAuthenticationRequired,

            x => Status::Custom(x),
        }
    }
}
