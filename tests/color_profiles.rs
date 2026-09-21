use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use image::{DynamicImage, Rgba, RgbaImage};

// The RGB Display P3 profile embedded in the original IMG_0409 crop.
const DISPLAY_P3: &[u8] = include_bytes!("fixtures/display-p3.icc");
static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "pixel-art-color-profile-{}-{}-{}",
            std::process::id(),
            time.as_nanos(),
            sequence,
        ));
        fs::create_dir(&path).unwrap();

        Self(path)
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn source_pixels() -> RgbaImage {
    RgbaImage::from_fn(16, 12, |x, y| match (x / 8, y / 6) {
        (0, 0) => Rgba([114, 155, 202, 255]),
        (1, 0) => Rgba([200, 181, 86, 255]),
        (0, 1) => Rgba([121, 132, 80, 255]),
        _ => Rgba([0, 0, 0, 0]),
    })
}

fn write_source(path: &Path, profile: Option<&[u8]>) {
    let pixels = source_pixels();
    let mut info = png::Info::with_size(pixels.width(), pixels.height());
    info.color_type = png::ColorType::Rgba;
    info.icc_profile = profile.map(Into::into);

    let file = fs::File::create(path).unwrap();
    let encoder = png::Encoder::with_info(file, info).unwrap();
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(pixels.as_raw()).unwrap();
    writer.finish().unwrap();
}

fn render(input: &Path, output: &Path, palette_bar: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_pixel-art"));
    command
        .arg("--input")
        .arg(input)
        .arg("--output")
        .arg(output)
        .args(["--size", "8", "--colors", "3", "--partitions", "4"]);

    if palette_bar {
        command.args(["--palette-height", "2"]);
    }

    command.output().unwrap()
}

fn png_profile(path: &Path) -> Option<Vec<u8>> {
    let decoder = png::Decoder::new(fs::File::open(path).unwrap());
    let reader = decoder.read_info().unwrap();

    reader
        .info()
        .icc_profile
        .as_ref()
        .map(|profile| profile.to_vec())
}

#[test]
fn png_profile_survives_pixelation_without_changing_pixels_or_palette_bar() {
    let directory = TestDirectory::new();
    let tagged = directory.0.join("tagged.png");
    let untagged = directory.0.join("untagged.png");
    write_source(&tagged, Some(DISPLAY_P3));
    write_source(&untagged, None);

    for palette_bar in [false, true] {
        let tagged_output = directory.0.join("tagged-output.png");
        let untagged_output = directory.0.join("untagged-output.png");

        for (input, output) in [(&tagged, &tagged_output), (&untagged, &untagged_output)] {
            let result = render(input, output, palette_bar);
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
        }

        let tagged_image = image::open(&tagged_output).unwrap();
        let untagged_image = image::open(&untagged_output).unwrap();
        assert_eq!(png_profile(&tagged_output).as_deref(), Some(DISPLAY_P3));
        assert_eq!(png_profile(&untagged_output), None);
        assert_eq!(tagged_image.to_rgba8(), untagged_image.to_rgba8());
        assert_eq!(tagged_image.width(), 8);
        assert_eq!(tagged_image.height(), if palette_bar { 8 } else { 6 });
    }
}

#[test]
fn jpeg_input_keeps_its_profile_and_decoded_pixels() {
    let directory = TestDirectory::new();
    let untagged = directory.0.join("untagged.jpg");
    let tagged = directory.0.join("tagged.jpg");
    DynamicImage::ImageRgba8(source_pixels())
        .to_rgb8()
        .save(&untagged)
        .unwrap();

    // JPEG stores ICC profiles in APP2 segments, with a one-based segment number and count.
    let jpeg = fs::read(&untagged).unwrap();
    let mut segment = b"ICC_PROFILE\0\x01\x01".to_vec();
    segment.extend_from_slice(DISPLAY_P3);
    let mut bytes = jpeg[..2].to_vec();
    bytes.extend_from_slice(&[0xff, 0xe2]);
    bytes.extend_from_slice(&((segment.len() + 2) as u16).to_be_bytes());
    bytes.extend_from_slice(&segment);
    bytes.extend_from_slice(&jpeg[2..]);
    fs::write(&tagged, bytes).unwrap();

    assert_eq!(
        image::open(&tagged).unwrap().to_rgb8(),
        image::open(&untagged).unwrap().to_rgb8()
    );

    let output = directory.0.join("pixel-art.png");
    let result = render(&tagged, &output, false);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(png_profile(&output).as_deref(), Some(DISPLAY_P3));
}

#[test]
fn unsupported_profile_output_does_not_create_a_mislabelled_image() {
    let directory = TestDirectory::new();
    let input = directory.0.join("tagged.png");
    let output = directory.0.join("output.jpg");
    write_source(&input, Some(DISPLAY_P3));

    let result = render(&input, &output, false);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("Use a .png output"));
    assert!(!output.exists());
}

#[test]
fn non_rgb_profile_is_not_attached_to_rgb_pixels() {
    let directory = TestDirectory::new();
    let input = directory.0.join("non-rgb-profile.png");
    let output = directory.0.join("output.png");
    let mut profile = DISPLAY_P3.to_vec();
    profile[16..20].copy_from_slice(b"GRAY");
    write_source(&input, Some(&profile));

    let result = render(&input, &output, false);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("RGB color profile"));
    assert!(!output.exists());
}

#[test]
fn debug_logging_keeps_pixels_and_profile_for_every_sampling_mode() {
    let directory = TestDirectory::new();
    let input = directory.0.join("source.png");
    write_source(&input, Some(DISPLAY_P3));

    for sampling in ["mode", "mean", "center", "ink"] {
        let normal = directory.0.join(format!("{sampling}.png"));
        let debug = directory.0.join(format!("{sampling}-debug.png"));

        for (output, enabled) in [(&normal, false), (&debug, true)] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_pixel-art"));
            command
                .arg("--input")
                .arg(&input)
                .arg("--output")
                .arg(output)
                .args(["--size", "8", "--sampling", sampling]);

            if enabled {
                command.arg("--debug");
            }

            let result = command.output().unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );

            let stdout = String::from_utf8_lossy(&result.stdout);
            for message in [
                "🤖 pixelate",
                "[image = 16×12]",
                "[output = 8×6]",
                "[ratio = ",
                "[grid_scalar = 2x2]",
                "[grid_size = 2×2]",
            ] {
                assert!(stdout.contains(message), "Missing diagnostic: {message}");
            }

            if sampling == "mode" {
                for stage in [
                    "initialize_color_counts",
                    "create_pixelated_buffer",
                    "calcuate_pixelated_grid_cells",
                    "put_pixel_pixelated",
                ] {
                    assert!(stdout.contains(&format!("[time::{stage}]")));
                }

                assert_eq!(stdout.contains("[1,2] (2,4) = "), enabled);
            }

            assert_eq!(stdout.contains("[pixelate] ("), enabled);
            let matching = if sampling == "mean" {
                "[mean]"
            } else {
                "closest_rgb --"
            };
            assert_eq!(stdout.contains(matching), enabled);
            assert_eq!(png_profile(output).as_deref(), Some(DISPLAY_P3));
        }

        assert_eq!(
            image::open(normal).unwrap().to_rgba8(),
            image::open(debug).unwrap().to_rgba8(),
        );
    }
}
