use std::error::Error;
use std::fs;
use std::path::Path;

use image::RgbaImage;

pub fn output(img: &RgbaImage, output_path: &str) -> Result<(), Box<dyn Error>> {
    let path = Path::new(output_path);
    let dir = path.parent().unwrap();
    fs::create_dir_all(dir)?;
    img.save(output_path)?;

    return Ok(());
}
