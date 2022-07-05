use std::error::Error;
use std::fs;
use std::path::Path;

use image::DynamicImage;

use pixel_art::image::zealous_crop;

fn main() -> Result<(), Box<dyn Error>> {
    // Use the open function to load an image from a Path.
    // `open` returns a `DynamicImage` on success.
    // let img = image::open("./images/pikachu.png").unwrap();
    // let img = image::open("./images/charizard.png").unwrap();
    let img = image::open("./images/venusaur.png").unwrap();

    output_cropped(&img, "./output/cropped.png")?;

    return Ok(());
}

fn output_cropped(img: &DynamicImage, output_path: &str) -> Result<(), Box<dyn Error>> {
    let cropped_img = zealous_crop(&img);

    let path = Path::new(output_path);
    let dir = path.parent().unwrap();
    fs::create_dir_all(dir)?;
    cropped_img.save(output_path)?;

    return Ok(());
}
