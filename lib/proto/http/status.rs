//! HTTP status codes.
//! Most functions that accept a [`Status`] will also accept any [`u16`] value, converting it to a Status::Custom if it is not a valid status code.s

macro_rules! status {
    {
        $(
            $(#[$attr:meta])*
            $name:ident => $status:literal, $reason:literal
        ),*
    } => {
        /// HTTP status codes.
        ///
        /// Used to indicate the status of an HTTP response.
        /// Note: Methods that accept a [`Status`] will also accept any [`u16`] value, converting it to a [`Status::Custom`] if it is not a valid status code.
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
            $(
                #[doc = concat!("**", $status, " ", $reason, "**")]
                ///
                $(#[$attr])*
                ///
                #[doc = concat!("[MDN Docs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/", $status, ")")]
                $name
            ),*,
            /// Custom status code.
            /// Only used when the status code is not a valid HTTP status code.
            Custom(u16)
        }

        impl Status {
            /// Gets the actual HTTP status code for the status.
            pub fn code(&self) -> u16 {
                match self {
                    $(Status::$name => $status),*,
                    Status::Custom(x) => *x
                }
            }

            /// Gets the default reason phrase for the status.
            /// For responses you can use the [`crate::Response::reason`] method to set a custom reason phrase.
            pub fn reason_phrase(&self) -> &str {
                match self.code() {
                    $($status => $reason),*,
                    _ => "OK"
                }
            }
        }

        impl From<u16> for Status {
            fn from(code: u16) -> Self {
                match code {
                    $($status => Status::$name),*,
                    x => Status::Custom(x)
                }
            }
        }
    };
}

status! {
    // == Informational ==

    /// Indicates that everything so far is OK and that the client should continue with the request or ignore it if it is already finished.
    /// To have a server check the request's headers, a client must send Expect: 100-continue as a header in its initial request and receive a 100 Continue status code in response before sending the body
    Continue           => 100, "Continue",
    /// Indicates a protocol to which the server switches. The protocol is specified in the Upgrade request header received from a client.
    /// The server includes in this response an Upgrade response header to indicate the protocol it switched to.
    SwitchingProtocols => 101, "Switching Protocols",
    /// Sent by a server while it is still preparing a response, with hints about the resources that the server is expecting the final response will link.
    /// This allows a browser to start preloading resources even before the server has prepared and sent that final response.
    EarlyHints         => 103, "Early Hints",

    // == Success ==

    /// Success status response code indicates that the request has succeeded.
    /// A 200 response is cacheable by default.
    Ok                          => 200, "OK",
    /// Indicates that the request has succeeded and has led to the creation of a resource.
    /// The new resource, or a description and link to the new resource, is effectively created before the response is sent back and the newly created items are returned in the body of the message, located at either the URL of the request, or at the URL in the value of the Location header.
    Created                     => 201, "Created",
    /// Indicates that the request has been accepted for processing, but the processing has not been completed; in fact, processing may not have started yet.
    /// The request might or might not eventually be acted upon, as it might be disallowed when processing actually takes place.
    Accepted                    => 202, "Accepted",
    /// Indicates that the request was successful but the enclosed payload has been modified by a transforming proxy from that of the origin server's 200 (OK) response.
    NonAuthoritativeInformation => 203, "Non-Authoritative Information",
    /// Indicates that a request has succeeded, but that the client doesn't need to navigate away from its current page.
    NoContent                   => 204, "No Content",
    /// Tells the client to reset the document view, so for example to clear the content of a form, reset a canvas state, or to refresh the UI.
    ResetContent                => 205, "Reset Content",
    /// Indicates that the request has succeeded and the body contains the requested ranges of data, as described in the Range header of the request.
    PartialContent              => 206, "Partial Content",

    // == Redirection ==

    /// Indicates that the request has more than one possible response.
    /// The user-agent or the user should choose one of them.
    /// As there is no standardized way of choosing one of the responses, this response code is very rarely used.
    MultipleChoices   => 300, "Multiple Choices",
    /// Redirect status response code indicates that the requested resource has been definitively moved to the URL given by the Location headers.
    /// A browser redirects to the new URL and search engines update their links to the resource.
    MovedPermanently  => 301, "Moved Permanently",
    /// Indicates that the resource requested has been temporarily moved to the URL given by the Location header.
    /// A browser redirects to this page but search engines don't update their links to the resource (in 'SEO-speak', it is said that the 'link-juice' is not sent to the new URL).
    Found             => 302, "Found",
    /// Indicates that the redirects don't link to the requested resource itself, but to another page (such as a confirmation page, a representation of a real-world object or an upload-progress page).
    /// This response code is often sent back as a result of PUT or POST.
    /// The method used to display this redirected page is always GET.
    SeeOther          => 303, "See Other",
    /// Indicates that there is no need to retransmit the requested resources.
    /// It is an implicit redirection to a cached resource.
    /// This happens when the request method is a safe method, such as GET or HEAD, or when the request is conditional and uses an If-None-Match or an If-Modified-Since header.
    NotModified       => 304, "Not Modified",
    /// Indicates that the resource requested has been temporarily moved to the URL given by the Location headers.
    TemporaryRedirect => 307, "Temporary Redirect",
    /// Indicates that the resource requested has been definitively moved to the URL given by the Location headers.
    /// A browser redirects to this page and search engines update their links to the resource (in 'SEO-speak', it is said that the 'link-juice' is sent to the new URL).
    PermanentRedirect => 308, "Permanent Redirect",

    // == Client Error ==

    /// Indicates that the server cannot or will not process the request due to something that is perceived to be a client error (for example, malformed request syntax, invalid request message framing, or deceptive request routing).
    BadRequest                  => 400, "Bad Request",
    /// Indicates that the client request has not been completed because it lacks valid authentication credentials for the requested resource.
    /// This status code is sent with an HTTP WWW-Authenticate response header that contains information on how the client can request for the resource again after prompting the user for authentication credentials.
    Unauthorized                => 401, "Unauthorized",
    /// Nonstandard response status code that is reserved for future use.
    /// This status code was created to enable digital cash or (micro) payment systems and would indicate that the requested content is not available until the client makes a payment.
    PaymentRequired             => 402, "Payment Required",
    /// Indicates that the server understands the request but refuses to authorize it.
    Forbidden                   => 403, "Forbidden",
    /// Indicates that the server cannot find the requested resource.
    /// Links that lead to a 404 page are often called broken or dead links and can be subject to link rot.
    /// A truly *iconic* HTTP status code.
    NotFound                    => 404, "Not Found",
    /// indicates that the server knows the request method, but the target resource doesn't support this method.
    /// The server must generate an Allow header field in a 405 status code response.
    /// The field must contain a list of methods that the target resource currently supports.
    MethodNotAllowed            => 405, "Method Not Allowed",
    /// Indicates that the server cannot produce a response matching the list of acceptable values defined in the request's proactive content negotiation headers, and that the server is unwilling to supply a default representation.
    NotAcceptable               => 406, "Not Acceptable",
    /// Indicates that the server cannot produce a response matching the list of acceptable values defined in the request's proactive content negotiation headers, and that the server is unwilling to supply a default representation.
    ProxyAuthenticationRequired => 407, "Proxy Authentication Required",
    /// Indicates that the server would like to shut down this unused connection. It is sent on an idle connection by some servers, even without any previous request by the client.
    /// A server should send the "close" Connection header field in the response, since 408 implies that the server has decided to close the connection rather than continue waiting.
    RequestTimeOut              => 408, "Request Time-out",
    /// Indicates a request conflict with the current state of the target resource.
    /// Conflicts are most likely to occur in response to a PUT request.
    /// For example, you may get a 409 response when uploading a file that is older than the existing one on the server, resulting in a version control conflict.
    Conflict                    => 409, "Conflict",
    /// Indicates that access to the target resource is no longer available at the origin server and that this condition is likely to be permanent.
    /// If you don't know whether this condition is temporary or permanent, a 404 status code should be used instead.
    Gone                        => 410, "Gone",
    /// Indicates that the server refuses to accept the request without a defined Content-Length header.
    LengthRequired              => 411, "Length Required",
    /// Indicates that access to the target resource has been denied.
    /// This happens with conditional requests on methods other than GET or HEAD when the condition defined by the If-Unmodified-Since or If-None-Match headers is not fulfilled.
    /// In that case, the request, usually an upload or a modification of a resource, cannot be made and this error response is sent back.
    PreconditionFailed          => 412, "Precondition Failed",
    /// Indicates that the request entity is larger than limits defined by server;
    /// the server might close the connection or return a Retry-After header field.
    /// Prior to RFC 9110 the response phrase for the status was Payload Too Large.
    /// That name is still widely used.
    ContentTooLarge             => 413, "Content Too Large",
    /// Indicates that the URI requested by the client is longer than the server is willing to interpret.
    URITooLarge                 => 414, "Request-URI Too Large",
    /// Indicates that the server refuses to accept the request because the payload format is in an unsupported format.
    /// The format problem might be due to the request's indicated Content-Type or Content-Encoding, or as a result of inspecting the data directly.
    UnsupportedMediaType        => 415, "Unsupported Media Type",
    /// Indicates that a server cannot serve the requested ranges.
    /// The most likely reason is that the document doesn't contain such ranges, or that the Range header value, though syntactically correct, doesn't make sense.
    /// The 416 response message contains a Content-Range indicating an unsatisfied range (that is a '*') followed by a '/' and the current length of the resource.
    /// E.g. `Content-Range: bytes */12777`.
    RangeNotSatisfiable         => 416, "Requested range not satisfiable",
    /// Indicates that the expectation given in the request's Expect header could not be met.
    ExpectationFailed           => 417, "Expectation Failed",
    /// Indicates that the server refuses to brew coffee because it is, permanently, a teapot.
    /// A combined coffee/tea pot that is temporarily out of coffee should instead return 503.
    /// This error is a reference to Hyper Text Coffee Pot Control Protocol defined in April Fools' jokes in 1998 and 2014.
    ImaTeapot                   => 418, "I'm a teapot",
    /// Indicates that the request was directed to a server that is not able to produce a response.
    /// This might be possible if a connection is reused or if an alternative service is selected.
    MisdirectedRequest          => 421, "Misdirected Request",
    /// Indicates that the server understands the content type of the request entity, and the syntax of the request entity is correct, but it was unable to process the contained instructions.
    UnprocessableContent        => 422, "Unprocessable Content",
    /// Indicates that the server is unwilling to risk processing a request that might be replayed, which creates the potential for a replay attack.
    TooEarly                    => 425, "Too Early",
    /// Indicates that the server refuses to perform the request using the current protocol but might be willing to do so after the client upgrades to a different protocol.
    UpgradeRequired             => 426, "Upgrade Required",
    /// Indicates that the server requires the request to be conditional.
    /// Typically, this means that a required precondition header, such as If-Match, is missing.
    /// When a precondition header is not matching the server side state, the response should be 412 Precondition Failed.
    PreconditionRequired        => 428, "Precondition Required",
    /// Response status code indicates the user has sent too many requests in a given amount of time ("rate limiting").
    /// A Retry-After header might be included to this response indicating how long to wait before making a new request.
    TooManyRequests             => 429, "Too Many Requests",
    /// Indicates that the server refuses to process the request because the request's HTTP headers are too long.
    /// The request may be resubmitted after reducing the size of the request headers.
    RequestHeaderFieldsTooLarge => 431, "Request Header Fields Too Large",
    /// Indicates that the user requested a resource that is not available due to legal reasons, such as a web page for which a legal action has been issued.
    UnavailableForLegalReasons  => 451, "Unavailable For Legal Reasons",

    // == Server Error ==

    /// Indicates that the server encountered an unexpected condition that prevented it from fulfilling the request.
    InternalServerError           => 500, "Internal Server Error",
    /// Indicates that the server does not support the functionality required to fulfill the request.
    NotImplemented                => 501, "Not Implemented",
    /// Indicates that the server, while acting as a gateway or proxy, received an invalid response from the upstream server.
    BadGateway                    => 502, "Bad Gateway",
    /// Indicates that the server is not ready to handle the request
    ServiceUnavailable            => 503, "Service Unavailable",
    /// Indicates that the server, while acting as a gateway or proxy, did not get a response in time from the upstream server that it needed in order to complete the request.
    GatewayTimeOut                => 504, "Gateway Time-out",
    /// Indicates that the HTTP version used in the request is not supported by the server.
    HTTPVersionNotSupported       => 505, "HTTP Version not supported",
    /// Code may be given in the context of Transparent Content Negotiation (see RFC 2295).
    /// This protocol enables a client to retrieve the best variant of a given resource, where the server supports multiple variants.
    VariantAlsoNegotiates         => 506, "Variant Also Negotiates",
    /// A client may send a request that contains an extension declaration, that describes the extension to be used.
    /// If the server receives such a request, but any described extensions are not supported for the request, then the server responds with the 510 status code.
    NotExtended                   => 510, "Not Extended",
    /// Indicates that the client needs to authenticate to gain network access.
    /// This status is not generated by origin servers, but by intercepting proxies that control access to the network.
    NetworkAuthenticationRequired => 511, "Network Authentication Required"
}
