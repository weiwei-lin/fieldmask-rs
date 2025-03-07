pub use fieldmask_derive::Maskable;
pub use mask::{Mask, MaskInput};
pub use maskable::{DeserializeMaskError, Maskable, OptionMaskable, SelfMaskable, UpdateOptions};

mod mask;
mod maskable;
