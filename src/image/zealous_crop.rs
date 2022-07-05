use core::fmt::Debug;
use std::cmp;

use image::DynamicImage;
use image::GenericImageView;
use image::Pixel;
use image::RgbaImage;

use crate::image::crop::Crop;
use crate::image::point::Point;
use crate::range::range;

pub fn zealous_crop(img: &DynamicImage) -> RgbaImage {
    let crop = get_zealous_crop(&img);

    println!("\n🤖 output_cropped");

    println!("   {}", crop);

    let mut x_start = 0;
    let mut y_start = 0;
    let size = cmp::max(crop.width(), crop.height());
    let delta = crop.width().abs_diff(crop.height());

    if crop.width() > crop.height() {
        y_start += delta / 2;
        println!("   wider by {}px", delta);
    } else {
        x_start += delta / 2;
        println!("   taller by {}px", delta);
    }

    println!(
        "   output size {:>3}x{:<3} ; crop copy start @ ({:>3}, {:>3})",
        size, size, x_start, y_start
    );

    print_crop(size, img.dimensions().0);

    // copy source pixels to image buffer and save to view cropped image

    let mut cropped_img = RgbaImage::new(size, size);
    // https://docs.rs/image/latest/image/struct.ImageBuffer.html
    for x in 0..crop.width() {
        for y in 0..crop.height() {
            // println!("pixel({:>3}, {:>3})", crop.left + x, crop.top + y);
            // grab from source
            let pixel = img.get_pixel(crop.left + x, crop.top + y);
            // place into cropped image buffer
            cropped_img.put_pixel(x_start + x, y_start + y, pixel);
        }
    }

    return cropped_img;
}

fn print_crop(crop_size: u32, original_size: u32) {
    let reduction_percent = (1.0 - (crop_size as f32 / original_size as f32)) * 100.0;

    println!(
        "   cropped {:>3}px -> {:>3}px ({:.2}% size reduction)",
        original_size, crop_size, reduction_percent
    );
}

pub fn get_zealous_crop(img: &DynamicImage) -> Crop {
    let mut crop = Crop {
        ..Default::default()
    };

    // top edge
    // scan from top to bottom, going left to right
    crop.top = scan_edge(&img, Scan::TopToBottom, is_pixel_not_alpha);

    // right edge
    // scan from right to left, going top to bottom
    crop.right = scan_edge(&img, Scan::RightToLeft, is_pixel_not_alpha);

    // bottom edge
    // scan from bottom to top, going left to right
    crop.bottom = scan_edge(&img, Scan::BottomToTop, is_pixel_not_alpha);

    // left edge
    // scan from left to right, going top to bottom
    crop.left = scan_edge(&img, Scan::LeftToRight, is_pixel_not_alpha);

    return crop;
}

fn scan_edge(img: &DynamicImage, scan: Scan, test: fn(&DynamicImage, Point) -> bool) -> u32 {
    println!("\nscan_edge [scan=[{:?}]", scan);

    let (width, height) = img.dimensions();

    let (range_a_start, range_a_end, range_b_start, range_b_end) = match scan {
        Scan::LeftToRight => (0, width, 0, height),
        Scan::RightToLeft => (width, 0, 0, height),
        Scan::TopToBottom => (0, height, 0, width),
        Scan::BottomToTop => (height, 0, 0, width),
    };

    println!(
        "   matched [range_a={:?}..{:?}] [range_b={:?}..{:?}]",
        range_a_start, range_a_end, range_b_start, range_b_end
    );

    for a in range(range_a_start, range_a_end) {
        for b in range(range_b_start, range_b_end) {
            // put a and b into the correct x,y variables
            let (x, y) = match scan {
                Scan::LeftToRight | Scan::RightToLeft => (a, b),
                Scan::TopToBottom | Scan::BottomToTop => (b, a),
            };

            if test(img, Point { x, y }) {
                println!("   returning [a={:?}] @ ({:>3?}, {:>3?})", a, x, y);
                return a;
            }
        }
    }

    println!("   unable to find passing pixel");

    return range_a_end;
}

fn is_pixel_not_alpha(img: &DynamicImage, point: Point) -> bool {
    let pixel = img.get_pixel(point.x, point.y);

    if let [_, _, _, alpha] = pixel.channels() {
        if *alpha != 0 {
            return true;
        }
    }

    return false;
}

#[derive(Debug)]
enum Scan {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}