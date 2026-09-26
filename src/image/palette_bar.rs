use std::error::Error;

use image::{DynamicImage, GenericImage, GenericImageView, Rgba};
use moxcms::{ColorProfile, Layout, TransformOptions};

pub struct PaletteBar<'a> {
    pub image: &'a DynamicImage,
    pub palette: &'a [Rgba<u8>],
    pub height: u32,
    /// Source ICC profile for sorting colors. None assumes sRGB.
    pub profile: Option<&'a [u8]>,
}

/// Use the source profile to order swatches, while preserving their original channel values.
pub fn with_palette_bar(bar: PaletteBar<'_>) -> Result<DynamicImage, Box<dyn Error>> {
    let PaletteBar {
        image,
        palette,
        height: bar_height,
        profile,
    } = bar;

    if bar_height == 0 {
        return Err("Palette bar height must be greater than zero".into());
    }

    let mut colors: Vec<_> = palette
        .iter()
        .copied()
        .filter(|color| color[3] != 0)
        .collect();

    if colors.is_empty() {
        return Err("A palette bar needs at least one visible color".into());
    }

    let (width, height) = image.dimensions();

    if colors.len() > width as usize {
        return Err("The image is too narrow to show every palette color; increase --size".into());
    }

    let output_height = height
        .checked_add(bar_height)
        .ok_or("The image and palette bar exceed the maximum image height")?;

    order_by_neighbors(&mut colors, profile)?;

    let mut output = DynamicImage::new_rgba8(width, output_height);
    output.copy_from(image, 0, 0)?;

    for x in 0..width {
        // Integer division fills the width with swatches that differ by at most one pixel.
        let color_index = (u64::from(x) * colors.len() as u64 / u64::from(width)) as usize;
        let color = colors[color_index];

        for y in height..output_height {
            output.put_pixel(x, y, color);
        }
    }

    Ok(output)
}

fn order_by_neighbors(
    colors: &mut [Rgba<u8>],
    profile: Option<&[u8]>,
) -> Result<(), Box<dyn Error>> {
    if colors.len() < 2 {
        return Ok(());
    }

    // Canonical input order makes ties independent of the palette's frequency order.
    colors.sort_by_key(|color| color.0);
    let labs = palette_oklab(colors, profile)?;
    let count = colors.len();
    let mut distances = vec![vec![0.0; count]; count];

    for left in 0..count {
        for right in left + 1..count {
            let distance = labs[left]
                .iter()
                .zip(labs[right])
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt();
            distances[left][right] = distance;
            distances[right][left] = distance;
        }
    }

    let mut by_lightness: Vec<_> = (0..count).collect();
    by_lightness.sort_by(|&left, &right| labs[left][0].total_cmp(&labs[right][0]));
    let darkest = by_lightness[0];
    let lightest = by_lightness[count - 1];
    let mut visited = vec![false; count];
    visited[darkest] = true;
    visited[lightest] = true;
    let mut order = vec![darkest];

    // Keep the lightest swatch for the end; visit the nearest unused color at each step.
    while order.len() < count - 1 {
        let current = *order.last().unwrap();
        let next = (0..count)
            .filter(|&index| !visited[index])
            .min_by(|&left, &right| distances[current][left].total_cmp(&distances[current][right]))
            .unwrap();

        visited[next] = true;
        order.push(next);
    }
    order.push(lightest);

    // A bounded 2-opt search smooths the greedy path while keeping both endpoints fixed.
    for _ in 0..100 {
        let mut best_gain = 0.0;
        let mut best_pair = None;

        for left in 1..count.saturating_sub(2) {
            for right in left + 1..count - 1 {
                let before = order[left - 1];
                let first = order[left];
                let last = order[right];
                let after = order[right + 1];
                let gain = distances[before][first] + distances[last][after]
                    - distances[before][last]
                    - distances[first][after];

                if gain > best_gain + 1e-12 {
                    best_gain = gain;
                    best_pair = Some((left, right));
                }
            }
        }

        let Some((left, right)) = best_pair else {
            break;
        };
        order[left..=right].reverse();
    }

    let original = colors.to_vec();
    for (color, index) in colors.iter_mut().zip(order) {
        *color = original[index];
    }

    Ok(())
}

fn palette_oklab(
    colors: &[Rgba<u8>],
    profile: Option<&[u8]>,
) -> Result<Vec<[f64; 3]>, Box<dyn Error>> {
    let mut rgb: Vec<_> = colors
        .iter()
        .flat_map(|color| color.0[..3].iter().copied())
        .collect();

    if let Some(profile) = profile {
        let source = ColorProfile::new_from_slice(profile)?;
        let transform = source.create_transform_8bit(
            Layout::Rgb,
            &ColorProfile::new_srgb(),
            Layout::Rgb,
            TransformOptions::default(),
        )?;
        let mut converted = vec![0; rgb.len()];
        transform.transform(&rgb, &mut converted)?;
        rgb = converted;
    }

    Ok(rgb.chunks_exact(3).map(srgb_to_oklab).collect())
}

fn srgb_to_oklab(rgb: &[u8]) -> [f64; 3] {
    let linear = |channel: u8| {
        let value = f64::from(channel) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    let red = linear(rgb[0]);
    let green = linear(rgb[1]);
    let blue = linear(rgb[2]);

    // Oklab's published linear-sRGB transform: https://bottosson.github.io/posts/oklab/
    let l = (0.4122214708 * red + 0.5363325363 * green + 0.0514459929 * blue).cbrt();
    let m = (0.2119034982 * red + 0.6806995451 * green + 0.1073969566 * blue).cbrt();
    let s = (0.0883024619 * red + 0.2817188376 * green + 0.6299787005 * blue).cbrt();

    [
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
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

        let output = with_palette_bar(PaletteBar {
            image: &image,
            palette: &[red, transparent, blue],
            height: 3,
            profile: None,
        })
        .unwrap();

        assert_eq!(output.dimensions(), (5, 5));

        for y in 0..2 {
            for x in 0..5 {
                assert_eq!(output.get_pixel(x, y), original_color);
            }
        }

        for y in 2..5 {
            for (x, color) in [blue, blue, blue, red, red].into_iter().enumerate() {
                assert_eq!(output.get_pixel(x as u32, y), color);
            }
        }
    }

    #[test]
    fn every_color_gets_a_pixel_when_the_image_is_narrow() {
        let image = DynamicImage::new_rgba8(2, 1);
        let palette = [Rgba([10, 20, 30, 255]), Rgba([40, 50, 60, 255])];

        let output = with_palette_bar(PaletteBar {
            image: &image,
            palette: &palette,
            height: 1,
            profile: None,
        })
        .unwrap();

        assert_eq!(output.get_pixel(0, 1), palette[0]);
        assert_eq!(output.get_pixel(1, 1), palette[1]);
        assert!(with_palette_bar(PaletteBar {
            image: &DynamicImage::new_rgba8(1, 1),
            palette: &palette,
            height: 1,
            profile: None,
        })
        .is_err());
    }

    #[test]
    fn swatch_order_is_independent_of_palette_order() {
        let image = DynamicImage::new_rgba8(6, 1);
        let dark_gold = Rgba([100, 80, 10, 255]);
        let light_gold = Rgba([240, 220, 120, 255]);
        let dark_blue = Rgba([10, 40, 80, 255]);
        let light_blue = Rgba([160, 200, 240, 255]);
        let black = Rgba([0, 0, 0, 255]);
        let white = Rgba([255, 255, 255, 255]);
        let palette = [light_blue, white, light_gold, dark_blue, black, dark_gold];
        let expected = [black, dark_blue, dark_gold, light_blue, light_gold, white];

        let output = with_palette_bar(PaletteBar {
            image: &image,
            palette: &palette,
            height: 1,
            profile: None,
        })
        .unwrap();
        let mut reversed = palette;
        reversed.reverse();
        let reversed_output = with_palette_bar(PaletteBar {
            image: &image,
            palette: &reversed,
            height: 1,
            profile: None,
        })
        .unwrap();

        for (x, color) in expected.into_iter().enumerate() {
            assert_eq!(output.get_pixel(x as u32, 1), color);
            assert_eq!(reversed_output.get_pixel(x as u32, 1), color);
        }
    }

    #[test]
    fn sunset_order_matches_the_approved_perceptual_preview() {
        let palette = [
            [20, 17, 14],
            [37, 30, 26],
            [91, 71, 61],
            [172, 180, 174],
            [65, 47, 40],
            [159, 113, 72],
            [136, 153, 158],
            [64, 58, 58],
            [211, 203, 178],
            [123, 88, 65],
            [168, 168, 136],
            [192, 151, 93],
            [205, 189, 136],
            [247, 219, 153],
            [239, 182, 98],
            [254, 251, 237],
        ]
        .map(|[red, green, blue]| Rgba([red, green, blue, 255]));
        let expected =
            [0, 1, 4, 7, 2, 9, 5, 11, 10, 6, 3, 8, 12, 14, 13, 15].map(|index| palette[index]);
        let image = DynamicImage::new_rgba8(16, 1);

        for offset in 0..palette.len() {
            let mut shuffled = palette;
            shuffled.rotate_left(offset);
            let output = with_palette_bar(PaletteBar {
                image: &image,
                palette: &shuffled,
                height: 1,
                profile: None,
            })
            .unwrap();

            for (x, color) in expected.iter().enumerate() {
                assert_eq!(output.get_pixel(x as u32, 1), *color);
            }
        }
    }

    #[test]
    fn tiny_palettes_and_identical_colors_keep_every_entry() {
        let red = Rgba([200, 40, 30, 255]);
        let faint_red = Rgba([200, 40, 30, 128]);

        for palette in [vec![red], vec![red, red], vec![red, faint_red, red]] {
            let image = DynamicImage::new_rgba8(palette.len() as u32, 1);
            let output = with_palette_bar(PaletteBar {
                image: &image,
                palette: &palette,
                height: 1,
                profile: None,
            })
            .unwrap();
            let mut actual: Vec<_> = (0..image.width())
                .map(|x| output.get_pixel(x, 1).0)
                .collect();
            let mut expected: Vec<_> = palette.iter().map(|color| color.0).collect();
            actual.sort();
            expected.sort();

            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn all_256_gray_shades_remain_in_order() {
        let palette: Vec<_> = (0..=255)
            .rev()
            .map(|value| Rgba([value, value, value, 255]))
            .collect();
        let image = DynamicImage::new_rgba8(256, 1);
        let output = with_palette_bar(PaletteBar {
            image: &image,
            palette: &palette,
            height: 1,
            profile: None,
        })
        .unwrap();

        for value in 0..=255u8 {
            assert_eq!(
                output.get_pixel(u32::from(value), 1),
                Rgba([value, value, value, 255])
            );
        }
    }

    #[test]
    fn oklab_conversion_matches_reference_colors() {
        for (rgb, expected) in [
            ([0, 0, 0], [0.0, 0.0, 0.0]),
            ([255, 255, 255], [1.0, 0.0, 0.0]),
            ([255, 0, 0], [0.6279553606, 0.2248630611, 0.1258462985]),
        ] {
            for (actual, expected) in srgb_to_oklab(&rgb).into_iter().zip(expected) {
                assert!((actual - expected).abs() < 1e-7);
            }
        }
    }

    #[test]
    fn profile_changes_the_comparison_colors_without_changing_the_swatches() {
        let profile = include_bytes!("../../tests/fixtures/display-p3.icc");
        let palette = [Rgba([114, 155, 202, 255]), Rgba([200, 181, 86, 255])];
        let labs = palette_oklab(&palette, Some(profile)).unwrap();

        // Reference: LittleCMS converts this P3 swatch to sRGB [102, 156, 206].
        let expected = [0.675175052, -0.035936754, -0.086906162];
        for (actual, expected) in labs[0].into_iter().zip(expected) {
            assert!((actual - expected).abs() < 0.0015);
        }
        assert_ne!(labs, palette_oklab(&palette, None).unwrap());

        let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(2, 1, palette[0]));
        let output = with_palette_bar(PaletteBar {
            image: &image,
            palette: &palette,
            height: 1,
            profile: Some(profile),
        })
        .unwrap();
        assert_eq!(output.crop_imm(0, 0, 2, 1).to_rgba8(), image.to_rgba8());

        let mut swatches = [output.get_pixel(0, 1).0, output.get_pixel(1, 1).0];
        let mut expected = palette.map(|color| color.0);
        swatches.sort();
        expected.sort();
        assert_eq!(swatches, expected);
        assert!(with_palette_bar(PaletteBar {
            image: &image,
            palette: &palette,
            height: 1,
            profile: Some(b"invalid"),
        })
        .is_err());
    }

    #[test]
    fn invalid_heights_and_empty_palettes_return_errors() {
        let image = DynamicImage::new_rgba8(2, 1);
        let palette = [Rgba([10, 20, 30, 255])];

        assert!(with_palette_bar(PaletteBar {
            image: &image,
            palette: &palette,
            height: 0,
            profile: None,
        })
        .is_err());
        assert!(with_palette_bar(PaletteBar {
            image: &image,
            palette: &palette,
            height: u32::MAX,
            profile: None,
        })
        .is_err());
        assert!(with_palette_bar(PaletteBar {
            image: &image,
            palette: &[],
            height: 1,
            profile: None,
        })
        .is_err());
        assert!(with_palette_bar(PaletteBar {
            image: &image,
            palette: &[Rgba([0, 0, 0, 0])],
            height: 1,
            profile: None,
        })
        .is_err());
    }
}
