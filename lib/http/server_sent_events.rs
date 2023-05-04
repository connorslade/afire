//! [Server-sent event](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events) (SSE) support.
//! ## Example
//! ```rust
//! # use afire::{Server, Request, Response, Method, server_sent_events::ServerSentEventsExt};
//! # use std::{thread, time::Duration};
//! # fn run(server: &mut Server) {
//! server.route(Method::GET, "/sse", |req| {
//!     let stream = req.sse().unwrap();
//!
//!     for i in 0..10 {
//!         stream.send("update", i.to_string());
//!         thread::sleep(Duration::from_secs(1));
//!     }
//!
//!     Response::end()
//! });
//! # }
//! ```
//!
//! Then in the browser you can connect to the event stream with JavaScript using the [`EventSource`](https://developer.mozilla.org/en-US/docs/Web/API/EventSource) API:
//! ```js
//! const events = new EventSource("/sse");
//! events.addEventListener("update", (event) => {
//!   console.log(event.data);
//! });
//! ```
use std::{
    fmt::Display,
    io::{self, Write},
    sync::mpsc::{self, Sender},
    thread,
};

use crate::{internal::common::ForceLock, Request};

/// A [server-sent event](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events) stream.
///
/// For more information and usage examples, visit the [module level documentation](index.html).
pub struct SSEStream {
    stream: Sender<EventType>,
    /// If the EventSource connection gets reset, the client will send the last received event id in the `Last-Event-ID` header.
    /// This will be available here, if applicable.
    pub last_index: Option<u32>,
}

/// An event that can be sent as a [server-sent event](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events).
pub struct Event {
    id: Option<u32>,
    event: String,
    data: String,
}

enum EventType {
    Event(Event),
    SetRetry(u32),
    Close,
}

impl SSEStream {
    /// Sends a new event with the given event type and data.
    pub fn send(&self, event_type: impl AsRef<str>, data: impl Display) {
        let _ = self.stream.send(Event::new(event_type).data(data).into());
    }

    /// Sends a new event with the given event type and id.
    pub fn send_id(&self, event_type: impl AsRef<str>, id: u32, data: impl Display) {
        let _ = self
            .stream
            .send(Event::new(event_type).id(id).data(data).into());
    }

    /// Sends a new event with an Event struct.
    pub fn send_event(&self, event: Event) {
        let _ = self.stream.send(event.into());
    }

    /// Sets the retry interval in milliseconds.
    /// Calling this will signal the client to try to reconnect after the given amount of milliseconds.
    pub fn set_retry(&self, retry: u32) {
        let _ = self.stream.send(EventType::SetRetry(retry));
    }

    /// Closes the SSE stream.
    /// This will leave the socket open, so a new SSEStream could be created.
    /// Note: The client will likely try to reconnect automatically after a few seconds.
    pub fn close(&self) {
        let _ = self.stream.send(EventType::Close);
    }

    /// Creates a new SSE stream from the given request.
    /// This is called automatically if you use the [`ServerSentEventsExt`] trait's .sse() method.
    pub fn from_request(this: &Request) -> io::Result<Self> {
        let last_index = this
            .headers
            .get("Last-Event-ID")
            .and_then(|id| id.parse::<u32>().ok());

        let socket = this.socket.clone();
        socket.force_lock().write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\n\r\n")?;

        let (tx, rx) = mpsc::channel::<EventType>();
        thread::Builder::new()
            .name("SSE worker".to_owned())
            .spawn(move || {
                for event in rx {
                    match event {
                        EventType::Event(e) => {
                            let _ = socket.force_lock().write_all(e.to_string().as_bytes());
                        }
                        EventType::SetRetry(retry) => {
                            let _ = socket
                                .force_lock()
                                .write_all(format!("retry: {}\n\n", retry).as_bytes());
                        }
                        EventType::Close => break,
                    }
                }
            })
            .unwrap();

        Ok(Self {
            stream: tx,
            last_index,
        })
    }
}

impl Event {
    /// Creates a new event with the given event type.
    pub fn new(event_type: impl AsRef<str>) -> Self {
        Self {
            id: None,
            event: event_type.as_ref().to_owned(),
            data: String::new(),
        }
    }

    /// Adds an id to the event.
    pub fn id(mut self, id: u32) -> Self {
        self.id = Some(id);
        self
    }

    /// Adds data to the event.
    pub fn data(mut self, data: impl Display) -> Self {
        self.data.push_str(&data.to_string());
        self
    }
}

impl ToString for Event {
    fn to_string(&self) -> String {
        let mut out = String::new();

        if let Some(id) = self.id {
            out.push_str(&format!("id: {}\n", id));
        }

        out.push_str(&format!("event: {}\n", self.event));

        for i in self.data.split('\n') {
            out.push_str(&format!("data: {}\n", i));
        }

        out.push('\n');
        out
    }
}

/// A trait for initiating a SSE connection on a request.
pub trait ServerSentEventsExt {
    /// Initiates a SSE connection on the request.
    fn sse(&self) -> io::Result<SSEStream>;
}

impl ServerSentEventsExt for Request {
    fn sse(&self) -> io::Result<SSEStream> {
        SSEStream::from_request(self)
    }
}

impl From<Event> for EventType {
    fn from(event: Event) -> Self {
        Self::Event(event)
    }
}

#[cfg(test)]
mod test {
    use super::Event;

    #[test]
    fn test_sse_event_format() {
        let event = Event::new("event");
        assert_eq!(event.to_string(), "event: event\ndata: \n\n");

        let event = Event::new("update").id(1).data("Hello");
        assert_eq!(event.to_string(), "id: 1\nevent: update\ndata: Hello\n\n");
    }
}
