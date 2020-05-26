pub struct Map {
    data: Vec<u8>,
    width: u32,
    height: u32,
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

    pub fn get_block(&self, x: u32, y: u32) -> u8 {
        self.data[(x + y * self.width) as usize]
    }
}
