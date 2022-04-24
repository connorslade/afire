# 🔥 afire Examples

## Basic

Basic examples give you a first look into using afire.
Run `cargo r --example basic` to use the example picker menu thing.
You can also run a specific example with `cargo r --example basic -- <EXAMPLE_NAME>`

| Name           | Description                                                            |
| -------------- | ---------------------------------------------------------------------- |
| basic          | Start a basic web server that serves static text.                      |
| serve_file     | Serve a file from the local file system.                               |
| routeing       | Learn about routing priority and create a 404 page                     |
| data           | Send data to server with a Query String, Path parameters and Form Data |
| header         | Make and Read Headers to send extra data                               |
| error_handling | Catch panics in routes and middleware                                  |
| serve_static   | Serve a whole directory                                                |
| path_params    | Use Path Parameters on a route                                         |
| middleware     | Use Middleware to log requests                                         |
| cookie         | Read and Write cookies to the client                                   |
| logging        | Log requests to a file or console                                      |
| rate_limit     | Add a rate limit to your server                                        |
| threading      | Use a thread pool to handle requests                                   |

## Application

These are more complete examples, still not a full web app through.
For more complete web apps you can reference the [Things Built with afire](https://connorcode.com/writing/afire#things-built-with-afire) section on my website.
To run these application examples you can use this command `cargo r --example applcation_<EXAMPLE_NAME>`.

| Name      | Description                              |
| --------- | ---------------------------------------- |
| paste_bin | A very simple in memory paste bin system |
