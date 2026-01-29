use std::fmt::Debug;

#[derive(Copy, Clone, Debug)]
pub struct CompOp {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

impl Default for CompOp {
    fn default() -> Self {
        CompOp { x: 0, y: 0, z: 0 }
    }
}

impl CompOp {
    pub fn SourceOver() -> CompOp {
        CompOp { x: 1, y: 1, z: 1 }
    }

    pub fn Plus() -> CompOp {
        CompOp { x: 2, y: 1, z: 1 }
    }

    pub fn is_source_over(self: &Self) -> bool {
        self.x == 1 && self.y == 1 && self.z == 1
    }
    pub fn is_plus(self: &Self) -> bool {
        self.x == 2 && self.y == 1 && self.z == 1
    }
}
