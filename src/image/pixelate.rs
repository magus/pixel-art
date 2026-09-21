use std::error::Error;
use std::str::FromStr;

use image::{DynamicImage, GenericImage, GenericImageView, Rgba, RgbaImage};
use rayon::prelude::*;

use crate::time::Stopwatch;
use crate::Options;

use super::closest_rgb;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sampling {
    Mode,
    Mean,
    Center,
    Ink,
}

impl FromStr for Sampling {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "mode" => Ok(Self::Mode),
            "mean" => Ok(Self::Mean),
            "center" => Ok(Self::Center),
            "ink" => Ok(Self::Ink),
            _ => Err(format!(
                "Unknown sampling mode: {value}. Use mode, mean, center, or ink."
            )),
        }
    }
}

pub fn pixelate(
    img: &DynamicImage,
    palette: &[Rgba<u8>],
    options: &Options,
) -> Result<DynamicImage, Box<dyn Error>> {
    let mut stopwatch = Stopwatch::start();

    println!("\n🤖 pixelate\n");

    let output_size = options.size;
    let (width, height) = img.dimensions();
    println!("   [image = {}×{}]", width, height);

    let ratio = width as f32 / height as f32;
    let output_width;
    let output_height;
    if height > width {
        output_width = (output_size as f32 * ratio) as u32;
        output_height = output_size;
    } else {
        output_width = output_size;
        output_height = (output_size as f32 / ratio) as u32;
    }
    println!("   [output = {}×{}]", output_width, output_height);
    println!("   [ratio = {}]", ratio);

    if output_width == 0 || output_height == 0 || output_width > width || output_height > height {
        return Err("--size must produce nonzero dimensions no larger than the input".into());
    }

    let grid_scalar_width = width as f32 / output_width as f32;
    let grid_scalar_height = height as f32 / output_height as f32;
    let grid_width = width / output_width;
    let grid_height = height / output_height;

    println!(
        "   [grid_scalar = {}x{}]",
        grid_scalar_width, grid_scalar_height
    );
    println!("   [grid_size = {}×{}]", grid_width, grid_height);

    if palette.is_empty() {
        return Err("The palette must contain at least one color".into());
    }

    if options.sampling == Sampling::Mode {
        let palette_size = palette.len();
        let debug = options.debug;

        // initialize vector for each palette color for each grid cell
        // e.g. [0, 0, 0] maps to [color_1, color_2, color_3]
        // we will increment the value at idnex when a pixel is closest to a particular color
        // end result will allow us to determine most representative palette color for a grid cell
        let mut color_counts = vec![];
        for _ in 0..output_width {
            let mut column = vec![];

            for _ in 0..output_height {
                let colors = vec![0; palette_size];
                column.push(colors);
            }

            color_counts.push(column);
        }
        stopwatch.record("initialize_color_counts");

        // println!("[color_counts={:?}]", color_counts);

        let mut pixelated = DynamicImage::new_rgba8(output_width, output_height);
        stopwatch.record("create_pixelated_buffer");

        color_counts
            .par_iter_mut()
            .enumerate()
            .for_each(|(grid_x, grid_column)| {
                grid_column
                    .par_iter_mut()
                    .enumerate()
                    .for_each(|(grid_y, grid_cell)| {
                        let x_start = (grid_x as f32 * grid_scalar_width).floor() as u32;
                        let y_start = (grid_y as f32 * grid_scalar_height).floor() as u32;
                        let x_end = x_start + grid_width;
                        let y_end = y_start + grid_height;

                        for x in x_start..x_end {
                            for y in y_start..y_end {
                                let pixel = img.get_pixel(x, y);
                                // println!("({},{}) = {:?}", x, y, pixel);

                                let closest_index = closest_rgb(&palette, &pixel, debug);
                                grid_cell[closest_index] += 1;

                                if debug {
                                    println!(
                                        "[{},{}] ({},{}) = {:?} [closest_index = {}]",
                                        grid_x, grid_y, x, y, pixel, closest_index
                                    )
                                }
                            }
                        }
                    });
            });

        stopwatch.record("calcuate_pixelated_grid_cells");

        // println!("[color_counts={:?}]", color_counts);

        // let mut pixelated = DynamicImage::new_rgba8(output_size, output_size);
        // let mut pixelated = RgbaImage::new(output_size, output_size);

        for y in 0..pixelated.height() {
            for x in 0..pixelated.width() {
                let grid_cell = color_counts
                    .get(x as usize)
                    .unwrap()
                    .get(y as usize)
                    .unwrap();

                // walk each palette color count in grid_cell vector
                // discover the highest count and color this pixel that color
                let mut found_max = false;
                let mut max_index = 0;
                let mut max_count = &0;
                for index in 0..grid_cell.len() {
                    let count = grid_cell.get(index).unwrap();

                    if count > max_count {
                        found_max = true;
                        max_index = index;
                        max_count = count;
                    }
                }

                // debugging horse photo at 32x32 with zealous crop
                // had uneven borders due to square grid cell not aligning with landscape image
                // if y == 27 {
                //     println!("({},{}) = {:?}", x, y, grid_cell);
                // }

                // color pixel the palette color of max_index
                if found_max {
                    let pixel = *palette.get(max_index).unwrap();
                    // println!("({},{}) = {:?}", x, y, pixel);
                    if debug {
                        println!(
                            "[pixelate] ({x},{y}) sampling={:?} color={pixel:?}",
                            options.sampling,
                        );
                    }

                    pixelated.put_pixel(x, y, pixel);
                }
            }
        }

        stopwatch.record("put_pixel_pixelated");

        stopwatch.all();

        return Ok(pixelated);
    }

    let mut brightness_order = Vec::new();
    if options.sampling == Sampling::Ink {
        brightness_order = (0..palette.len())
            .filter(|&index| palette[index][3] != 0)
            .collect();
        brightness_order.sort_by_key(|&index| {
            let color = palette[index];
            299 * color[0] as u32 + 587 * color[1] as u32 + 114 * color[2] as u32
        });
    }

    let mut output = RgbaImage::new(output_width, output_height);
    stopwatch.record("create_pixelated_buffer");

    output
        .par_chunks_mut(output_width as usize * 4)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, pixel) in row.chunks_exact_mut(4).enumerate() {
                let start_x = (x as f32 * grid_scalar_width).floor() as u32;
                let start_y = (y as f32 * grid_scalar_height).floor() as u32;
                let block = img.view(start_x, start_y, grid_width, grid_height);

                let color = match options.sampling {
                    Sampling::Mode => unreachable!("mode is handled above"),
                    Sampling::Mean => mean(&*block, palette, options),
                    Sampling::Center => {
                        let center = block.get_pixel(grid_width / 2, grid_height / 2);
                        palette[closest_rgb(palette, &center, options.debug)]
                    }
                    Sampling::Ink => ink(&*block, palette, &brightness_order, options),
                };

                if options.debug {
                    println!(
                        "[pixelate] ({x},{y}) source=({start_x},{start_y}) block={grid_width}×{grid_height} sampling={:?} color={:?}",
                        options.sampling, color,
                    );
                }

                pixel.copy_from_slice(&color.0);
            }
        });

    stopwatch.record("sample_and_put_pixels");
    stopwatch.all();

    return Ok(DynamicImage::ImageRgba8(output));
}

fn mean(
    block: &impl GenericImageView<Pixel = Rgba<u8>>,
    palette: &[Rgba<u8>],
    options: &Options,
) -> Rgba<u8> {
    let mut sums = [0u64; 3];
    let mut alpha = 0u64;

    for (_, _, pixel) in block.pixels() {
        alpha += pixel[3] as u64;
        for channel in 0..3 {
            sums[channel] += pixel[channel] as u64 * pixel[3] as u64;
        }
    }

    let area = block.width() as u64 * block.height() as u64;
    if alpha * 2 < area * 255 {
        return Rgba([0, 0, 0, 0]);
    }

    // Keep fractional RGB values until matching; rounding can change the nearest palette color.
    let average = sums.map(|sum| sum as f64 / alpha as f64);
    if options.debug {
        println!("[mean] average RGB = {average:?}");
    }

    let mut closest = Rgba([0, 0, 0, 0]);
    let mut closest_distance = f64::INFINITY;
    for color in palette.iter().filter(|color| color[3] != 0) {
        let mut distance = 0.0;
        for channel in 0..3 {
            distance += (color[channel] as f64 - average[channel]).abs();
        }

        if options.debug {
            println!("[mean] palette color={color:?} distance={distance}");
        }

        if distance < closest_distance {
            closest = *color;
            closest_distance = distance;
        }
    }

    closest
}

fn ink(
    block: &impl GenericImageView<Pixel = Rgba<u8>>,
    palette: &[Rgba<u8>],
    brightness_order: &[usize],
    options: &Options,
) -> Rgba<u8> {
    let mut counts = vec![0u64; palette.len()];
    let mut alpha = 0u64;
    let mut visible = 0u64;

    for (_, _, pixel) in block.pixels() {
        alpha += pixel[3] as u64;
        if pixel[3] != 0 {
            counts[closest_rgb(palette, &pixel, options.debug)] += 1;
            visible += 1;
        }
    }

    let area = block.width() as u64 * block.height() as u64;
    if alpha * 2 < area * 255 {
        return Rgba([0, 0, 0, 0]);
    }

    let mut cumulative = 0u64;
    for &index in brightness_order {
        cumulative += counts[index];
        if cumulative * 100 >= visible * 15 {
            return palette[index];
        }
    }

    Rgba([0, 0, 0, 0])
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLACK: Rgba<u8> = Rgba([0, 0, 0, 255]);
    const GRAY: Rgba<u8> = Rgba([128, 128, 128, 255]);
    const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);
    const CLEAR: Rgba<u8> = Rgba([0, 0, 0, 0]);
    const PALETTE: [Rgba<u8>; 4] = [WHITE, BLACK, GRAY, CLEAR];

    fn render(pixels: &[Rgba<u8>], sampling: Sampling) -> Rgba<u8> {
        let side = (pixels.len() as f64).sqrt() as u32;
        let source = RgbaImage::from_fn(side, side, |x, y| pixels[(y * side + x) as usize]);
        let options = Options {
            size: 1,
            sampling,
            ..Options::default()
        };

        pixelate(&source.into(), &PALETTE, &options)
            .unwrap()
            .get_pixel(0, 0)
    }

    #[test]
    fn thin_strokes_distinguish_the_sampling_rules() {
        let pixels = [BLACK, WHITE, WHITE, WHITE];

        assert_eq!(render(&pixels, Sampling::Mode), WHITE);
        assert_eq!(render(&pixels, Sampling::Mean), GRAY);
        assert_eq!(render(&pixels, Sampling::Center), WHITE);
        assert_eq!(render(&pixels, Sampling::Ink), BLACK);
    }

    #[test]
    fn mode_keeps_the_first_palette_entry_on_a_tie() {
        assert_eq!(render(&[BLACK, BLACK, WHITE, WHITE], Sampling::Mode), WHITE);
    }

    #[test]
    fn center_uses_the_lower_right_middle_pixel_in_an_even_block() {
        assert_eq!(
            render(&[WHITE, WHITE, WHITE, BLACK], Sampling::Center),
            BLACK
        );
    }

    #[test]
    fn aggregate_rules_use_half_alpha_coverage_and_ignore_hidden_colors() {
        let hidden_red = Rgba([255, 0, 0, 0]);
        for sampling in [Sampling::Mean, Sampling::Ink] {
            assert_eq!(
                render(&[WHITE, WHITE, hidden_red, hidden_red], sampling),
                WHITE
            );
            assert_eq!(
                render(&[WHITE, hidden_red, hidden_red, hidden_red], sampling),
                CLEAR
            );
            assert_eq!(render(&[Rgba([255, 255, 255, 127]); 4], sampling), CLEAR);
            assert_eq!(render(&[Rgba([255, 255, 255, 128]); 4], sampling), WHITE);
        }
    }

    #[test]
    fn ink_requires_fifteen_percent_dark_pixels() {
        let mut pixels = [WHITE; 100];
        pixels[..14].fill(BLACK);
        assert_eq!(render(&pixels, Sampling::Ink), WHITE);

        pixels[14] = BLACK;
        assert_eq!(render(&pixels, Sampling::Ink), BLACK);
    }

    #[test]
    fn mean_weights_partial_alpha_and_keeps_fractional_rgb() {
        let source = RgbaImage::from_fn(2, 2, |x, _| {
            if x == 0 {
                Rgba([255, 255, 255, 128])
            } else {
                BLACK
            }
        });
        let options = Options {
            size: 1,
            sampling: Sampling::Mean,
            ..Options::default()
        };
        let output = pixelate(&source.into(), &[WHITE, BLACK, CLEAR], &options).unwrap();
        assert_eq!(output.get_pixel(0, 0), BLACK);

        let almost_black = Rgba([1, 1, 1, 255]);
        let source = RgbaImage::from_fn(2, 2, |x, y| {
            if x == 0 && y == 0 {
                BLACK
            } else {
                almost_black
            }
        });
        let output = pixelate(&source.into(), &[BLACK, almost_black], &options).unwrap();
        assert_eq!(output.get_pixel(0, 0), almost_black);
    }

    #[test]
    fn every_rule_preserves_a_fully_transparent_block() {
        for sampling in [
            Sampling::Mode,
            Sampling::Mean,
            Sampling::Center,
            Sampling::Ink,
        ] {
            assert_eq!(render(&[CLEAR; 4], sampling), CLEAR);
        }
    }

    #[test]
    fn rectangular_output_keeps_the_existing_block_boundaries() {
        let source = RgbaImage::from_fn(7, 5, |x, y| if x == 6 || y == 4 { BLACK } else { WHITE });

        for sampling in [
            Sampling::Mode,
            Sampling::Mean,
            Sampling::Center,
            Sampling::Ink,
        ] {
            let options = Options {
                size: 3,
                sampling,
                ..Options::default()
            };
            let output = pixelate(&source.clone().into(), &PALETTE, &options).unwrap();
            assert_eq!(output.dimensions(), (3, 2));
            assert!(output.pixels().all(|(_, _, pixel)| pixel == WHITE));
        }
    }

    #[test]
    fn rejects_invalid_sizes_and_empty_palettes() {
        let source = DynamicImage::new_rgba8(4, 2);
        for size in [0, 1, 5] {
            let options = Options {
                size,
                ..Options::default()
            };
            assert!(pixelate(&source, &PALETTE, &options).is_err());
        }
        let options = Options {
            size: 2,
            ..Options::default()
        };
        assert!(pixelate(&source, &[], &options).is_err());
    }

    #[test]
    fn parses_only_the_documented_sampling_names() {
        for (name, sampling) in [
            ("mode", Sampling::Mode),
            ("mean", Sampling::Mean),
            ("center", Sampling::Center),
            ("ink", Sampling::Ink),
        ] {
            assert_eq!(name.parse::<Sampling>(), Ok(sampling));
        }
        assert!("unknown"
            .parse::<Sampling>()
            .unwrap_err()
            .contains("mode, mean, center, or ink"));
    }
}
