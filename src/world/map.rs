// use parking_lot::Mutex;

pub struct Map {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl Map {
    pub fn from(width: u32, height: u32, data: Vec<u8>) -> Map {
        Map {
            data,
            width,
            height,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn block(&self, x: u32, y: u32) -> u8 {
        self.data[(x + y * self.width) as usize]
    }

    // TODO: Add various utility method to reduce locking/unlocking.
}
