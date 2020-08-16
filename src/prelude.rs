#[derive(Debug)]
pub struct OutputInfo {
    pub version: u8,
    pub height: u32,
    pub width: u32,
    pub fps: u16,
}

pub type Bytevec = Vec<u8>;
pub type Frame = Vec<[u8; 3]>;
