use std::error::Error;
use std::fs;
use std::io::BufWriter;
use std::path::Path;

use image::codecs::{jpeg::JpegDecoder, png::PngDecoder, tiff::TiffDecoder, webp::WebPDecoder};
use image::{io::Reader, DynamicImage, ImageDecoder, ImageFormat};

/// Write pixel art with the color profile from its source file.
pub fn output(
    img: &DynamicImage,
    source_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let icc_profile = source_profile(source_path)?;
    let path = Path::new(output_path);

    let dir = path.parent().unwrap();
    fs::create_dir_all(dir)?;

    let Some(profile) = icc_profile else {
        img.save(output_path)?;
        return Ok(());
    };

    if ImageFormat::from_path(path)? != ImageFormat::Png {
        return Err("Use a .png output to preserve the source image's color profile".into());
    }

    let pixels = img.to_rgba8();
    let mut info = png::Info::with_size(pixels.width(), pixels.height());
    info.color_type = png::ColorType::Rgba;
    info.bit_depth = png::BitDepth::Eight;
    info.icc_profile = Some(profile.into());

    let file = BufWriter::new(fs::File::create(path)?);
    let mut encoder = png::Encoder::with_info(file, info)?;
    encoder.set_compression(png::Compression::Fast);
    encoder.set_filter(png::FilterType::Sub);
    encoder.set_adaptive_filter(png::AdaptiveFilterType::Adaptive);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(pixels.as_raw())?;
    writer.finish()?;

    Ok(())
}

pub fn source_profile(path: &str) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
    let reader = Reader::open(path)?.with_guessed_format()?;
    let profile = match reader.format() {
        Some(ImageFormat::Png) => PngDecoder::new(reader.into_inner())?.icc_profile(),
        Some(ImageFormat::Jpeg) => JpegDecoder::new(reader.into_inner())?.icc_profile(),
        Some(ImageFormat::Tiff) => TiffDecoder::new(reader.into_inner())?.icc_profile(),
        Some(ImageFormat::WebP) => WebPDecoder::new(reader.into_inner())?.icc_profile(),
        _ => None,
    };

    if let Some(profile) = &profile {
        // Pixel art uses RGB channels, so a grayscale or CMYK profile cannot describe its output.
        if profile.get(16..20) != Some(b"RGB ") {
            return Err(
                "Convert the source to an RGB color profile before rendering pixel art".into(),
            );
        }
    }

    Ok(profile)
}
