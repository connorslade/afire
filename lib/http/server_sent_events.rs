//! [Server-sent event](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events) (SSE) support.

use std::{
    fmt::Display,
    io::{self, Write},
    sync::mpsc::{self, Sender},
    thread,
};

use crate::Request;

/// A server-sent event stream.
pub struct ServerSentEvents(Sender<Event>);

/// An event that can be sent as a [server-sent event](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events).
pub struct Event {
    id: Option<u32>,
    event: String,
    data: String,
}

impl ServerSentEvents {
    /// Sends a new event with the given event type and data.
    pub fn send(&self, event_type: impl AsRef<str>, data: impl Display) {
        let _ = self.0.send(Event::new(event_type).data(data));
    }

    /// Sends a new event with the given event type and id.
    pub fn send_id(&self, event_type: impl AsRef<str>, id: u32, data: impl Display) {
        let _ = self.0.send(Event::new(event_type).id(id).data(data));
    }

    /// Sends a new event with an Event struct.
    pub fn send_event(&self, event: Event) {
        let _ = self.0.send(event);
    }

    fn from_request(this: &Request) -> io::Result<Self> {
        let socket = this.socket.clone();
        socket.lock().unwrap().write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\n\r\n")?;

        let (tx, rx) = mpsc::channel::<Event>();
        thread::Builder::new()
            .name("SSE worker".to_owned())
            .spawn(move || {
                for event in rx {
                    let _ = socket
                        .lock()
                        .unwrap()
                        .write_all(event.to_string().as_bytes());
                }
            })
            .unwrap();

        Ok(Self(tx))
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
    fn sse(&self) -> io::Result<ServerSentEvents>;
}

impl ServerSentEventsExt for Request {
    fn sse(&self) -> io::Result<ServerSentEvents> {
        ServerSentEvents::from_request(self)
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
