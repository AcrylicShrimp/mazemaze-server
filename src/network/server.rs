use super::client_task::{client_read_task, client_write_task};
use super::network_client_mgr::NetworkClientManager;
use super::{super::world::world::World, ClientManager};
use crate::network::ServerDriver;
use parking_lot::Mutex;
use std::net::Shutdown;
use std::sync::Arc;
use std::{io::Error as IOError, ops::DerefMut};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{
    error::SendError as MpscSendError, unbounded_channel as mpsc_channel,
    UnboundedReceiver as MpscReceiver, UnboundedSender as MpscSender,
};
use tokio::sync::oneshot::{
    channel as oneshot_channel, error::RecvError as OneshotRecvError, Receiver as OneshotReceiver,
    Sender as OneshotSender,
};
use tokio::task::JoinHandle;

pub struct Server {
    driver: Arc<dyn ServerDriver>,
    exit_signal_sender: OneshotSender<()>,
    handle: JoinHandle<Result<(), ServerError>>,
}

#[derive(Debug)]
pub enum ServerError {
    IOError(IOError),
}

impl From<IOError> for ServerError {
    fn from(err: IOError) -> Self {
        Self::IOError(err)
    }
}

impl Server {
    pub fn start<S: Into<String>, D: ServerDriver>(host: S, port: u16, driver: D) -> Self {
        let host = host.into();
        let driver = Arc::new(driver);
        let client_manager = Arc::new(ClientManager::new());

        let cloned_driver = driver.clone();
        let (exit_signal_sender, exit_signal_receiver) = oneshot_channel();
        let handle = tokio::spawn(async move {
            let listener = TcpListener::bind(format!("{}:{}", host, port)).await?;
            run_task(
                cloned_driver,
                listener,
                client_manager,
                exit_signal_receiver,
            )
            .await;
            Ok(())
        });

        Self {
            driver,
            exit_signal_sender,
            handle,
        }
    }

    pub fn stop(self) -> JoinHandle<Result<(), ServerError>> {
        self.exit_signal_sender.send(()).ok();
        self.handle
    }
}

async fn run_task(
    driver: Arc<dyn ServerDriver>,
    listener: TcpListener,
    client_manager: Arc<ClientManager>,
    exit_signal_receiver: OneshotReceiver<()>,
) {
    let (termination_signal_sender, termination_signal_receiver) = mpsc_channel();

    let (killer_exit_signal_sender, killer_exit_signal_receiver) = oneshot_channel();
    let (listener_exit_signal_sender, listener_exit_signal_receiver) = oneshot_channel();

    let killer_handle = tokio::spawn(killer_task(
        driver.clone(),
        client_manager.clone(),
        killer_exit_signal_receiver,
        termination_signal_receiver,
    ));
    let listener_handle = tokio::spawn(listener_task(
        driver,
        listener,
        client_manager.clone(),
        termination_signal_sender,
        listener_exit_signal_receiver,
    ));

    exit_signal_receiver.await.ok();

    listener_exit_signal_sender.send(()).ok();
    tokio::join!(listener_handle).0.ok();

    // TODO: Send a server closed packet to all clients.
    client_manager.for_each_client(|client| {
        match client.io_pump_exit_signal_sender().deref_mut().take() {
            Some(sender) => {
                sender.send(()).ok();
            }
            None => {}
        }
    });

    killer_exit_signal_sender.send(()).ok();
    tokio::join!(killer_handle).0.ok();
}

async fn killer_task(
    driver: Arc<dyn ServerDriver>,
    client_manager: Arc<ClientManager>,
    mut exit_signal_receiver: OneshotReceiver<()>,
    mut termination_signal_receiver: MpscReceiver<usize>,
) {
    loop {
        tokio::select! {
            _ = &mut exit_signal_receiver => {
                break;
            }
            id = termination_signal_receiver.recv() => match id {
                Some(id) => if client_manager.remove_client(id) {
                    driver.on_client_out(id).await;
                }
                None => break,
            }
        }
    }
}

async fn listener_task(
    driver: Arc<dyn ServerDriver>,
    listener: TcpListener,
    client_manager: Arc<ClientManager>,
    termination_signal_sender: MpscSender<usize>,
    mut exit_signal_receiver: OneshotReceiver<()>,
) -> Result<(), ServerError> {
    loop {
        tokio::select! {
            _ = &mut exit_signal_receiver => {
                break;
            }
            result = listener.accept() => {
                let (stream, addr) = result?;
                stream.set_nodelay(true)?;
                let client = client_manager.add_client(stream, termination_signal_sender.clone());
                driver.on_client_in(client, addr).await;
            }
        }
    }

    Ok(())
}

// pub fn start_new(host: &'static str, port: u16) -> JoinHandle<()> {
//     tokio::spawn(async move {
//         // TODO: Set-up server primitives here.
//         let network_client_mgr = Arc::new(NetworkClientManager::with_capacity(16));
//         let world = Arc::new(Mutex::new(World::new()));

//         let mut listener = match TcpListener::bind(format!("{}:{}", host, port)).await {
//             Ok(listener) => listener,
//             Err(err) => panic!("unable to bind server at {}:{} due to: {}", host, port, err),
//         };

//         loop {
//             let (stream, address) = match listener.accept().await {
//                 Ok(client) => client,
//                 Err(err) => {
//                     eprintln!("client connection aborted due to: {}", err);
//                     continue;
//                 }
//             };

//             println!("new client connected from {}", address);

//             network_client_mgr.new_client(network_client_mgr.clone(), world.clone(), stream);
//         }
//     })
// }

// pub fn start(
//     sockets: std::sync::Arc<std::sync::Mutex<Vec<super::socket::Socket>>>,
//     world: std::sync::Arc<std::sync::Mutex<super::super::world::world::World>>,
//     handler: std::sync::Arc<std::sync::Mutex<super::handler::Handler>>,
//     host: &str,
//     port: u16,
// ) {
//     crossbeam::scope(|s| {
//         s.spawn(|_| loop {
//             {
//                 let mut sockets = sockets.lock().unwrap();
//                 let mut world = world.lock().unwrap();
//                 let mut handler = handler.lock().unwrap();

//                 for index in (0..sockets.len()).rev() {
//                     if !sockets[index].update() {
//                         println!("current players: {}", sockets.len());
//                         println!("player exit: {}, {}", index, sockets[index].id());

//                         sockets[index]
//                             .stream()
//                             .shutdown(std::net::Shutdown::Both)
//                             .or_else(|_| std::io::Result::Ok(()))
//                             .unwrap();

//                         handler.remove_socket(index, &mut sockets, &mut world);
//                         sockets.remove(index);
//                     }
//                 }
//             }

//             std::thread::sleep(std::time::Duration::from_millis(1u64));

//             {
//                 let mut sockets = sockets.lock().unwrap();
//                 let mut world = world.lock().unwrap();
//                 let mut handler = handler.lock().unwrap();

//                 handler.handle_sockets(&mut sockets, &mut world);
//             }

//             std::thread::sleep(std::time::Duration::from_millis(1u64));
//         });
//         s.spawn(|_| {
//             let listener = std::net::TcpListener::bind(format!("{}:{}", host, port)).unwrap();

//             println!("listening...");

//             for stream in listener.incoming() {
//                 match stream {
//                     Ok(stream) => {
//                         if stream.set_nodelay(false).is_err() {
//                             continue;
//                         }

//                         if stream.set_nonblocking(true).is_err() {
//                             continue;
//                         }

//                         let mut sockets = sockets.lock().unwrap();
//                         let mut handler = handler.lock().unwrap();

//                         let mut socket = super::socket::Socket::from(stream);

//                         handler.add_socket(&mut socket);
//                         sockets.push(socket);

//                         println!("client income...");
//                     }
//                     Err(..) => {}
//                 }
//             }
//         });
//     })
//     .unwrap();
// }
