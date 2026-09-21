use std::error::Error;

use image::{DynamicImage, GenericImage, GenericImageView, Rgba};

/// Append the visible palette colors without changing the original image pixels.
pub fn with_palette_bar(
    image: &DynamicImage,
    palette: &[Rgba<u8>],
    bar_height: u32,
) -> Result<DynamicImage, Box<dyn Error>> {
    if bar_height == 0 {
        return Err("Palette bar height must be greater than zero".into());
    }

    let mut colors: Vec<_> = palette.iter().filter(|color| color[3] != 0).collect();

    if colors.is_empty() {
        return Err("A palette bar needs at least one visible color".into());
    }

    colors.sort_by_key(|color| swatch_sort_key(**color));

    let (width, height) = image.dimensions();

    if colors.len() > width as usize {
        return Err("The image is too narrow to show every palette color; increase --size".into());
    }

    let output_height = height
        .checked_add(bar_height)
        .ok_or("The image and palette bar exceed the maximum image height")?;
    let mut output = DynamicImage::new_rgba8(width, output_height);
    output.copy_from(image, 0, 0)?;

    for x in 0..width {
        // Integer division fills the width with swatches that differ by at most one pixel.
        let color_index = (u64::from(x) * colors.len() as u64 / u64::from(width)) as usize;
        let color = *colors[color_index];

        for y in height..output_height {
            output.put_pixel(x, y, color);
        }
    }

    Ok(output)
}

fn swatch_sort_key(color: Rgba<u8>) -> (u8, u32, [u8; 4]) {
    let [red, green, blue, _] = color.0;
    let brightest = red.max(green).max(blue);
    let darkest = red.min(green).min(blue);
    let chroma = f64::from(brightest - darkest);

    // Keep six hue families together; gray shades follow the colored swatches.
    let hue_group = if chroma == 0.0 {
        6
    } else {
        let red = f64::from(red);
        let green = f64::from(green);
        let blue = f64::from(blue);
        let brightest = f64::from(brightest);

        let hue = if brightest == red {
            ((green - blue) / chroma).rem_euclid(6.0)
        } else if brightest == green {
            (blue - red) / chroma + 2.0
        } else {
            (red - green) / chroma + 4.0
        };

        hue.floor() as u8
    };

    // Sort dark to light within each family, with RGB values breaking any ties.
    let brightness = 299 * u32::from(red) + 587 * u32::from(green) + 114 * u32::from(blue);

    (hue_group, brightness, color.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    #[test]
    fn bar_preserves_the_image_and_splits_an_uneven_width() {
        let original_color = Rgba([30, 60, 90, 128]);
        let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(5, 2, original_color));
        let red = Rgba([255, 0, 0, 255]);
        let blue = Rgba([0, 0, 255, 255]);
        let transparent = Rgba([0, 0, 0, 0]);

        let output = with_palette_bar(&image, &[red, transparent, blue], 3).unwrap();

        assert_eq!(output.dimensions(), (5, 5));

        for y in 0..2 {
            for x in 0..5 {
                assert_eq!(output.get_pixel(x, y), original_color);
            }
        }

        for y in 2..5 {
            for (x, color) in [red, red, red, blue, blue].into_iter().enumerate() {
                assert_eq!(output.get_pixel(x as u32, y), color);
            }
        }
    }

    #[test]
    fn every_color_gets_a_pixel_when_the_image_is_narrow() {
        let image = DynamicImage::new_rgba8(2, 1);
        let palette = [Rgba([10, 20, 30, 255]), Rgba([40, 50, 60, 255])];

        let output = with_palette_bar(&image, &palette, 1).unwrap();

        assert_eq!(output.get_pixel(0, 1), palette[0]);
        assert_eq!(output.get_pixel(1, 1), palette[1]);
        assert!(with_palette_bar(&DynamicImage::new_rgba8(1, 1), &palette, 1).is_err());
    }

    #[test]
    fn swatches_group_hues_and_order_shades_independently_of_palette_order() {
        let image = DynamicImage::new_rgba8(6, 1);
        let dark_gold = Rgba([100, 80, 10, 255]);
        let light_gold = Rgba([240, 220, 120, 255]);
        let dark_blue = Rgba([10, 40, 80, 255]);
        let light_blue = Rgba([160, 200, 240, 255]);
        let black = Rgba([0, 0, 0, 255]);
        let white = Rgba([255, 255, 255, 255]);
        let palette = [light_blue, white, light_gold, dark_blue, black, dark_gold];
        let expected = [dark_gold, light_gold, dark_blue, light_blue, black, white];

        let output = with_palette_bar(&image, &palette, 1).unwrap();
        let mut reversed = palette;
        reversed.reverse();
        let reversed_output = with_palette_bar(&image, &reversed, 1).unwrap();

        for (x, color) in expected.into_iter().enumerate() {
            assert_eq!(output.get_pixel(x as u32, 1), color);
            assert_eq!(reversed_output.get_pixel(x as u32, 1), color);
        }
    }

    #[test]
    fn invalid_heights_and_empty_palettes_return_errors() {
        let image = DynamicImage::new_rgba8(2, 1);
        let palette = [Rgba([10, 20, 30, 255])];

        assert!(with_palette_bar(&image, &palette, 0).is_err());
        assert!(with_palette_bar(&image, &palette, u32::MAX).is_err());
        assert!(with_palette_bar(&image, &[], 1).is_err());
        assert!(with_palette_bar(&image, &[Rgba([0, 0, 0, 0])], 1).is_err());
    }
}
