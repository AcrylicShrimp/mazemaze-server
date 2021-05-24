use super::client::{Client, ClientId};
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::net::TcpStream;

pub struct ClientManager {
    clients: Mutex<Vec<Arc<Mutex<Client>>>>,
}

impl ClientManager {
    pub fn new() -> ClientManager {
        ClientManager {
            clients: Mutex::new(Vec::new()),
        }
    }

    pub fn add(&mut self, stream: TcpStream) -> Arc<Mutex<Client>> {
        let client = Client::from_stream(stream);
        self.clients.lock().push(client.clone());
        client
    }

    pub fn remove(&mut self, id: ClientId) {
        let mut clients = self.clients.lock();
        let clients_len = clients.len();

        for index in 0..clients_len {
            if clients[index].lock().id() != id {
                continue;
            }

            let tmp = clients[index].clone();
            clients[index] = clients[clients_len - 1].clone();
            clients[clients_len - 1] = tmp;

            clients.pop();

            break;
        }
    }
}
