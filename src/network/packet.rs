extern crate byteorder;

use byteorder::WriteBytesExt;

pub fn inform_world(
	width: u32,
	height: u32,
	data: &Vec<u8>,
	players: &Vec<super::super::world::player::Player>,
) -> Vec<u8> {
	let mut packet = Vec::with_capacity(
		2 + 4 + 4 + data.len() + (8 + 4 + 1 + 1 + 1 + 1 + 4 + 4) * players.len(),
	);

	packet.write_u16::<byteorder::LittleEndian>(1).unwrap();

	packet.write_u32::<byteorder::LittleEndian>(width).unwrap();
	packet.write_u32::<byteorder::LittleEndian>(height).unwrap();
	packet.extend(data);

	packet
		.write_u32::<byteorder::LittleEndian>(players.len() as u32)
		.unwrap();

	{
		let player = players.last().unwrap();

		packet
			.write_u64::<byteorder::LittleEndian>(player.id())
			.unwrap();

		let mut glyph = vec![0; 4];
		let encoded_length = player.glyph().encode_utf8(&mut glyph).len() as u8;

		packet.extend(glyph);
		packet.push(encoded_length);
		packet.push(player.color().0);
		packet.push(player.color().1);
		packet.push(player.color().2);
		packet
			.write_i32::<byteorder::LittleEndian>(player.object().x)
			.unwrap();
		packet
			.write_i32::<byteorder::LittleEndian>(player.object().x)
			.unwrap();
	}

	for player in players.iter().take(players.len() - 1) {
		packet
			.write_u64::<byteorder::LittleEndian>(player.id())
			.unwrap();

		let mut glyph = vec![0; 4];
		let encoded_length = player.glyph().encode_utf8(&mut glyph).len() as u8;

		packet.extend(glyph);
		packet.push(encoded_length);
		packet.push(player.color().0);
		packet.push(player.color().1);
		packet.push(player.color().2);
		packet
			.write_i32::<byteorder::LittleEndian>(player.object().x)
			.unwrap();
		packet
			.write_i32::<byteorder::LittleEndian>(player.object().y)
			.unwrap();
	}

	packet
}

pub fn player_income(player: &super::super::world::player::Player) -> Vec<u8> {
	let mut packet = Vec::with_capacity(2 + 8 + 4 + 1 + 1 + 1 + 1 + 4 + 4);

	packet.write_u16::<byteorder::LittleEndian>(2).unwrap();

	packet
		.write_u64::<byteorder::LittleEndian>(player.id())
		.unwrap();

	let mut glyph = vec![0; 4];
	let encoded_length = player.glyph().encode_utf8(&mut glyph).len() as u8;

	packet.extend(glyph);
	packet.push(encoded_length);
	packet.push(player.color().0);
	packet.push(player.color().1);
	packet.push(player.color().2);
	packet
		.write_i32::<byteorder::LittleEndian>(player.object().x)
		.unwrap();
	packet
		.write_i32::<byteorder::LittleEndian>(player.object().y)
		.unwrap();

	packet
}

pub fn player_exit(player: u64) -> Vec<u8> {
	let mut packet = Vec::with_capacity(2 + 8);

	packet.write_u16::<byteorder::LittleEndian>(3).unwrap();

	packet.write_u64::<byteorder::LittleEndian>(player).unwrap();

	packet
}
