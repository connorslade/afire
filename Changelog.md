# 0.2.0*
- Response Overhaul, Now more like a Response Builder
- Update *every* example with new syntax...
- Small improvement to Query parseing
- Update SetCookie Function Names
- Update Cookie Example
- Add a Build Script to write the Readme from the docstring in lib.rs
- Use Generics for more functions that take &str / Strings to just impl Display

# 0.1.7
- Add Panic Message to Error Handel
- Add http.rs to move raw http parseing out of server.rs
- Start / Start Threaded returns Option
- Add .unwrap to all server.starts in examples
- Add http.rs to move raw http parsing out of server.rs
- Dont give up on cookie parsing if cookie header is malformed
- Add optinal Socket Timeout
- Add Socket Timeout Docs

# 0.1.6
- Add Example for Logging
- Add Example for Rate Limiter
- Improve Rate limiter
- Add More Function Docs
- Show Query data in Info Logger
- Ignore extra slashes in path
- Remove nose.txt... don't know how that got there :P
- Don't unwrap stream.read, ignore errors like a good programmer
- Fix Routeing Issue
- Ignore Case in Method String
- Add different Reason Phrase for the status codes
- Update Server Header to add Version
- Cleanup Raw HTTP Parsing
- Fix / Update some examples
- Update Logger Middleware

# 0.1.5
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
- Allow responding with bytes, not just strings
- Add Serving Favoricon as example thing

# 0.1.3
- Add Support for query strings in paths
- Add More docs for Query

# 0.1.2
- Fix a bug where '.any' routes were not working

# 0.1.1
- Add Optional Rate limiter *beta*
- Update readme abit
- Copy new readme to lib.rs
- Add internal docs to Ratelimit
- Add Optional Logger *beta*
- Update Keywords in Cargo.toml
- Update Version number in Cargo.toml
- Make a function to add default headers to a server
- Don't let Logger Crash in debug mode if there are no headers
- Code Cleanup
