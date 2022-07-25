use image::{Pixel, Rgba};

pub fn closest_rgb(color_list: &Vec<Rgba<u8>>, pixel: &Rgba<u8>, debug: bool) -> usize {
    let mut closest_index = 0;
    let mut closest_diff = rgba_diff(color_list.get(closest_index).unwrap(), &pixel);

    for color_index in 1..color_list.len() {
        let color_diff = rgba_diff(&color_list.get(color_index).unwrap(), &pixel);

        if debug {
            println!(
                "closest_rgb -- [{:?}][closest_diff = {}] {:?} v {:?} [color_diff = {}]",
                color_list.get(closest_index).unwrap(),
                closest_diff,
                &color_list.get(color_index).unwrap(),
                &pixel,
                color_diff
            );
        }

        if color_diff < closest_diff {
            closest_diff = color_diff;
            closest_index = color_index;
        }
    }

    // println!("closest_rgb -- {} :: {}", closest_index, closest_diff);
    return closest_index;
}

fn rgba_diff(pixel_1: &Rgba<u8>, pixel_2: &Rgba<u8>) -> u32 {
    if let [r1, g1, b1, a1] = pixel_1.channels() {
        if let [r2, g2, b2, a2] = pixel_2.channels() {
            // require exact alpha match for transparent pixels
            let is_transparent = *a1 == 0 || *a2 == 0;
            if is_transparent {
                if *a1 == 0 && *a2 == 0 {
                    return 0;
                }

                return u32::MAX;
            }

            // ... otherwise return total difference of each rgb value
            let mut total_diff: u32 = 0;

            total_diff += r1.abs_diff(*r2) as u32;
            total_diff += g1.abs_diff(*g2) as u32;
            total_diff += b1.abs_diff(*b2) as u32;

            // println!("rgba_diff -- {:?} :: {:?} ({})", pixel_1, pixel_2, total_diff);

            return total_diff;
        }
    }

    return 0;
}
