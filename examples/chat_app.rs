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

fn main() -> Result<(), Box<dyn Error>> {
    set_log_level(Level::Debug);
    let mut server = Server::new(Ipv4Addr::LOCALHOST, 8080)
        .state(App::new())
        .workers(8);

    server.route(Method::GET, "/", |ctx| Ok(ctx.text(HTML).send()?));
    server.route(Method::GET, "/api/chat", |ctx| {
        let (ws_tx, ws_rx) = ctx.ws()?.split();
        let (tx, rx) = sync_channel(10);

        let client = Client::new(tx);
        let id = client.id;
        ctx.app().add_client(client);

        ws_tx.send(format!("[SYSTEM] Your id is {id}. Welcome!"));

        let this_ws_tx = ws_tx.clone();
        thread::spawn(move || {
            for i in rx {
                if !this_ws_tx.is_open() {
                    break;
                }

                this_ws_tx.send(i);
            }
        });

        let pool_size = ctx.server.thread_pool.threads();
        ctx.server.thread_pool.resize(pool_size.saturating_add(1));

        for i in ws_rx.into_iter() {
            match i {
                TxType::Close => break,
                TxType::Binary(_) => ws_tx.send("[SYSTEM] Binary is not supported"),
                TxType::Text(t) => ctx.app().message(format!("[{id}] {t}"), id),
            }
        }

        ctx.app().remove_client(id);
        let pool_size = ctx.server.thread_pool.threads();
        ctx.server.thread_pool.resize(pool_size.saturating_sub(1));

        Ok(())
    });

    server.start()?;
    Ok(())
}

fn new_id() -> u64 {
    static ID: AtomicU64 = AtomicU64::new(0);
    ID.fetch_add(1, Ordering::Relaxed)
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
        self.message(format!("[SYSTEM] {} left", id), id);
    }
}

impl Client {
    fn new(sender: SyncSender<String>) -> Self {
        Self {
            id: new_id(),
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
        var loc = window.location, new_uri;
        if (loc.protocol === "https:") new_uri = "wss:";
        else new_uri = "ws:";
        new_uri += "//" + loc.host;
        new_uri += loc.pathname + "/api/chat";

        const ws = new WebSocket(new_uri);
        const messages = document.querySelector("[messages]");
        const form = document.querySelector("form");
        const input = document.querySelector("input");

        let id = null;
        ws.onmessage = (e) => {
            if (id === null)
                id = e.data.split(" ")[4].slice(0, -1);
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
</html>
"#;
