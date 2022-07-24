use std::error::Error;
use std::fs;
use std::path::Path;

use image::DynamicImage;

pub fn output(img: &DynamicImage, output_path: &str) -> Result<(), Box<dyn Error>> {
    let path = Path::new(output_path);
    let dir = path.parent().unwrap();
    fs::create_dir_all(dir)?;
    img.save(output_path)?;

    return Ok(());
}
