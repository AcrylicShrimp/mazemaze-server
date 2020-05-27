extern crate byteorder;

use byteorder::WriteBytesExt;

pub fn inform_world(
	width: u32,
	height: u32,
	data: &Vec<u8>,
	players: &Vec<(u64, super::super::world::object::Object)>,
) -> Vec<u8> {
	let mut packet = Vec::with_capacity(2 + 4 + 4 + data.len() + 16 * players.len());

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
			.write_u64::<byteorder::LittleEndian>(player.0)
			.unwrap();
		packet
			.write_i32::<byteorder::LittleEndian>(player.1.x)
			.unwrap();
		packet
			.write_i32::<byteorder::LittleEndian>(player.1.y)
			.unwrap();
	}

	for player in players.iter().take(players.len() - 1) {
		packet
			.write_u64::<byteorder::LittleEndian>(player.0)
			.unwrap();
		packet
			.write_i32::<byteorder::LittleEndian>(player.1.x)
			.unwrap();
		packet
			.write_i32::<byteorder::LittleEndian>(player.1.y)
			.unwrap();
	}

	packet
}

pub fn player_income(player: &(u64, super::super::world::object::Object)) -> Vec<u8> {
	let mut packet = Vec::with_capacity(2 + 16);

	packet.write_u16::<byteorder::LittleEndian>(2).unwrap();

	packet
		.write_u64::<byteorder::LittleEndian>(player.0)
		.unwrap();
	packet
		.write_i32::<byteorder::LittleEndian>(player.1.x)
		.unwrap();
	packet
		.write_i32::<byteorder::LittleEndian>(player.1.y)
		.unwrap();

	packet
}

pub fn player_exit(player: u64) -> Vec<u8> {
	let mut packet = Vec::with_capacity(2 + 8);

	packet.write_u16::<byteorder::LittleEndian>(3).unwrap();

	packet.write_u64::<byteorder::LittleEndian>(player).unwrap();

	packet
}
