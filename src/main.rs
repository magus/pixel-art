use std::error::Error;

use image::DynamicImage;
use image::GenericImage;
use image::GenericImageView;

use rayon::prelude::*;

use pixel_art::image as pixel_art_image;
use pixel_art::time::Stopwatch;

struct Options {
    input: String,
    output: String,
    size: u32,
    colors: usize,
    partitions: usize,
    show_palette: bool,
    palette_height: u32,
}

fn options() -> Result<Option<Options>, Box<dyn Error>> {
    let mut options = Options {
        input: "./images/panda-bear.JPG".into(),
        output: "./output/pixelated.png".into(),
        size: 32,
        colors: 16,
        partitions: 3,
        show_palette: false,
        palette_height: 8,
    };

    let mut args = std::env::args().skip(1);

    while let Some(flag) = args.next() {
        if flag == "--help" || flag == "-h" {
            println!("pixel-art [--input PATH] [--output PATH] [--size PIXELS] [--colors COUNT] [--partitions COUNT] [--show-palette] [--palette-height PIXELS]");
            println!(
                "--size sets the longest output edge; aspect ratio is preserved (default 32)."
            );
            println!(
                "--colors caps the RGB palette at 1..=256 colors, plus transparency (default 16)."
            );
            println!("--partitions sets RGB buckets per channel, 1..=32 (default 3).");
            println!("Use --partitions 16 for a finer palette when comparing color counts.");
            println!(
                "--show-palette appends a bar of equal-width palette swatches below the image."
            );
            println!("--palette-height sets the bar height in output pixels and enables the bar (default 8).");
            return Ok(None);
        }

        match flag.as_str() {
            "--show-palette" => options.show_palette = true,
            "--input" | "--output" | "--size" | "--colors" | "--partitions"
            | "--palette-height" => {
                let value = args
                    .next()
                    .ok_or_else(|| format!("Missing value for {flag}"))?;

                match flag.as_str() {
                    "--input" => options.input = value,
                    "--output" => options.output = value,
                    "--size" => options.size = value.parse()?,
                    "--colors" => options.colors = value.parse()?,
                    "--partitions" => options.partitions = value.parse()?,
                    _ => {
                        options.palette_height = value.parse()?;
                        options.show_palette = true;
                    }
                }
            }
            _ => return Err(format!("Unknown option: {flag}").into()),
        }
    }

    if options.size == 0 {
        return Err("--size must be greater than zero".into());
    }

    if !(1..=256).contains(&options.colors) {
        return Err("--colors must be between 1 and 256".into());
    }

    if !(1..=32).contains(&options.partitions) {
        return Err("--partitions must be between 1 and 32".into());
    }

    if options.palette_height == 0 {
        return Err("--palette-height must be greater than zero".into());
    }

    Ok(Some(options))
}

fn main() -> Result<(), Box<dyn Error>> {
    let Some(options) = options()? else {
        return Ok(());
    };
    let debug = false;

    let mut stopwatch = Stopwatch::start();
    // Use the open function to load an image from a Path.
    // `open` returns a `DynamicImage` on success.
    // let img = image::open("./images/pikachu.png").unwrap();
    // let img = image::open("./images/charizard.png").unwrap();
    // let img = image::open("./images/venusaur.png").unwrap();
    // let img = image::open("./images/758.png").unwrap();
    // let img = image::open("./images/magus.jpg").unwrap();
    // let img = image::open("./images/IMG_0383.PNG").unwrap();
    // let img = image::open("./images/portrait-landscape.JPG").unwrap();
    // let img = image::open("./images/landscape.webp").unwrap();
    // let img = image::open("./images/horse.JPG").unwrap();
    let img = image::open(&options.input)?;

    stopwatch.record("reading_image");

    // do zealous square crop for images with lots of uneven transparency on edges
    // for example, pokemon pixel art often has this
    // otherwise, just do the pixelation on original
    let squared = false;
    let squared_output = false;

    let img = pixel_art_image::zealous_crop(&img, squared);
    // pixel_art_image::output(&img, &options.input, "./output/cropped.png")?;
    stopwatch.record("zealous_crop");

    // draw image to cli
    // pixel_art_image::print(&img);

    let palette = pixel_art_image::palette(&img, options.colors, options.partitions);
    stopwatch.record("palette");

    println!("\n🤖 pixelate\n");

    let palette_size = palette.len();

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

                            let closest_index =
                                pixel_art_image::closest_rgb(&palette, &pixel, debug);
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
                pixelated.put_pixel(x, y, pixel);
            }
        }
    }

    stopwatch.record("put_pixel_pixelated");

    // try zealous cropping at this point once we are finished?
    // pixel_art_image::print(&pixelated);
    if squared_output {
        pixelated = pixel_art_image::zealous_crop(&pixelated, true);
    }

    if options.show_palette {
        pixelated =
            pixel_art_image::with_palette_bar(&pixelated, &palette, options.palette_height)?;
    }

    pixel_art_image::output(&pixelated, &options.input, &options.output)?;
    stopwatch.record("output_pixelated");

    stopwatch.all();

    return Ok(());
}
