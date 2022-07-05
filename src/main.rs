use pixel_art::image as pixel_art_image;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Use the open function to load an image from a Path.
    // `open` returns a `DynamicImage` on success.
    // let img = image::open("./images/pikachu.png").unwrap();
    // let img = image::open("./images/charizard.png").unwrap();
    let img = image::open("./images/venusaur.png").unwrap();

    let cropped_img = pixel_art_image::zealous_crop(&img);
    pixel_art_image::output(&cropped_img, "./output/cropped.png")?;

    return Ok(());
}
