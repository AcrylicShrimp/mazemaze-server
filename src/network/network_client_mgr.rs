use super::super::world::world::World;
use super::client_task::{client_read_task, client_write_task};
use super::network_client::NetworkClient;
use bytes::Bytes;
use parking_lot::Mutex;
use std::net::Shutdown;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::join;
use tokio::net::TcpStream;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::oneshot;

pub struct NetworkClientManager {
    network_client_vec: Vec<Mutex<Option<NetworkClient>>>,
    network_client_channel_vec: Vec<Mutex<Option<UnboundedSender<Bytes>>>>,
}

impl NetworkClientManager {
    pub fn with_capacity(max_capacity: usize) -> NetworkClientManager {
        let mut network_client_vec = Vec::new();
        let mut network_client_channel_vec = Vec::new();

        for _ in 0..max_capacity {
            network_client_vec.push(Mutex::new(None));
            network_client_channel_vec.push(Mutex::new(None));
        }

        NetworkClientManager {
            network_client_vec,
            network_client_channel_vec,
        }
    }

    pub async fn new_client(
        &self,
        this: Arc<Self>,
        world: Arc<Mutex<World>>,
        mut stream: TcpStream,
    ) {
        for index in 0..self.network_client_vec.len() {
            let mut network_client = self.network_client_vec[index].lock();

            if network_client.is_some() {
                continue;
            }

            let (tx, rx) = unbounded_channel();
            let (termination_tx, termination_rx) = oneshot::channel();
            let (reader, writer) = stream.into_split();
            *network_client = Some(NetworkClient::new(
                index,
                termination_tx,
                tokio::spawn(async move {
                    client_write_task(writer, rx).await;
                }),
                tokio::spawn(async move {
                    client_read_task(reader, termination_rx, index, this, world).await;
                }),
            ));

            let mut network_client_channel = self.network_client_channel_vec[index].lock();
            *network_client_channel = Some(tx);

            return;
        }

        match stream.shutdown().await {
            Ok(()) => {}
            Err(err) => {
                eprintln!("unable to shutdown connection gracefully due to: {}", err);
            }
        }
    }

    pub async fn remove_client(&self, index: usize) -> bool {
        let handles;

        {
            let mut network_client = self.network_client_vec[index].lock();

            if network_client.is_none() {
                return false;
            }

            handles = network_client.take().unwrap().task_join_handle();

            let mut network_client_channel = self.network_client_channel_vec[index].lock();
            network_client_channel.take();
        }

        match handles.0.send(()) {
            Ok(_) => {
                let (write_join, read_join) = join!(handles.1, handles.2);
                match write_join {
                    Ok(()) => {}
                    Err(err) => eprintln!("unable to join with a write task due to: {}", err),
                }
                match read_join {
                    Ok(()) => {}
                    Err(err) => eprintln!("unable to join with a read task due to: {}", err),
                }
            }
            Err(_) => {}
        }

        println!("client connection closed by server");

        true
    }

    pub fn send_packet(&'static self, index: usize, packet: Bytes) {
        if self.network_client_channel_vec[index]
            .lock()
            .as_ref()
            .unwrap()
            .send(packet)
            .is_err()
        {
            tokio::spawn(async move {
                self.remove_client(index).await;
            });
        }
    }

    pub fn broadcast_packet(&'static self, packet: Bytes) {
        for index in 0..self.network_client_channel_vec.len() {
            // let result = match *self.network_client_channel_vec[index].lock() {
            // 	Some(network_client_channel) => network_client_channel.send(packet),
            // 	None => continue,
            // };

            // if result.is_err() {
            // 	tokio::spawn(async move {
            // 		self.remove_client(index).await;
            // 	});
            // }
        }
    }

    pub fn broadcast_packet_except(&'static self, except_index: usize, packet: Bytes) {
        for index in 0..self.network_client_channel_vec.len() {
            if index == except_index {
                continue;
            }

            // let result = match *self.network_client_channel_vec[index].lock() {
            // 	Some(network_client_channel) => network_client_channel.send(packet),
            // 	None => continue,
            // };

            // if result.is_err() {
            // 	tokio::spawn(async move {
            // 		self.remove_client(index).await;
            // 	});
            // }
        }
    }
}
