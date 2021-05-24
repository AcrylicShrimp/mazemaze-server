use crate::network::Client;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedSender as MpscSender;

pub struct ClientManager {
    clients: Mutex<Vec<Arc<Client>>>,
}

impl ClientManager {
    pub fn new() -> Self {
        Self {
            clients: Mutex::new(Vec::new()),
        }
    }

    pub fn add_client(
        &self,
        stream: TcpStream,
        termination_signal_sender: MpscSender<usize>,
    ) -> Arc<Client> {
        let client = Client::new(stream, termination_signal_sender);
        self.clients.lock().push(client.clone());
        client
    }

    pub fn remove_client(&self, id: usize) -> bool {
        let mut clients = self.clients.lock();
        let index = match clients.iter().position(|client| client.id() == id) {
            Some(index) => index,
            None => return false,
        };
        clients.swap_remove(index);
        true
    }

    pub fn for_each_client<F>(&self, mut f: F)
    where
        F: FnMut(&Arc<Client>),
    {
        let clients = self.clients.lock();
        for client in clients.iter() {
            f(client);
        }
    }

    pub fn for_each_client_mut<F>(&self, mut f: F)
    where
        F: FnMut(&mut Arc<Client>),
    {
        let mut clients = self.clients.lock();
        for client in clients.iter_mut() {
            f(client);
        }
    }
}
