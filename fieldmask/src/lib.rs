// Make macros from fieldmask_derive available in this crate.
// Without this, `::fieldmask::*` generated by fieldmask_derive will not work.
extern crate self as fieldmask;

mod mask;
mod maskable;

pub use fieldmask_derive::{Maskable, OptionMaskable, SelfMaskable};

pub use mask::{Mask, MaskInput};
pub use maskable::{DeserializeMaskError, Maskable, OptionMaskable, SelfMaskable, UpdateOptions};
