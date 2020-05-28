extern crate byteorder;

use super::packet;
use byteorder::ReadBytesExt;

pub enum Context {
    DirectionReceive,
}

pub struct Handler {
    status: std::collections::HashMap<u64, Option<u16>>,
    context: std::collections::HashMap<u64, Option<Context>>,
}

impl Handler {
    pub fn new() -> Handler {
        Handler {
            status: std::collections::HashMap::new(),
            context: std::collections::HashMap::new(),
        }
    }

    pub fn add_socket(&mut self, socket: &mut super::socket::Socket) {
        socket.receive(2);
        self.status.insert(socket.id(), None);
        self.context.insert(socket.id(), None);
    }

    pub fn remove_socket(
        &mut self,
        index: usize,
        sockets: &mut Vec<super::socket::Socket>,
        world: &mut super::super::world::world::World,
    ) {
        self.status.remove(&sockets[index].id());
        self.context.remove(&sockets[index].id());

        if !world.remove_player(sockets[index].id()) {
            return;
        }

        let packet = packet::player_exit(sockets[index].id());

        for socket_index in 0..sockets.len() {
            if socket_index == index {
                continue;
            }

            sockets[socket_index].send(packet.clone());

            break;
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

                    break;
                }

                sockets[index].receive(2);
                self.status.insert(sockets[index].id(), None);
                self.context.insert(sockets[index].id(), None);
            }
            2 => {
                if self.context[&sockets[index].id()].is_none() {
                    sockets[index].receive(1);
                    sockets[index].update();
                    self.context
                        .insert(sockets[index].id(), Some(Context::DirectionReceive));
                }

                match self.context[&sockets[index].id()] {
                    Some(..) => match sockets[index].retrieve() {
                        Some(received) => {
                            for player in world.players_mut().iter_mut() {
                                if player.id() != sockets[index].id() {
                                    continue;
                                }

                                match received[0] {
                                    0 => {
                                        player.object_mut().y -= 1;
                                    }
                                    1 => {
                                        player.object_mut().y += 1;
                                    }
                                    2 => {
                                        player.object_mut().x -= 1;
                                    }
                                    3 => {
                                        player.object_mut().x += 1;
                                    }
                                    _ => {}
                                }

                                break;
                            }

                            let packet = packet::player_move(sockets[index].id(), received[0]);

                            for socket in sockets.iter_mut() {
                                socket.send(packet.clone());
                            }

                            sockets[index].receive(2);
                            self.status.insert(sockets[index].id(), None);
                            self.context.insert(sockets[index].id(), None);
                        }
                        None => {}
                    },
                    None => {
                        sockets[index].receive(1);
                        self.context
                            .insert(sockets[index].id(), Some(Context::DirectionReceive));
                    }
                }
            }
            _ => {
                sockets[index].receive(2);
                self.status.insert(sockets[index].id(), None);
                self.context.insert(sockets[index].id(), None);
            }
        }
    }
}
