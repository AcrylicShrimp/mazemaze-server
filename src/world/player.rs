use super::object;

use byteorder::WriteBytesExt;

pub struct Player {
	id: u64,
	glyph: char,
	color: (u8, u8, u8),
	object: object::Object,
}

impl Player {
	pub fn new(id: u64, x: i32, y: i32) -> Player {
		let mut bytes = vec![];

		bytes.write_u64::<byteorder::LittleEndian>(id).unwrap();

		Player {
			id,
			glyph: '@',
			color: (
				std::cmp::min(bytes[0] as u32 + 64, 255) as u8,
				std::cmp::min(bytes[1] as u32 + 64, 255) as u8,
				std::cmp::min(bytes[2] as u32 + 64, 255) as u8,
			),
			object: object::Object::new(x, y),
		}
	}

	pub fn id(&self) -> u64 {
		self.id
	}

	pub fn glyph(&self) -> char {
		self.glyph
	}

	pub fn color(&self) -> (u8, u8, u8) {
		self.color
	}

	pub fn object(&self) -> &object::Object {
		&self.object
	}

	pub fn object_mut(&mut self) -> &mut object::Object {
		&mut self.object
	}
}
