use std::{
    io,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::{
    internal::encoding::{base64, sha1},
    HeaderType, Request, Response, Status,
};

const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub struct WebSocketStream {
    _rx: Receiver<EventType>,
    _tx: Sender<EventType>,
}

enum EventType {}

impl WebSocketStream {
    pub fn from_request(req: &Request) -> io::Result<Self> {
        let ws_key = req.headers.get("Sec-WebSocket-Key").unwrap().to_owned();
        trace!("WS Key: {}", ws_key);
        let accept = base64::encode(&sha1::hash((ws_key + WS_GUID).as_bytes()));
        trace!("WS Accept: {}", accept);

        let mut upgrade = Response::new()
            .status(Status::SwitchingProtocols)
            .header(HeaderType::Upgrade, "websocket")
            .header(HeaderType::Connection, "Upgrade")
            .header("Sec-WebSocket-Accept", &accept)
            .header("Sec-WebSocket-Version", "13");
        upgrade.write(req.socket.clone(), &[]).unwrap();

        let (s2c, _rx) = mpsc::channel::<EventType>();
        let (_tx, c2s) = mpsc::channel::<EventType>();

        // todo: everything else :sweat_smile:

        Ok(Self { _rx: c2s, _tx: s2c })
    }
}

trait WebSocketExt {
    fn ws(&self) -> io::Result<WebSocketStream>;
}

impl WebSocketExt for Request {
    fn ws(&self) -> io::Result<WebSocketStream> {
        WebSocketStream::from_request(self)
    }
}
