use super::super::world::world::World;
use super::client_task::{client_read_task, client_write_task};
use super::network_client_mgr::NetworkClientManager;
use parking_lot::Mutex;
use std::net::Shutdown;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;

pub fn start_new(host: &'static str, port: u16) -> JoinHandle<()> {
    tokio::spawn(async move {
        // TODO: Set-up server primitives here.
        let network_client_mgr = Arc::new(NetworkClientManager::with_capacity(16));
        let world = Arc::new(Mutex::new(World::new()));

        let mut listener = match TcpListener::bind(format!("{}:{}", host, port)).await {
            Ok(listener) => listener,
            Err(err) => panic!("unable to bind server at {}:{} due to: {}", host, port, err),
        };

        loop {
            let (stream, address) = match listener.accept().await {
                Ok(client) => client,
                Err(err) => {
                    eprintln!("client connection aborted due to: {}", err);
                    continue;
                }
            };

            println!("new client connected from {}", address);

            network_client_mgr.new_client(network_client_mgr.clone(), world.clone(), stream);
        }
    })
}

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
