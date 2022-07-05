// extend module crate::image with submodules
// e.g. crate::image::zealous_crop
mod crop;
mod output;
mod point;
mod zealous_crop;

// this affects the public API of this module
// we expose the internal zealous_crop fn on crate::image with `pub use`
// now it is accessible via `crate::image::zealous_crop`
pub use self::zealous_crop::zealous_crop;

pub use self::output::output;
