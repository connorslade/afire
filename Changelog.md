# 3.0.0

Coming Soon

- The thread pool can now be resized at runtime.
- `Server::start_threaded` has been replaced with `Server::workers` to set the number of worker threads and the normal `Server::start`.
- Supply a context to all route handlers.
  The context contains a reference to the server, request, and acts as a response builder.
  Note: This also allows access to the thread_pool for both executing tasks and resizing it.
- Remove the `stateful_route` method, use normal routes with `ctx.app()` instead.
- You can now send responses before the route handler ends.
  This allows you to easily send a response and then do some work after the response is sent as to not block the client.
- Responses can be sent from other threads.
  You will have to notify the handler that you will be sending a response after the handler function returns with `ctx.guarantee_will_send()`.
- Allow returning any error type from route handlers.
  This is done with a return type of `Result<(), Box<dyn Error>>`.
- Allow attaching extra context to an error with [anyhow](https://crates.io/crates/anyhow) like functions: `context`, `with_context`, `status`, `with_status`, `header`, `with_header`.
- Remove `StartupError::NoState` error.
- Make `ForceLockMutex`, `ForceLockRwLock`, and `SingleBarrier` public through the `internal::sync` module.
  - The force lock traits are used to lock a `Mutex` or `RwLock` even if it is poisoned.
  - The `SingleBarrier` is used to have a thread wait until another thread unlocks the barrier.
    The difference from a normal Barrier is that it does not block on both sides, only the waiting side.
- Replace the `TcpStream` in `Request` with `Socket`, a wrapper struct.
  Its used to allow automatically the barrier with a response is sent.
  In the future it might also be used for optional TLS support.
- Move path parameters into Context and use a HashMap instead of a Vec.
- Parse the HTTP version into an enum instead of a string.
- Properly determine weather to use keep-alive or not.
  In HTTP/1.1 keep-alive is opt-out, but previous versions of afire assumed it was opt-in.
  Fixing this produced a 2x performance increase in the benchmarks I ran.
  Now, this is still only a ~500us improvement but hey.
- Filter CLRF characters from headers. This prevents a potential [response splitting attack](https://datatracker.ietf.org/doc/html/rfc7230#section-9.4).
- Properly disallow HTTP/1.1 requests with no Host header.
- Added a new ResponseBody type of Empty.
- Added a `current_thread` function to the threadpool.
- Catch panics at the thread-pool level, not the route handler level.
  This will ensure that a worker will not die, even if internal afire code panics.
- Use an Arc instead of a Rc for `HandleError::Panic`
- Add `with_response` function on Context to overwrite the response.
- Finish implementing Websockets. :tada:
- Add chat app example application.
- Rename `Server::start` to `Server::run` to emphasize that it blocks.
- Pass `Arguments` to trace formatters instead of a String.
  This can be more efficient if the formatter decides not to format the trace.
- Attach a unique ID to each socket.

# 2.2.1

August 20, 2023

- Properly support `ErrorKind::Interrupted` on streaming responses (#46).
  Previously if a Reader returned any error, afire would just print an error and close the socket.
- Build extension docs on docs.rs

# 2.2.0

July 02, 2023

- Use binary search on ServeStatic MMIE types (save those clock cycles)
- Some optimizations throughout afire
- Logger can now make use of the `RealIp` extension and log the real ip of the client
- Logger now holds a persistent file handle instead of opening and closing it every time
- In ServeStatic, when paths have the '..', they will now go up a directory instead of being ignored
  Note: The highest you can can go is the data directory that you define, so there is no path traversal vulnerability
- Accept `impl Into<HeaderType>` in `RequestId::new` instead of just `AsRef<str>`.
  This allows for using `HeaderType`s as well as strings to set the header.
- Add a `HEAD` middleware that adds support for the HTTP HEAD method.
- Update `ServeStatic` to send a Content-Length header when streaming a file.
- Add a `TRACE` middleware that adds support for the HTTP TRACE method.
- Add support for [Server-Sent Events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events) (SSE).
- Progress on Websocket support

# 2.1.0

April 24, 2023

- Added a get_query method on Query
- Changed default log level back to Error
- Response flags (Close & End)
- More built-in encoding systems (base64 & sha-1)
- Change encoding system module format
- Multipart request parsing
- CookieJar struct for holding Cookies in the request
- RealIp extension
- Allow serving an IPv6 addr
- Use a `Headers` struct to hold default headers
- Added a HeaderParams struct
- Impl ToHostAddress for &String
- Add Server::app to get a reference to the app
- Increase ServeStatic compatibility with other middleware
- Custom log formatter support
- Optional emoji in logging
- Fix the Display impl on Query
- Add body_str method to Request
- Impl std::error::Error for afire::Error
- Impl Display for error types
- Don't execute format on lower log-levels
- Fix spelling errors
- Fix Logger middleware always appending `?` to the path
- Don't consider sockets closing to be an error (only printed in debug tracing)
- Mild performance improvements in the path matcher with catch-all routes

# 2.0.0

February 11, 2023

- Fix improper URL decoding behavior
- Improve Memory Usage On `Request`s
- Internal code cleanup
- More clear info on IO errors
- Make SocketAddr accessible from Request
- Remade social share image
- Let ServeStatic::new take in strings and paths (previously only strings)
- Remove unnecessary feature flags (cookies, path_patterns, dynamic_resize, path_decode_url)
- More clear info on IO errors
- Improve Memory Usage On `Request`s
- Less cloning internally
- Make SocketAddr accessible from Request
- New error types: Startup / Stream
- Date middleware in extensions
- Another middleware rewrite
- Util module
- All Content variants use charset=utf-8 by default
- HeaderType enum
- Status enum
- New Header methods
- New Query methods
- Encoding module
- Server::new accepts ToHostAddress (Ipv4Addr, String, &str, [u8; 4])
- Rewrote socket handler (this is a big one)
- Trace system
- Streaming response
- Socket keep-alive!
- Request modifier
- Error handler has app state
- Panic if no app state and stateful routes
- Documentation of internal structs
- Fix improper URL decoding behavior
- Internal code cleanup
- Remade social share image
- Let ServeStatic::new take in strings and paths (previously only strings)
- Rewrote lots of documentation with spelling fixes and better code examples
- Remove unnecessary feature flags (cookies, path_patterns, dynamic_resize, path_decode_url)
- Removed cache extension
- Removed socket handler struct (don't think it was ever used)
- Removed the buff_size field from server, its handled automatically now
- Removed `set_run` on the server, its no longer needed internally

# 1.2.0

June 24, 2022

- oh windows,,,
- Fix Path Traversal on windows
- Use AsRef<str> more instead of Display
- Add a serve path to Serve Static
- Serve index from serve path
- Remove the `ignore_trailing_path_slash` feature
- Redo Internal Error handling system
- Middleware use references to Requests and Responses and stuff
- Improve built-in serve_static middleware
- Re organize extension stuff
- RateLimit use RwLock
- Add Request ID Middleware
- Server Wide State
- Add Cache Middleware
- Remove insane build script
- When building http response only add Content-Length and default headers if they are not already present
- Add server state system
- Improved Request Parsing
- Redo Error system
- Remove the requests raw_data felid
- Remove Request::body_string in favor of String::from_utf8()
- Fix HTTP parsing and generation issues

# 1.1.0

Apr 10, 2022

- Update Path Matcher to support AnyAfter segments (\*\*)
- Remove Test Example
- Add Paste Bin App Example
- Add SocketHandler struct to hold socket interacting functions
- Fix Path Traversal Exploit O_O

# 1.0.0!

Mar 14, 2022

- Add ThreadPool Back!
- Tracing Feature
- Remove Middleware Interior Mutability by Default
- Make remove_address_port usable without Feature
- Add _end_ middleware to run after sending a request
- Make use of end middleware on logger

# 0.4.0

Feb 19, 2022

- Make serve static types public
- Fix File Uploading stuff
- Add a Prelude for afire essentials
- Optimize HTTP parser _(saving nanoseconds!)_
- More unit tests
- Middleware Error Handling!
- Make Route and Error Handler functions take closures
- Remove deprecated .all function (deprecated in 0.3.0)
- Remove deprecated .ip_string function (deprecated in 0.3.0)
- Finally remove deprecated .any function (deprecated in 0.1.5)
- Route take AsRef<str> instead of any Displayable Type
- Now .header takes in a key and value rather than a Header struct

# 0.3.0

Jan 25, 2022

- Add content types
- Update Logger syntax
- Allow changing socket buffer size
- Switch Server to more of a Builder
- Add Path parameters
- Redo Path Handling
- Update README Example
- Deprecate `.all` routes, Use .route(Method::Any, "\*", ...)
- Make `content_type` a built-in (not an extension)
- Rename 'path pram' to Path Parameters
- Update Data Example with Path Parameters
- Remove Threading Example
- Update Rate limit to be a Builder
- Update Rate limit Example
- Remove thread pool from project (for now)
- Make Custom Content Type use &str not String
- Make VERSION Public
- Add External Unit Tests remove_address_port
- Use the built-in `IpAddr` enum for server Ip
- Remove `.ip_string()` for `.ip.to_string()`
- Add `Response.close()` for closing a stream with no response
- Completely Redo Middleware, Now it can modify Requests / Responses
- Removed Server.middleware()
- Export Internal Functions
- Supply the Request to the Middleware Post Function

# 0.2.2

Dec 04, 2021

- Remove Debug Print Left in...
- Small changes to features
- This is mostly about the Debug Print

# 0.2.1

Dec 04, 2021

- Only Build common::remove_address_port if logger or rate-limiter are enabled
- Make Header name / value Public
- Serve Static Middleware
- Make Routes use Closures
- Remove Threadpool (for now)
- Make Error handler use a closure
- Rename `set_error_handler` to `error_handler`
- Rename `set_socket_timeout` to `socket_timeout`
- Update Serve Static Example to use Middleware
- Allow for Manually setting the reason phrase
- Support URL encoded cookies
- Rename `add_default_header` to `default_header`
- Store Raw Request data and Request body as `Vec<u8>`
- Fix Panic Handler feature compile problems
- Don't use an Option for Vec of default headers
- Fix Header Parsing
- Add a `header` method on Request to get headers

# 0.2.0

Nov 04, 2021

- Response Overhaul, Now more like a Response Builder
- Update _every_ example with new syntax...
- Small improvement to Query parsing
- Update SetCookie Function Names
- Update Cookie Example
- Add a Build Script to write the Readme from the docstring in lib.rs
- Use Generics for more functions that take &str / Strings to just impl Display
- Rename .every to .middleware
- Update Readme in /examples
- Add a dynamic buffer resize feature
- Update Logger Middleware to be a builder
- Add Path parameter Example in 04_data

# 0.1.7

Oct 27, 2021

- Add Panic Message to Error Handel
- Add http.rs to move raw http parsing out of server.rs
- Start / Start Threaded returns Option
- Add .unwrap to all server.starts in examples
- Add http.rs to move raw http parsing out of server.rs
- Don't give up on cookie parsing if cookie header is malformed
- Add optional Socket Timeout
- Add Socket Timeout Docs

# 0.1.6

Oct 20, 2021

- Add Example for Logging
- Add Example for Rate Limiter
- Improve Rate limiter
- Add More Function Docs
- Show Query data in Info Logger
- Ignore extra slashes in path
- Remove nose.txt... don't know how that got there :P
- Don't unwrap stream.read, ignore errors like a good programmer
- Fix Routing Issue
- Ignore Case in Method String
- Add different Reason Phrase for the status codes
- Update Server Header to add Version
- Cleanup Raw HTTP Parsing
- Fix / Update some examples
- Update Logger Middleware

# 0.1.5

Sep 17, 2021

- Add a route error handler
- Add `set_error_handler` fn to set the error handler
- Implement clone for more structs
- Add More Examples
- Put default headers after route headers
- Auto decode URL encoded Queries
- Update Readme
- Proper Spelling In Description
- Add support for Request Cookies
- Add Cookies to a Feature
- Add Support for Response Cookies
- Deprecate .any routes (Use `.route(Method::ANY...)` instead)
- Make Built in middleware less garbage

# 0.1.4

Sep 05, 2021

- Allow responding with bytes, not just strings
- Add Serving Favicon as example thing

# 0.1.3

Sep 03, 2021

- Add Support for query strings in paths
- Add More docs for Query

# 0.1.2

Sep 01, 2021

- Fix a bug where '.any' routes were not working

# 0.1.1

Aug 31, 2021

- Add Optional Rate limiter _beta_
- Update readme a bit
- Copy new readme to lib.rs
- Add internal docs to Ratelimit
- Add Optional Logger _beta_
- Update Keywords in Cargo.toml
- Update Version number in Cargo.toml
- Make a function to add default headers to a server
- Don't let Logger Crash in debug mode if there are no headers
- Code Cleanup

# 0.1.0

Aug 21, 2021

- Base Code
