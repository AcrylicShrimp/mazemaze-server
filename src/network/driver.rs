use net::{Client, ServerDriver};
use crate::world::world::World;
use crate::network::packet_builder::*;
use async_trait::async_trait;
use parking_lot::Mutex;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::join;
use std::collections::HashMap;
use tokio::task::JoinHandle;

pub struct Driver {
    clients: Mutex<HashMap<usize, JoinHandle<()>>>,
    world: Arc<Mutex<World>>,
}

impl Driver {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new().into(),
            world: Mutex::new(World::new()).into(),
        }
    }
}

#[async_trait]
impl ServerDriver for Driver {
    async fn on_client_in(&self, client: Arc<Client>, addr: SocketAddr) {
        let mut receiver = match client.tx_receiver().take() {
            Some(receiver) => receiver,
            None => {
                if let Some(sender) = client.io_pump_exit_signal_sender().take() {
                    sender.send(()).ok();
                }
                return;
            }
        };

        let client_id = client.id();
        let world = self.world.clone();

        self.clients.lock().insert(
            client_id,
            tokio::spawn(async move {
                {
                    let mut world = world.lock();
                    world.add_player(client_id as _);
                    client
                        .tx_sender()
                        .send(inform_world(
                            world.map().width(),
                            world.map().height(),
                            world.map().data(),
                            world.players(),
                        ))
                        .ok();
                }

                loop {
                    let (code, payload) = match receiver.recv().await {
                        Some(packet) => packet.freeze(),
                        None => break,
                    };

                    println!("code: {}", code);

                    match code {
                        1 => {
                            
                        }
                        _ => {}
                    }
                }

                world.lock().remove_player(client.id() as _);
            }),
        );
        
        println!("[client in] {} : {}", client_id, addr);
    }

    async fn on_client_out(&self, client_id: usize) {
        let handle;
        
        {
            let mut clients = self.clients.lock();
            handle = clients.remove(&client_id);
        }

        match handle {
            Some(handle) => {
                join!(handle).0.ok();
            }
            None => {}
        }

        println!("[client out] {}", client_id);
    }
}
