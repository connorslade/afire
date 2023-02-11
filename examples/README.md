# 🔥 afire Examples

[![Open in GitHub Codespaces](https://github.com/codespaces/badge.svg)](https://github.com/codespaces/new?hide_repo_select=true&ref=main&repo=394493528&machine=basicLinux32gb&location=EastUs&devcontainer_path=.devcontainer%2Fdevcontainer.json)

## Basic

Basic examples give you a nice and simple first look into using afire.
Run `cargo r --example basic` to use the example picker menu thing.
You can also run a specific example with `cargo r --example basic -- <EXAMPLE_NAME>`.
The source code for each basic example can be found within the `basic` subdirectory in a file with the same name as the example.

| Name                           | Description                                                            |
| ------------------------------ | ---------------------------------------------------------------------- |
| basic                          | Start a basic web server that serves static text.                      |
| serve_file                     | Serve a file from the local file system.                               |
| routing                        | Learn about routing priority and create a 404 page                     |
| data                           | Send data to server with a Query String, Path parameters and Form Data |
| header                         | Make and Read Headers to send extra data                               |
| path_param                     | Use path parameters on a route                                         |
| state                          | Add a server-wide state and use stateful routes                        |
| cookie                         | Read and Write cookies to the client                                   |
| error_handling                 | Catch panics in routes and middleware                                  |
| serve_static                   | Staticky serve a whole directory                                       |
| middleware (show state access) | Use Middleware to log requests and modify responses                    |
| logging                        | Log requests to a file or console                                      |
| rate_limit                     | Add a rate limit to your server                                        |
| threading                      | Use a thread pool to handle requests                                   |
| trace                          | Use afire's built-in logging system                                    |

## Application

These are more complete examples, still not a full web app through.
For more complete web apps you can reference the [Things Built with afire](https://connorcode.com/writing/afire#things-built-with-afire) section on my website.
To run these application examples you can use this command `cargo r --example application_<EXAMPLE_NAME>`.

| Name       | Description                                         |
| ---------- | --------------------------------------------------- |
| paste_bin  | A very simple in memory paste bin system            |
| quote_book | A delightfully 90s website to store and view quotes |
