// extend module crate::image with submodules
// e.g. crate::image::zealous_crop
mod colors;
mod crop;
mod output;
mod palette;
mod palette_bar;
mod pixelate;
mod point;
mod print;
mod zealous_crop;

// this affects the public API of this module
// we expose the internal zealous_crop fn on crate::image with `pub use`
// now it is accessible via `crate::image::zealous_crop`
pub use self::colors::closest_rgb;
pub use self::output::{output, source_profile};
pub use self::palette::palette;
pub use self::palette_bar::{with_palette_bar, PaletteBar};
pub use self::pixelate::{pixelate, Sampling};
pub use self::print::print;
pub use self::zealous_crop::zealous_crop;
