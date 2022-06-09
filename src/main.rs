use image::DynamicImage;
use image::GenericImageView;
use image::Pixel;

fn main() {
    // Use the open function to load an image from a Path.
    // `open` returns a `DynamicImage` on success.
    // let img = image::open("./images/pikachu.png").unwrap();
    // let img = image::open("./images/charizard.png").unwrap();
    let img = image::open("./images/venusaur.png").unwrap();

    // The dimensions method returns the images width and height.
    println!("dimensions {:?}", img.dimensions());

    // The color method returns the image's `ColorType`.
    println!("{:?}", img.color());

    // Or use the `get_pixel` method from the `GenericImage` trait.
    let pixel = img.get_pixel(32, 32);
    println!("{:?}", pixel);

    if let [red, green, blue, alpha] = pixel.channels() {
        println!("R{:?} G{:?} B{:?} A{:?}", red, green, blue, alpha);
    }

    let crop = get_crop(&img);

    println!("{:?}", crop);
}

fn get_crop(img: &DynamicImage) -> Option<Crop> {
    let mut crop = Crop {
        ..Default::default()
    };

    let (width, height) = img.dimensions();

    // top edge
    // scan left to right, from top to bottom
    crop.top = scan_edge(&img, 0..height, 0..width, true)?.1;

    // right edge
    // scan top to bottom, from right to left
    crop.right = scan_edge(&img, (0..width).rev().into_iter(), 0..height, false)?.0;

    // bottom edge
    // scan left to right, from bottom to top
    crop.bottom = scan_edge(&img, (0..height).rev().into_iter(), 0..width, true)?.1;

    // left edge
    // scan top to bottom, from left to right
    crop.left = scan_edge(&img, 0..width, 0..height, false)?.0;

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

                    return Some(Point(x, y));
                }
            }
        }
    }

    return if reversed {
        Some(Point(range_b.last()?, range_a.last()?))
    } else {
        Some(Point(range_a.last()?, range_b.last()?))
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
struct Point(u32, u32);
