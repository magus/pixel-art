use core::fmt::Debug;
use std::fmt;

#[derive(Default, Debug)]
pub struct Crop {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

impl Crop {
    pub fn width(&self) -> u32 {
        self.right - self.left + 1
    }

    pub fn height(&self) -> u32 {
        self.bottom - self.top + 1
    }
}

impl fmt::Display for Crop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "[{}x{}] {:?} ", self.width(), self.height(), self)
    }
}
