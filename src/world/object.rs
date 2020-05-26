pub struct Object {
    pub x: i32,
    pub y: i32,
}

impl Object {
    pub fn new(x: i32, y: i32) -> Object {
        Object { x, y }
    }
}
