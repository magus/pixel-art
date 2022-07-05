use core::fmt::Debug;
use std::cmp;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

use image::DynamicImage;
use image::GenericImageView;
use image::Pixel;
use image::RgbaImage;

fn main() -> Result<(), Box<dyn Error>> {
    // Use the open function to load an image from a Path.
    // `open` returns a `DynamicImage` on success.
    // let img = image::open("./images/pikachu.png").unwrap();
    // let img = image::open("./images/charizard.png").unwrap();
    let img = image::open("./images/venusaur.png").unwrap();

    output_cropped(&img, "./output/cropped.png")?;

    return Ok(());
}

fn output_cropped(img: &DynamicImage, output_path: &str) -> Result<(), Box<dyn Error>> {
    let crop = get_crop(&img).unwrap();

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

    let path = Path::new(output_path);
    let dir = path.parent().unwrap();
    fs::create_dir_all(dir)?;
    cropped_img.save(output_path)?;

    return Ok(());
}

fn print_crop(crop_size: u32, original_size: u32) {
    let reduction_percent = (1.0 - (crop_size as f32 / original_size as f32)) * 100.0;

    println!(
        "   cropped {:>3}px -> {:>3}px ({:.2}% size reduction)",
        original_size, crop_size, reduction_percent
    );
}

fn get_crop(img: &DynamicImage) -> Option<Crop> {
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

    return Some(crop);
}

#[derive(Debug)]
enum Scan {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

fn range(start: u32, end: u32) -> Box<dyn Iterator<Item = u32>> {
    let reverse = start > end;

    if reverse {
        return Box::new((end..start).rev());
    }

    return Box::new(start..end);
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

#[derive(Default, Debug)]
struct Crop {
    top: u32,
    right: u32,
    bottom: u32,
    left: u32,
}

impl Crop {
    fn width(&self) -> u32 {
        self.right - self.left + 1
    }

    fn height(&self) -> u32 {
        self.bottom - self.top + 1
    }
}

impl fmt::Display for Crop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "[{}x{}] {:?} ", self.width(), self.height(), self)
    }
}

#[derive(Default, Debug)]
struct Point {
    x: u32,
    y: u32,
}
