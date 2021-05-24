use super::super::world::world::World;
use super::network_client_mgr::NetworkClientManager;
use super::packet::*;
use bytes::Bytes;
use parking_lot::Mutex;
use std::io::Result;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::select;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::oneshot::Receiver;

pub async fn client_write_task(mut writer: OwnedWriteHalf, mut rx: UnboundedReceiver<Bytes>) {
    loop {
        match rx.recv().await {
            Some(packet) => match writer.write_all(&packet).await {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("writing failure: {}", err);
                    break;
                }
            },
            None => break,
        }
    }
}

pub async fn client_read_task(
    reader: OwnedReadHalf,
    mut termination_signal_receiver: Receiver<()>,
    client_index: usize,
    network_client_mgr: Arc<NetworkClientManager>,
    world: Arc<Mutex<World>>,
) -> Result<()> {
    let mut reader = BufReader::with_capacity(128, reader);

    select! {
        _ = &mut termination_signal_receiver => return Ok(()),
        packet = reader.read_u16_le() => match packet? {
            1 => {
                // let packet;

                // {
                // 	let mut world = world.lock();
                // 	world.add_player(client_index as u64);
                // 	packet = inform_world(
                // 		world.map().width(),
                // 		world.map().height(),
                // 		world.map().data(),
                // 		world.players(),
                // 	);
                // }

                // network_client_mgr.send_packet(client_index, packet);
            }
            2 => {
                println!("2");
            }
            _ => {
                tokio::spawn(async move {
                    network_client_mgr.remove_client(client_index).await;
                });
            }
        }
    };

    Ok(())
}
