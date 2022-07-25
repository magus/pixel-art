use image::DynamicImage;
use image::GenericImage;
use image::GenericImageView;
use pixel_art::image as pixel_art_image;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
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
    let img = image::open("./images/horse.JPG").unwrap();
    // let img = image::open("./images/panda-bear.JPG").unwrap();

    // do zealous square crop for images with lots of uneven transparency on edges
    // for example, pokemon pixel art often has this
    // otherwise, just do the pixelation on original
    let squared = false;
    let squared_output = false;

    let img = pixel_art_image::zealous_crop(&img, squared);
    pixel_art_image::output(&img, "./output/cropped.png")?;

    // draw image to cli
    // pixel_art_image::print(&img);

    let palette = pixel_art_image::palette(&img);

    println!("\n🤖 pixelate\n");

    let palette_size = palette.len();

    let output_size = 32;
    let (width, height) = img.dimensions();
    println!("  [output_size={}]", output_size);
    println!("  [image={}×{}]", width, height);

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
    println!(
        "  [output][ratio={}][{}x{}]",
        ratio, output_width, output_height
    );

    let grid_scalar_width = width as f32 / output_width as f32;
    let grid_scalar_height = height as f32 / output_height as f32;
    let grid_width = width / output_width;
    let grid_height = height / output_height;

    println!(
        "  [grid_scalar={}x{}]",
        grid_scalar_width, grid_scalar_height
    );
    println!("  [grid_size={}×{}]", grid_width, grid_height);

    // initialize vector for each palette color for each grid cell
    // e.g. [0, 0, 0] maps to [color_1, color_2, color_3]
    // we will increment the value at idnex when a pixel is closest to a particular color
    // end result will allow us to determine most representative palette color for a grid cell
    let mut color_counts = vec![];
    for _ in 0..output_size {
        let mut row = vec![];

        for _ in 0..output_size {
            let colors = vec![0; palette_size];
            row.push(colors);
        }

        color_counts.push(row);
    }

    // println!("[color_counts={:?}]", color_counts);

    let mut pixelated = DynamicImage::new_rgba8(output_width, output_height);

    for x in 0..width {
        for y in 0..height {
            // break up image into windows based on output bit size
            // e.g. 8x8 pixel art would break image into 8x8 grid
            // for each grid cell (window), calculate most frequent color
            // color that cell with that color in final output

            // let grid_x = x / grid_size;
            let grid_x = (x as f32 / grid_scalar_width).floor() as u32;
            // let grid_y = y / grid_size;
            let grid_y = (y as f32 / grid_scalar_height).floor() as u32;

            // keep grid within output_size
            let grid_x = if grid_x >= output_size {
                output_size - 1
            } else {
                grid_x
            };

            let grid_y = if grid_y >= output_size {
                output_size - 1
            } else {
                grid_y
            };

            let debug = false;
            // let debug = grid_x == 3 && grid_y == 3;

            let grid_cell = color_counts
                .get_mut(grid_x as usize)
                .unwrap()
                .get_mut(grid_y as usize)
                .unwrap();

            let pixel = img.get_pixel(x, y);

            let closest_index = pixel_art_image::closest_rgb(&palette, &pixel, debug);
            grid_cell[closest_index] += 1;

            if debug {
                println!(
                    "[{},{}]({},{})={:?} [closest_index={}]",
                    grid_x, grid_y, x, y, pixel, closest_index
                )
            }

            // println!(
            //     "({},{})=({},{}) [closest_index={}] {:?}",
            //     x, y, grid_x, grid_y, closest_index, grid_cell
            // );
        }
    }

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

    // try zealous cropping at this point once we are finished?
    // pixel_art_image::print(&pixelated);
    if squared_output {
        pixelated = pixel_art_image::zealous_crop(&pixelated, true);
    }

    pixel_art_image::output(&pixelated, "./output/pixelated.png")?;

    return Ok(());
}
