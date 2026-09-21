use crate::image::Sampling;

pub struct Options {
    pub input: String,
    pub output: String,
    pub size: u32,
    pub colors: usize,
    pub partitions: usize,
    pub sampling: Sampling,
    pub debug: bool,
    pub show_palette: bool,
    pub palette_height: u32,
}

impl Default for Options {
    fn default() -> Self {
        return Self {
            input: "./images/panda-bear.JPG".into(),
            output: "./output/pixelated.png".into(),
            size: 32,
            colors: 16,
            partitions: 3,
            sampling: Sampling::Mode,
            debug: false,
            show_palette: false,
            palette_height: 8,
        };
    }
}
