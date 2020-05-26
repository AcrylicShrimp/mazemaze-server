extern crate crossbeam;

pub struct Server {
    sockets: std::sync::Mutex<Vec<super::socket::Socket>>,
}

impl Server {
    pub fn new() -> Server {
        Server {
            sockets: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn start(&self, host: &str, port: u16) {
        crossbeam::scope(|s| {
            s.spawn(|_| {
                self.loop_stream();
            });
        })
        .unwrap();

        let listener = std::net::TcpListener::bind(format!("{}:{}", host, port)).unwrap();

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if stream.set_nodelay(true).is_err() {
                        continue;
                    }

                    if stream.set_nonblocking(true).is_err() {
                        continue;
                    }

                    self.sockets
                        .lock()
                        .unwrap()
                        .push(super::socket::Socket::from(stream));
                }
                Err(..) => {}
            }
        }
    }

    fn loop_stream(&self) {
        loop {
            let mut sockets = self.sockets.lock().unwrap();

            for index in (0..sockets.len()).rev() {
                if !sockets[index].update() {
                    sockets[index]
                        .stream()
                        .shutdown(std::net::Shutdown::Both)
                        .or_else(|_| std::io::Result::Ok(()))
                        .unwrap();
                    sockets.remove(index);
                }
            }
        }
    }
}
