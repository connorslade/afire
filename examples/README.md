# 🔥 afire Examples

| Name              | Description                                           |
| ----------------- | ----------------------------------------------------- |
| 01_basic          | Start a basic web server that can serve text.         |
| 02_serve_file     | Serve a local file.                                   |
| 03_routeing       | Learn about routeing priority and add a 404 page      |
| 04_data           | Send data to server with a Query String and Form Data |
| 05_header         | Make and Read Headers to send extra data              |
| 06_error_handling | Catch panics in routes                                |
| 07_serve_static   | Serve static files from a dir                         |
| 08_middleware.rs  | Use Middleware to log requests                        |

## 01 - Basic

Create a basic web server that can serve some static text.

## 02 - Serve File

Read and server binary files from disk.

In the example a html text file is served but this code would work with images, videos, etc.

## 03 - Routeing

Learn about routeing priority and add a 404 page.

## 04 - Data

Use Query Strings and HTML forms to send data to the server from a webpage.

## 05 - Headers

Add response headers to the response to redirect to another page or send extra data.

Also read and echo client headers as a response.

## 06 - Error Handling

Learn about afire's automatic route error handling and add your own error handler.

## 07 - Serve Static

Serve all static files from a directory.

## 08 - Middleware

Learn about Middleware and how to use it to log requests.

### TODO

- Logger
- Ratelimater
