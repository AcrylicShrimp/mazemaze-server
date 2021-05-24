use bytes::{BufMut, Bytes, BytesMut};

pub fn inform_world(
    width: u32,
    height: u32,
    data: &Vec<u8>,
    players: &Vec<super::super::world::player::Player>,
) -> Bytes {
    let mut packet = BytesMut::with_capacity(
        2 + 4 + 4 + data.len() + (8 + 4 + 1 + 1 + 1 + 1 + 4 + 4) * players.len(),
    );

    packet.put_u16_le(1);

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

    packet.freeze()
}

pub fn player_income(player: &super::super::world::player::Player) -> Bytes {
    let mut packet = BytesMut::with_capacity(2 + 8 + 4 + 1 + 1 + 1 + 1 + 4 + 4);

    packet.put_u16_le(2);

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

    packet.freeze()
}

pub fn player_exit(player: u64) -> Bytes {
    let mut packet = BytesMut::with_capacity(2 + 8);

    packet.put_u16_le(3);

    packet.put_u64_le(player);

    packet.freeze()
}

pub fn player_move(player: u64, direction: u8) -> Bytes {
    let mut packet = BytesMut::with_capacity(2 + 8 + 1);

    packet.put_u16_le(4);

    packet.put_u64_le(player);

    packet.put_u8(direction);

    packet.freeze()
}
