use image::DynamicImage;
use image::GenericImageView;
use image::Pixel;

fn main() {
    // Use the open function to load an image from a Path.
    // `open` returns a `DynamicImage` on success.
    let img = image::open("./images/pikachu.png").unwrap();
    // let img = image::open("./images/charizard.png").unwrap();

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

    let crop = get_crop(img);
    println!("{:?}", crop);
}

fn get_crop(img: DynamicImage) -> Crop {
    let mut crop = Crop {
        ..Default::default()
    };

    let (width, height) = img.dimensions();

    let mut scan_done = false;

    // top edge
    // scan left to right, from top to bottom
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            if let [red, green, blue, alpha] = pixel.channels() {
                if *alpha != 0 {
                    println!(
                        "![TOP] ({},{}) [R{:?}G{:?}B{:?}A{:?}]",
                        x, y, red, green, blue, alpha
                    );

                    crop.top = y;

                    scan_done = true;
                    break;
                }
            }
        }

        if scan_done {
            break;
        }
    }

    scan_done = false;

    // right edge
    // scan top to bottom, from right to left
    for x in (0..width).rev() {
        for y in 0..height {
            let pixel = img.get_pixel(x, y);
            if let [red, green, blue, alpha] = pixel.channels() {
                if *alpha != 0 {
                    println!(
                        "![RIGHT] ({},{}) [R{:?}G{:?}B{:?}A{:?}]",
                        x, y, red, green, blue, alpha
                    );

                    crop.right = x;

                    scan_done = true;
                    break;
                }
            }
        }

        if scan_done {
            break;
        }
    }

    scan_done = false;

    // bottom edge
    // scan left to right, from bottom to top
    for y in (0..height).rev() {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            if let [red, green, blue, alpha] = pixel.channels() {
                if *alpha != 0 {
                    println!(
                        "![BOTTOM] ({},{}) [R{:?}G{:?}B{:?}A{:?}]",
                        x, y, red, green, blue, alpha
                    );

                    crop.bottom = y;

                    scan_done = true;
                    break;
                }
            }
        }

        if scan_done {
            break;
        }
    }

    scan_done = false;

    // left edge
    // scan top to bottom, from left to right
    for x in 0..width {
        for y in 0..height {
            let pixel = img.get_pixel(x, y);
            if let [red, green, blue, alpha] = pixel.channels() {
                if *alpha != 0 {
                    println!(
                        "![LEFT] ({},{}) [R{:?}G{:?}B{:?}A{:?}]",
                        x, y, red, green, blue, alpha
                    );

                    crop.left = x;

                    scan_done = true;
                    break;
                }
            }
        }

        if scan_done {
            break;
        }
    }

    crop.width = crop.right - crop.left + 1;
    crop.height = crop.bottom - crop.top + 1;

    return crop;
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
