use bytes::{Bytes, BytesMut};

#[derive(Debug)]
pub struct RawPacket {
    code: u16,
    payload: BytesMut,
}

impl RawPacket {
    pub fn new<P: AsRef<[u8]>>(code: u16, payload: P) -> Self {
        let mut bytes = BytesMut::with_capacity(payload.as_ref().len());
        bytes.extend(payload.as_ref());

        Self {
            code,
            payload: bytes,
        }
    }

    pub fn freeze(self) -> (u16, Bytes) {
        (self.code, self.payload.freeze())
    }
}
