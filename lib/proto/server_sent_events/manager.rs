use std::{
    fmt::Display,
    sync::{atomic::AtomicBool, Arc, RwLock},
    thread::{self, JoinHandle},
};

use crate::{
    context::ContextFlag,
    error::AnyResult,
    internal::{
        socket::Socket,
        sync::{ForceLockMutex, ForceLockRwLock},
    },
    response::ResponseFlag,
    Context,
};

use super::{handshake, Event};

#[derive(Clone)]
pub struct SseManager<EventHandler> {
    inner: Arc<SseManagerInner<EventHandler>>,
}

struct SseManagerInner<EventHandler> {
    sockets: RwLock<Vec<Arc<Socket>>>,
    handler: EventHandler,
    dropped: AtomicBool,
}

pub trait SseEventHandler
where
    Self: Sized,
{
    fn on_connect(&self, manager: &SseManager<Self>, socket: &Socket, last_id: Option<u32>);
    fn on_disconnect(&self, manager: &SseManager<Self>, socket: &Socket);
}

// todo: disconnect / garbage collect on socket disconnect (poll)

impl<EventHandler: SseEventHandler> SseManager<EventHandler> {
    pub fn new(handler: EventHandler) -> Self {
        let this = Self {
            inner: Arc::new(SseManagerInner {
                sockets: RwLock::new(Vec::new()),
                handler,
                dropped: AtomicBool::new(false),
            }),
        };

        // let cloned = this.clone();
        // thread::spawn(move || {
        //     let mut
        // });

        this
    }

    pub fn handle<State: Send + Sync>(&self, ctx: &Context<State>) -> AnyResult<()> {
        ctx.flags.set(ContextFlag::ResponseSent);
        ctx.req.socket.set_flag(ResponseFlag::End);
        ctx.req.socket.set_raw(true);

        let last_index = handshake(&ctx.req, &ctx.server.config.default_headers)?;
        let socket = ctx.req.socket.clone();

        self.inner.sockets.force_write().push(socket.clone());
        self.inner.handler.on_connect(&self, &socket, last_index);

        Ok(())
    }

    pub fn send_event_all(&self, event: Event) {
        for socket in self.inner.sockets.force_read().iter() {
            let _ = socket
                .socket
                .force_lock()
                .write_all(&event.to_string().as_bytes());
        }
    }

    pub fn send_event(&self, event: Event, socket_id: u64) {
        let sockets = self.inner.sockets.force_read();
        let socket = sockets.iter().find(|x| x.id == socket_id);

        if let Some(socket) = socket {
            let _ = socket
                .socket
                .force_lock()
                .write_all(&event.to_string().as_bytes());
        }
    }

    pub fn send_all(&self, event: impl AsRef<str>, data: impl Display) {
        let event = Event::new(event.as_ref()).data(data.to_string());
        self.send_event_all(event);
    }

    pub fn send_id_all(&self, event: impl AsRef<str>, data: impl Display, id: u32) {
        let event = Event::new(event.as_ref()).data(data.to_string()).id(id);
        self.send_event_all(event);
    }

    pub fn send(&self, event: impl AsRef<str>, data: impl Display, socket_id: u64) {
        let event = Event::new(event.as_ref()).data(data.to_string());
        self.send_event(event, socket_id);
    }

    pub fn send_id(&self, event: impl AsRef<str>, data: impl Display, id: u32, socket_id: u64) {
        let event = Event::new(event.as_ref()).data(data.to_string()).id(id);
        self.send_event(event, socket_id);
    }
}
