use crate::world::player::Player;
use bytes::{BufMut, Bytes, BytesMut};
use std::mem::size_of;
use std::ops::{Deref, DerefMut};

pub struct PacketBuilder {
    packet: BytesMut,
}

impl PacketBuilder {
    pub fn new(code: u16, payload_size: usize) -> Self {
        let mut packet = BytesMut::with_capacity(size_of::<u16>() * 2 + payload_size);
        packet.put_u16(code);
        packet.put_u16(payload_size as u16);

        Self { packet }
    }

    pub fn into_packet(self) -> Bytes {
        self.packet.freeze()
    }
}

impl Deref for PacketBuilder {
    type Target = BytesMut;

    fn deref(&self) -> &Self::Target {
        &self.packet
    }
}

impl DerefMut for PacketBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.packet
    }
}

pub fn inform_world(width: u32, height: u32, data: &Vec<u8>, players: &Vec<Player>) -> Bytes {
    let mut packet = PacketBuilder::new(
        1,
        size_of::<u32>()
            + size_of::<u32>()
            + size_of::<u8>() * data.len()
            + (8 + 4 + 1 + 1 + 1 + 1 + 4 + 4) * players.len(),
    );

    packet.put_u32_le(width);
    packet.put_u32_le(height);
    packet.extend(data);

    packet.put_u32_le(players.len() as u32);

    {
        let player = players.last().unwrap();

        packet.put_u64_le(player.id());

        let mut glyph = vec![0; 4];
        let encoded_length = player.glyph().encode_utf8(&mut glyph).len() as u8;

        packet.extend(glyph);
        packet.put_u8(encoded_length);
        packet.put_u8(player.color().0);
        packet.put_u8(player.color().1);
        packet.put_u8(player.color().2);
        packet.put_i32_le(player.object().x);
        packet.put_i32_le(player.object().y);
    }

    for player in players.iter().take(players.len() - 1) {
        packet.put_u64_le(player.id());

        let mut glyph = vec![0; 4];
        let encoded_length = player.glyph().encode_utf8(&mut glyph).len() as u8;

        packet.extend(glyph);
        packet.put_u8(encoded_length);
        packet.put_u8(player.color().0);
        packet.put_u8(player.color().1);
        packet.put_u8(player.color().2);
        packet.put_i32_le(player.object().x);
        packet.put_i32_le(player.object().y);
    }

    packet.into_packet()
}
