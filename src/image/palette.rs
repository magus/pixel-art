use std::cmp;

use colored::*;
use image::GenericImageView;
use image::{DynamicImage, Pixel, Rgba};

static PARTITIONS: u8 = 4;
static OUTPUT_COLOR_COUNT: usize = 32;

pub fn palette(img: &DynamicImage) -> Vec<Rgba<u8>> {
    println!("\n🤖 calculating color_space ...");

    let mut color_space: Vec<Vec<Rgba<u8>>> = Vec::new();

    // PARTITIONS^3 for each value in rgb
    // equivalent to 3 nested for loops
    // e.g.
    //   for r in 0..PARTITIONS {
    //     for g in 0..PARTITIONS {
    //         for b in 0..PARTITIONS {
    for _ in 0..PARTITIONS.pow(3) {
        color_space.push(Vec::new());
    }

    println!("\nwalking pixels of image ...\n");

    let (width, height) = img.dimensions();

    let mut total_pixel_count = 0;
    let mut used_pixel_count = 0;

    for x in 0..width {
        for y in 0..height {
            total_pixel_count += 1;

            let pixel = img.get_pixel(x, y);

            if let Some(color_space_index) = get_pixel_index(&pixel) {
                used_pixel_count += 1;

                // periodic debug print
                // if x == 0 && y == 0 || total_pixel_count % 574 == 0 {
                //     println!(
                //         "   pixel({:>3?},{:>3?}) = {:?} = {:?}",
                //         x, y, pixel, color_space_index
                //     );
                // }

                // println!(
                //     "   pixel({:>3?},{:>3?}) = {:?} = {:?} / {}",
                //     x,
                //     y,
                //     pixel,
                //     color_space_index,
                //     color_space.len()
                // );

                let space = color_space.get_mut(color_space_index).unwrap();
                space.push(pixel.clone());
            }
        }
    }

    println!("\n   color_space[{}]\n", color_space.len());
    println!(
        "   {:.2}% used pixel density ({:?}/{:?})",
        percent(used_pixel_count, total_pixel_count),
        used_pixel_count,
        total_pixel_count,
    );
    println!();

    // sort by number of pixels in each partition
    color_space.sort_by(|a, b| b.len().cmp(&a.len()));

    let output_count = cmp::min(OUTPUT_COLOR_COUNT, color_space.len());
    let mut output = vec![];

    println!("\n🤖 palette\n");

    for i in 0..output_count {
        let space = color_space.get(i).unwrap();

        let space_pixel_count = space.len() as u32;
        let (mut red, mut green, mut blue) = (0_u32, 0_u32, 0_u32);

        for pixel in space {
            if let [r, g, b, _] = pixel.channels() {
                red += *r as u32;
                green += *g as u32;
                blue += *b as u32;
            }
        }

        let (a_red, a_green, a_blue) = (
            average_color(red, space_pixel_count),
            average_color(green, space_pixel_count),
            average_color(blue, space_pixel_count),
        );

        let average_pixel = Rgba([a_red, a_green, a_blue, 255]);

        println!(
            "  space[{:>3}] [{:>8} pixels] {} {:?}",
            i,
            space.len(),
            "     ".on_truecolor(a_red, a_green, a_blue),
            average_pixel,
        );

        output.push(average_pixel.clone());
    }

    output.push(Rgba::from([0, 0, 0, 0]));

    return output;
}

fn partition_len() -> u8 {
    (u8::MAX as f32 / PARTITIONS as f32).ceil() as u8
}

fn get_index(r: u8, g: u8, b: u8) -> u8 {
    // println!("{},{},{}", r, g, b);
    return (r * PARTITIONS.pow(0)) + (g * PARTITIONS.pow(1)) + (b * PARTITIONS.pow(2));
}

fn get_pixel_index(pixel: &Rgba<u8>) -> Option<usize> {
    if let [r, g, b, alpha] = pixel.channels() {
        if *alpha == 0 {
            return None;
        }

        let index = get_index(
            // force line break
            get_partition(r),
            get_partition(g),
            get_partition(b),
        );

        return Some(index as usize);
    };

    None
}

fn get_partition(color: &u8) -> u8 {
    // ensure color isn't max
    let mut color = *color;
    if color == u8::MAX {
        color = color - 1
    }

    let result = color / partition_len();
    // println!(
    //     "color={},PARTITION_LEN={},result={}, u8::MAX={}",
    //     color,
    //     partition_len(),
    //     result,
    //     u8::MAX
    // );
    return result;
}

fn average_color(total: u32, count: u32) -> u8 {
    if count == 0 {
        return 0;
    }

    (total / count) as u8
}

fn percent(num: i32, den: i32) -> f32 {
    100.0 * num as f32 / den as f32
}
