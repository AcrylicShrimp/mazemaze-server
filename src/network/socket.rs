extern crate rand;

use rand::RngCore;
use std::io::Read;
use std::io::Write;

pub struct Transmitter {
    sent: usize,
    queue: std::collections::VecDeque<Vec<u8>>,
}

impl Transmitter {
    pub fn new() -> Transmitter {
        Transmitter {
            sent: 0,
            queue: std::collections::VecDeque::new(),
        }
    }

    pub fn send(&mut self, data: Vec<u8>) {
        self.queue.push_front(data);
    }

    pub fn update(&mut self, stream: &mut std::net::TcpStream) -> bool {
        while !self.queue.is_empty() {
            let data = self.queue.front().unwrap();

            match stream.write(&data[self.sent..]) {
                Ok(sent) => {
                    self.sent += sent;

                    if self.sent == data.len() {
                        self.sent = 0;
                        self.queue.pop_front();
                    } else {
                        break;
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(..) => {
                    return false;
                }
            }
        }

        true
    }
}

pub struct Receiver {
    received: usize,
    buffer: Option<Vec<u8>>,
}

impl Receiver {
    pub fn new() -> Receiver {
        Receiver {
            received: 0,
            buffer: None,
        }
    }

    pub fn retrieve(&mut self) -> Option<Vec<u8>> {
        match &self.buffer {
            Some(buffer) => {
                if self.received == buffer.len() {
                    self.received = 0;
                    self.buffer.take()
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn receive(&mut self, length: usize) {
        self.received = 0;
        self.buffer = Some(vec![0; length]);
    }

    pub fn update(&mut self, stream: &mut std::net::TcpStream) -> bool {
        match &mut self.buffer {
            Some(buffer) => {
                if self.received == buffer.len() {
                    true
                } else {
                    match stream.read(&mut buffer[self.received..]) {
                        Ok(received) => {
                            self.received += received;
                            true
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => true,
                        Err(..) => false,
                    }
                }
            }
            None => true,
        }
    }
}

pub struct Socket {
    id: u64,
    stream: std::net::TcpStream,
    tx: Transmitter,
    rx: Receiver,
}

impl Socket {
    pub fn from(stream: std::net::TcpStream) -> Socket {
        let mut rng = rand::prelude::thread_rng();

        Socket {
            id: rng.next_u64(),
            stream,
            tx: Transmitter::new(),
            rx: Receiver::new(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn stream(&mut self) -> &mut std::net::TcpStream {
        &mut self.stream
    }

    pub fn send(&mut self, data: Vec<u8>) {
        self.tx.send(data);
    }

    pub fn receive(&mut self, length: usize) {
        self.rx.receive(length);
    }

    pub fn retrieve(&mut self) -> Option<Vec<u8>> {
        self.rx.retrieve()
    }

    pub fn update(&mut self) -> bool {
        if !self.tx.update(&mut self.stream) {
            return false;
        }

        if !self.rx.update(&mut self.stream) {
            return false;
        }

        true
    }
}
