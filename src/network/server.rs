extern crate crossbeam;

pub fn start(
    sockets: std::sync::Arc<std::sync::Mutex<Vec<super::socket::Socket>>>,
    world: std::sync::Arc<std::sync::Mutex<super::super::world::world::World>>,
    handler: std::sync::Arc<std::sync::Mutex<super::handler::Handler>>,
    host: &str,
    port: u16,
) {
    crossbeam::scope(|s| {
        s.spawn(|_| loop {
            {
                let mut sockets = sockets.lock().unwrap();
                let mut world = world.lock().unwrap();
                let mut handler = handler.lock().unwrap();

                for index in (0..sockets.len()).rev() {
                    if !sockets[index].update() {
                        sockets[index]
                            .stream()
                            .shutdown(std::net::Shutdown::Both)
                            .or_else(|_| std::io::Result::Ok(()))
                            .unwrap();

                        handler.remove_socket(index, &mut sockets, &mut world);
                        sockets.remove(index);
                    }
                }
            }

            {
                let mut sockets = sockets.lock().unwrap();
                let mut world = world.lock().unwrap();
                let mut handler = handler.lock().unwrap();

                handler.handle_sockets(&mut sockets, &mut world);
            }
        });
        s.spawn(|_| {
            let listener = std::net::TcpListener::bind(format!("{}:{}", host, port)).unwrap();

            println!("listening...");

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        if stream.set_nodelay(false).is_err() {
                            continue;
                        }

                        if stream.set_nonblocking(true).is_err() {
                            continue;
                        }

                        let mut socket = super::socket::Socket::from(stream);

                        handler.lock().unwrap().add_socket(&mut socket);
                        sockets.lock().unwrap().push(socket);

                        println!("client income...");
                    }
                    Err(..) => {}
                }
            }
        });
    })
    .unwrap();
}
