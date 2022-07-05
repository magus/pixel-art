use std::error::Error;

use image::Pixel;

use pixel_art::image as pixel_art_image;

fn main() -> Result<(), Box<dyn Error>> {
    // Use the open function to load an image from a Path.
    // `open` returns a `DynamicImage` on success.
    let img = image::open("./images/pikachu.png").unwrap();
    // let img = image::open("./images/charizard.png").unwrap();
    // let img = image::open("./images/venusaur.png").unwrap();

    let img = pixel_art_image::zealous_crop(&img);
    pixel_art_image::output(&img, "./output/cropped.png")?;

    // draw image to cli
    pixel_art_image::print(&img);

    let colors = pixel_art_image::palette(&img);

    let (width, height) = img.dimensions();
    let output_size = 8;
    let window_size = width / output_size;

    for x in 0..width {
        for y in 0..height {
            // break up image into windows based on output bit size
            // e.g. 8x8 pixel art would break image into 8x8 grid
            // for each grid cell (window), calculate most frequent color
            // color that cell with that color in final output

            let x_window = x / window_size;
            let y_window = y / window_size;

            // println!("({},{})=({},{})", x, y, x_window, y_window);

            let pixel = img.get_pixel(x, y);
            if let [r, g, b, alpha] = pixel.channels() {
                if *alpha == 0 {
                    // count transparent pixel
                } else {
                    // find closest color match
                    // increment counter
                }
            };
        }
    }

    return Ok(());
}
