use super::object;

pub struct Player {
	id: u64,
	glyph: char,
	color: (u8, u8, u8),
	object: object::Object,
}

impl Player {
	pub fn new(id: u64, x: i32, y: i32) -> Player {
		Player {
			id,
			glyph: '@',
			color: (0, 127, 255),
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
