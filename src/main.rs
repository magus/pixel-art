use std::cmp;
use std::error::Error;
use std::fs;

use image::DynamicImage;
use image::GenericImageView;
use image::Pixel;
use image::RgbaImage;

fn main() -> Result<(), Box<dyn Error>> {
    // Use the open function to load an image from a Path.
    // `open` returns a `DynamicImage` on success.
    let img = image::open("./images/pikachu.png").unwrap();
    // let img = image::open("./images/charizard.png").unwrap();
    // let img = image::open("./images/venusaur.png").unwrap();

    // The dimensions method returns the images width and height.
    let dimensions = img.dimensions();
    println!("dimensions {:?}", dimensions);

    // The color method returns the image's `ColorType`.
    println!("{:?}", img.color());

    // Or use the `get_pixel` method from the `GenericImage` trait.
    let pixel = img.get_pixel(32, 32);
    println!("{:?}", pixel);

    if let [red, green, blue, alpha] = pixel.channels() {
        println!("R{:?} G{:?} B{:?} A{:?}", red, green, blue, alpha);
    }

    let crop = get_crop(&img).unwrap();
    println!("{:?}", crop);

    let mut x_start: u32 = 0;
    let mut y_start: u32 = 0;
    let size: u32 = cmp::max(crop.width, crop.height);

    if crop.width > crop.height {
        y_start += (crop.width - crop.height) / 2;
        println!("wider");
    } else {
        x_start += (crop.height - crop.width) / 2;
        println!("taller",);
    }

    println!(
        "size {:>3}x{:<3} ; start @ ({:>3}, {:>3})",
        size, size, x_start, y_start
    );

    println!(
        "{:.2}% size reduction",
        ((size * 100) as f32 / dimensions.0 as f32)
    );

    // copy source pixels to image buffer and save to view cropped image

    let mut cropped_img = RgbaImage::new(size, size);
    // https://docs.rs/image/latest/image/struct.ImageBuffer.html
    for x in 0..crop.width {
        for y in 0..crop.height {
            // println!("pixel({:>3}, {:>3})", crop.left + x, crop.top + y);
            // grab from source
            let pixel = img.get_pixel(crop.left + x, crop.top + y);
            // place into cropped image buffer
            cropped_img.put_pixel(x_start + x, y_start + y, pixel);
        }
    }

    fs::create_dir_all("output/")?;
    cropped_img.save("output/cropped.png")?;

    return Ok(());
}

fn get_crop(img: &DynamicImage) -> Option<Crop> {
    let mut crop = Crop {
        ..Default::default()
    };

    let (width, height) = img.dimensions();

    // top edge
    // scan left to right, from top to bottom
    crop.top = scan_edge(&img, 0..height, 0..width, true)?.y;

    // right edge
    // scan top to bottom, from right to left
    crop.right = scan_edge(&img, (0..width).rev().into_iter(), 0..height, false)?.x;

    // bottom edge
    // scan left to right, from bottom to top
    crop.bottom = scan_edge(&img, (0..height).rev().into_iter(), 0..width, true)?.y;

    // left edge
    // scan top to bottom, from left to right
    crop.left = scan_edge(&img, 0..width, 0..height, false)?.x;

    crop.width = crop.right - crop.left + 1;
    crop.height = crop.bottom - crop.top + 1;

    return Some(crop);
}

fn scan_edge(
    img: &DynamicImage,
    range_a: impl Iterator<Item = u32> + Clone,
    range_b: impl Iterator<Item = u32> + Clone,
    reversed: bool,
) -> Option<Point> {
    for a in range_a.clone() {
        for b in range_b.clone() {
            // put a and b into the correct x,y variables
            let (x, y) = if reversed { (b, a) } else { (a, b) };

            let pixel = img.get_pixel(x, y);
            if let [red, green, blue, alpha] = pixel.channels() {
                if *alpha != 0 {
                    println!(
                        "![EDGE] ({: >3},{: >3}) [R{:>4}, G{:>4}, B{:>4}, A{:>4}]",
                        x, y, red, green, blue, alpha
                    );

                    return Some(Point { x, y });
                }
            }
        }
    }

    return if reversed {
        Some(Point {
            x: range_b.last()?,
            y: range_a.last()?,
        })
    } else {
        Some(Point {
            x: range_a.last()?,
            y: range_b.last()?,
        })
    };
}

#[derive(Default, Debug)]
struct Crop {
    top: u32,
    right: u32,
    bottom: u32,
    left: u32,

    width: u32,
    height: u32,
}

#[derive(Default, Debug)]
struct Point {
    x: u32,
    y: u32,
}
