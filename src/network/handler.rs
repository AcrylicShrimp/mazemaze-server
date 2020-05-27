extern crate byteorder;

use super::packet;
use byteorder::ReadBytesExt;

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

    pub fn remove_socket(
        &mut self,
        index: usize,
        sockets: &mut Vec<super::socket::Socket>,
        world: &mut super::super::world::world::World,
    ) {
        self.status.remove(&sockets[index].id());
        world.remove_player(sockets[index].id());

        let packet = packet::player_exit(sockets[index].id());

        for socket_index in 0..sockets.len() {
            if socket_index == index {
                continue;
            }

            sockets[socket_index].send(packet.clone());
        }
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

        self.handle_packet(status_code, index, sockets, world);
    }

    fn handle_packet(
        &mut self,
        status: u16,
        index: usize,
        sockets: &mut Vec<super::socket::Socket>,
        world: &mut super::super::world::world::World,
    ) {
        println!("index={}, status={}", index, status);

        match status {
            1 => {
                world.add_player(sockets[index].id());

                sockets[index].send(packet::inform_world(
                    world.map().width(),
                    world.map().height(),
                    world.map().data(),
                    world.players(),
                ));

                let player_income_packet = packet::player_income(world.players().last().unwrap());

                for socket_index in 0..sockets.len() {
                    if socket_index == index {
                        continue;
                    }

                    sockets[socket_index].send(player_income_packet.clone());
                }

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
