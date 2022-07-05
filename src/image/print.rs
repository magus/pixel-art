use colored::*;
use image::{Pixel, RgbaImage};

pub fn print(img: &RgbaImage) {
    let (width, height) = img.dimensions();

    // draw image to cli
    print!("\n");
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            if let [r, g, b, alpha] = pixel.channels() {
                if *alpha == 0 {
                    print!("{}", &DRAW_PIXEL);
                } else {
                    print!("{}", &DRAW_PIXEL.on_truecolor(*r, *g, *b));
                }
            };
        }
        print!("\n");
    }
}

static DRAW_PIXEL: &str = "  ";
