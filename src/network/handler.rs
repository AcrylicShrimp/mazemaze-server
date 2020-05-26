extern crate byteorder;

use byteorder::{ReadBytesExt, WriteBytesExt};

pub struct Handler {
    status: std::collections::HashMap<u64, Option<u16>>,
}

impl Handler {
    pub fn new() -> Handler {
        Handler {
            status: std::collections::HashMap::new(),
        }
    }

    pub fn add_socket(&mut self, socket: &mut super::socket::Socket) {
        socket.receive(2);
        self.status.insert(socket.id(), None);
    }

    pub fn remove_socket(&mut self, socket: &super::socket::Socket) {
        self.status.remove(&socket.id());
    }

    pub fn handle_sockets(
        &mut self,
        sockets: &mut Vec<super::socket::Socket>,
        world: &mut super::super::world::world::World,
    ) {
        for index in 0..sockets.len() {
            self.handle_socket(index, sockets, world);
        }
    }

    fn handle_socket(
        &mut self,
        index: usize,
        sockets: &mut Vec<super::socket::Socket>,
        world: &mut super::super::world::world::World,
    ) {
        let status_code: u16;

        {
            let status = self.status.get_mut(&sockets[index].id()).unwrap();

            if status.is_none() {
                let received = sockets[index].retrieve();

                if received.is_none() {
                    return;
                }

                *status = Some(
                    std::io::Cursor::new(received.unwrap())
                        .read_u16::<byteorder::LittleEndian>()
                        .unwrap(),
                );
            }

            status_code = status.unwrap();
        }

        self.handle_packet(status_code, index, world, sockets);
    }

    fn handle_packet(
        &mut self,
        status: u16,
        index: usize,
        world: &mut super::super::world::world::World,
        sockets: &mut Vec<super::socket::Socket>,
    ) {
        println!("index={}, status={}", index, status);

        match status {
            1 => {
                let mut packet = vec![];

                packet
                    .write_u32::<byteorder::LittleEndian>(world.map().width())
                    .unwrap();
                packet
                    .write_u32::<byteorder::LittleEndian>(world.map().height())
                    .unwrap();
                packet.extend(world.map().data());

                sockets[index].send(packet);

                sockets[index].receive(2);
                self.status.insert(sockets[index].id(), None);
            }
            _ => {
                sockets[index].receive(2);
                self.status.insert(sockets[index].id(), None);
            }
        }
    }
}
