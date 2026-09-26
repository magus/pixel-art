use std::error::Error;

use pixel_art::image as pixel_art_image;
use pixel_art::time::Stopwatch;
use pixel_art::Options;

fn options() -> Result<Option<Options>, Box<dyn Error>> {
    let mut options = Options::default();

    let mut args = std::env::args().skip(1);

    while let Some(flag) = args.next() {
        if flag == "--help" || flag == "-h" {
            println!("pixel-art [--input PATH] [--output PATH] [--size PIXELS] [--colors COUNT] [--partitions COUNT] [--sampling MODE] [--debug] [--show-palette] [--palette-height PIXELS]");
            println!(
                "--size sets the longest output edge; aspect ratio is preserved (default 32)."
            );
            println!(
                "--colors caps the RGB palette at 1..=256 colors, plus transparency (default 16)."
            );
            println!("--partitions sets RGB buckets per channel, 1..=32 (default 3).");
            println!("--sampling chooses mode, mean, center, or ink for each output pixel (default mode).");
            println!("  mode: most common palette match; mean: average RGB, then match.");
            println!("  center: center pixel; ink: 15th percentile from dark to light.");
            println!("--debug prints source matching and output pixel details (default off).");
            println!("Use --partitions 16 for a finer palette when comparing color counts.");
            println!(
                "--show-palette appends a bar of equal-width palette swatches below the image."
            );
            println!("--palette-height sets the bar height in output pixels and enables the bar (default 8).");
            return Ok(None);
        }

        match flag.as_str() {
            "--show-palette" => options.show_palette = true,
            "--debug" => options.debug = true,
            "--input" | "--output" | "--size" | "--colors" | "--partitions"
            | "--palette-height" | "--sampling" => {
                let value = args
                    .next()
                    .ok_or_else(|| format!("Missing value for {flag}"))?;

                match flag.as_str() {
                    "--input" => options.input = value,
                    "--output" => options.output = value,
                    "--size" => options.size = value.parse()?,
                    "--colors" => options.colors = value.parse()?,
                    "--partitions" => options.partitions = value.parse()?,
                    "--sampling" => options.sampling = value.parse()?,
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

    let mut pixelated = pixel_art_image::pixelate(&img, &palette, &options)?;
    stopwatch.record("pixelate");

    println!("   [sampling = {:?}]", options.sampling);

    // try zealous cropping at this point once we are finished?
    // pixel_art_image::print(&pixelated);
    if squared_output {
        pixelated = pixel_art_image::zealous_crop(&pixelated, true);
    }

    if options.show_palette {
        let profile = pixel_art_image::source_profile(&options.input)?;
        pixelated = pixel_art_image::with_palette_bar(pixel_art_image::PaletteBar {
            image: &pixelated,
            palette: &palette,
            height: options.palette_height,
            profile: profile.as_deref(),
        })?;
    }

    pixel_art_image::output(&pixelated, &options.input, &options.output)?;
    stopwatch.record("output_pixelated");

    stopwatch.all();

    return Ok(());
}
