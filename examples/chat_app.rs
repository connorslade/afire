use std::{
    error::Error,
    net::Ipv4Addr,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{sync_channel, SyncSender},
        RwLock,
    },
    thread,
};

use afire::{
    internal::sync::ForceLockRwLock,
    trace::{set_log_level, Level},
    websocket::{TxType, WebSocketExt},
    Method, Server,
};

struct App {
    clients: RwLock<Vec<Client>>,
}

struct Client {
    id: u64,
    sender: SyncSender<String>,
}

// Instead of using `Result<(), Box<dyn Error>>` you should use anyhow::Result
fn main() -> Result<(), Box<dyn Error>> {
    set_log_level(Level::Trace);

    // Create a server on ivp4 localhost port 8080 with 4 workers
    let mut server = Server::builder(Ipv4Addr::LOCALHOST, 8080, App::new())
        .workers(4)
        .build()?;

    // Root route to serve the client html + js
    server.route(Method::GET, "/", |ctx| Ok(ctx.text(HTML).send()?));

    // Websocket route to handle chat
    server.route(Method::GET, "/api/chat", |ctx| {
        // Convert the socket to a websocket. This automatically performs the handshake.
        // We then call `split` to separate the websocket stream into a sender and receiver.
        let (ws_tx, ws_rx) = ctx.ws()?.split();
        let (tx, rx) = sync_channel(10);

        // Create a client with a channel to send messages to the websocket and add it to the app
        let client = Client::new(tx);
        let id = client.id;
        ctx.app().add_client(client);

        // Send a welcome message with the client's id
        ws_tx.send(format!("[SYSTEM] Your id is {id}. Welcome!"));

        // Proxy messages from the channel to the websocket
        let this_ws_tx = ws_tx.clone();
        thread::spawn(move || {
            for i in rx {
                if !this_ws_tx.is_open() {
                    break;
                }

                this_ws_tx.send(i);
            }
        });

        // Proxy messages from the websocket to the channel to be sent to other clients
        let app = ctx.app();
        thread::spawn(move || {
            for i in ws_rx.into_iter() {
                match i {
                    TxType::Close => break,
                    TxType::Binary(_) => ws_tx.send("[SYSTEM] Binary is not supported"),
                    TxType::Text(t) => app.message(format!("[{id}] {t}"), id),
                }
            }

            // If the socket is closed
            app.remove_client(id);
        });

        Ok(())
    });

    // Start the server
    server.run()?;
    Ok(())
}

impl App {
    fn new() -> Self {
        Self {
            clients: RwLock::new(Vec::new()),
        }
    }

    fn message(&self, msg: String, sender: u64) {
        println!("{}", msg);
        let clients = self.clients.force_read();
        for client in clients.iter().filter(|c| c.id != sender) {
            client.sender.send(msg.clone()).unwrap();
        }
    }

    fn add_client(&self, client: Client) {
        self.message(format!("[SYSTEM] {} joined", client.id), client.id);
        self.clients.force_write().push(client);
    }

    fn remove_client(&self, id: u64) {
        let mut clients = self.clients.force_write();
        clients.retain(|c| c.id != id);
        drop(clients);
        self.message(format!("[SYSTEM] {} left", id), id);
    }
}

impl Client {
    fn new(sender: SyncSender<String>) -> Self {
        static ID: AtomicU64 = AtomicU64::new(0);

        Self {
            id: ID.fetch_add(1, Ordering::Relaxed),
            sender,
        }
    }
}

const HTML: &str = r#"
<!DOCTYPE html>
<html>
<body>
    <div messages></div>

    <form>
        <input type="text" />
        <button type="submit">Send</button>
    </form>

    <script>
        // Modified From https://stackoverflow.com/questions/10406930
        let new_uri =
            location.origin.replace("http://", "ws://").replace("https://", "wss://") +
            "/api/chat";

        const ws = new WebSocket(new_uri);
        const messages = document.querySelector("[messages]");
        const form = document.querySelector("form");
        const input = document.querySelector("input");

        let id = null;
        ws.onmessage = (e) => {
            if (id === null) id = e.data.split(" ")[4].slice(0, -1);
            const p = document.createElement("p");
            p.innerText = e.data;
            messages.appendChild(p);
        };

        form.onsubmit = (e) => {
            e.preventDefault();
            const p = document.createElement("p");
            p.innerText = `[${id}] ${input.value}`;
            messages.appendChild(p);
            ws.send(input.value);
            input.value = "";
        };
    </script>
</body>
</html>"#;
