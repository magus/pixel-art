use colored::*;
use image::GenericImageView;
use image::{DynamicImage, Rgba};

const PARTITIONS: usize = 3;
const OUTPUT_COLOR_COUNT: usize = 32;

#[derive(Clone, Default)]
struct ColorBucket {
    pixel_count: u64,
    channel_totals: [u64; 3],
}

impl ColorBucket {
    fn add_pixel(&mut self, pixel: Rgba<u8>) {
        self.pixel_count += 1;

        for channel in 0..3 {
            self.channel_totals[channel] += u64::from(pixel[channel]);
        }
    }

    fn mean(&self) -> [f64; 3] {
        self.channel_totals
            .map(|total| total as f64 / self.pixel_count as f64)
    }

    fn merge(&mut self, other: Self) {
        self.pixel_count += other.pixel_count;

        for channel in 0..3 {
            self.channel_totals[channel] += other.channel_totals[channel];
        }
    }

    fn color(&self) -> Rgba<u8> {
        let [red, green, blue] = self.mean().map(|channel| channel as u8);

        Rgba([red, green, blue, 255])
    }
}

pub fn palette(img: &DynamicImage) -> Vec<Rgba<u8>> {
    palette_with_options(img, OUTPUT_COLOR_COUNT, PARTITIONS)
}

/// Merge RGB buckets until the palette fits the requested color limit.
/// The limit applies to opaque colors; transparency is appended separately.
pub fn palette_with_options(
    img: &DynamicImage,
    output_color_count: usize,
    partitions: usize,
) -> Vec<Rgba<u8>> {
    assert!((1..=256).contains(&output_color_count));
    assert!((1..=32).contains(&partitions));

    let mut buckets = vec![ColorBucket::default(); partitions.pow(3)];

    for (_, _, pixel) in img.pixels() {
        if pixel[3] == 0 {
            continue;
        }

        let index = bucket_index(pixel, partitions);
        buckets[index].add_pixel(pixel);
    }

    buckets.retain(|bucket| bucket.pixel_count > 0);
    println!("\nMerging {} occupied color buckets...", buckets.len());

    merge_buckets(&mut buckets, output_color_count);
    buckets.sort_by(|left, right| right.pixel_count.cmp(&left.pixel_count));

    let mut output = Vec::with_capacity(buckets.len() + 1);

    for (index, bucket) in buckets.iter().enumerate() {
        let color = bucket.color();
        let [red, green, blue, _] = color.0;

        println!(
            "  space[{:>3}] [{:>8} pixels] {} {:?}",
            index,
            bucket.pixel_count,
            "     ".on_truecolor(red, green, blue),
            color,
        );

        output.push(color);
    }

    output.push(Rgba([0, 0, 0, 0]));

    output
}

fn merge_buckets(buckets: &mut Vec<ColorBucket>, color_limit: usize) {
    while buckets.len() > color_limit {
        let means: Vec<_> = buckets.iter().map(ColorBucket::mean).collect();
        let mut best_pair = (0, 1);
        let mut lowest_cost = f64::INFINITY;

        for left in 0..buckets.len() {
            for right in left + 1..buckets.len() {
                let cost = merge_cost(
                    means[left],
                    buckets[left].pixel_count,
                    means[right],
                    buckets[right].pixel_count,
                );

                if cost < lowest_cost {
                    lowest_cost = cost;
                    best_pair = (left, right);
                }
            }
        }

        let (left, right) = best_pair;
        let other = buckets.remove(right);
        buckets[left].merge(other);
    }
}

fn merge_cost(left: [f64; 3], left_count: u64, right: [f64; 3], right_count: u64) -> f64 {
    let mut squared_distance = 0.0;

    for channel in 0..3 {
        let difference = left[channel] - right[channel];
        squared_distance += difference * difference;
    }

    // This is the increase in total squared RGB error after a weighted merge.
    let left_count = left_count as f64;
    let right_count = right_count as f64;
    let weight = left_count * right_count / (left_count + right_count);

    weight * squared_distance
}

fn bucket_index(pixel: Rgba<u8>, partitions: usize) -> usize {
    let partition_width = (255.0 / partitions as f64).ceil() as usize;
    let red = usize::from(pixel[0].min(254)) / partition_width;
    let green = usize::from(pixel[1].min(254)) / partition_width;
    let blue = usize::from(pixel[2].min(254)) / partition_width;

    red + green * partitions + blue * partitions.pow(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    fn sample_image(samples: &[(u32, [u8; 4])]) -> DynamicImage {
        let width = samples.iter().map(|(count, _)| count).sum();
        let mut image = RgbaImage::new(width, 1);
        let mut x = 0;

        for &(count, color) in samples {
            for _ in 0..count {
                image.put_pixel(x, 0, Rgba(color));
                x += 1;
            }
        }

        DynamicImage::ImageRgba8(image)
    }

    #[test]
    fn similar_blues_merge_and_preserve_a_less_common_green() {
        let image = sample_image(&[
            (100, [100, 150, 200, 255]),
            (100, [116, 150, 200, 255]),
            (20, [60, 100, 40, 255]),
        ]);

        let palette = palette_with_options(&image, 2, 16);

        assert_eq!(
            palette,
            vec![
                Rgba([108, 150, 200, 255]),
                Rgba([60, 100, 40, 255]),
                Rgba([0, 0, 0, 0]),
            ],
        );
    }

    #[test]
    fn a_single_color_uses_the_weighted_mean_of_all_visible_pixels() {
        let image = sample_image(&[
            (3, [0, 0, 0, 255]),
            (1, [255, 255, 255, 255]),
            (10, [255, 0, 0, 0]),
        ]);

        let palette = palette_with_options(&image, 1, 16);

        assert_eq!(palette, vec![Rgba([63, 63, 63, 255]), Rgba([0, 0, 0, 0])]);
    }

    #[test]
    fn merge_choice_accounts_for_pixel_counts() {
        let image = sample_image(&[
            (100, [0, 0, 0, 255]),
            (100, [16, 0, 0, 255]),
            (1, [48, 0, 0, 255]),
        ]);

        let palette = palette_with_options(&image, 2, 16);

        // Merging the rare red costs less than merging the two closer, common colors.
        assert_eq!(
            palette,
            vec![
                Rgba([16, 0, 0, 255]),
                Rgba([0, 0, 0, 255]),
                Rgba([0, 0, 0, 0])
            ],
        );
    }

    #[test]
    fn sparse_and_transparent_images_do_not_add_empty_bucket_colors() {
        let solid = sample_image(&[(4, [90, 120, 160, 255])]);
        let transparent = sample_image(&[(4, [255, 255, 255, 0])]);

        for partitions in [1, 2, 4, 8, 16, 32] {
            assert_eq!(
                palette_with_options(&solid, 16, partitions),
                vec![Rgba([90, 120, 160, 255]), Rgba([0, 0, 0, 0])],
            );
            assert_eq!(
                palette_with_options(&transparent, 16, partitions),
                vec![Rgba([0, 0, 0, 0])],
            );
        }
    }
}
