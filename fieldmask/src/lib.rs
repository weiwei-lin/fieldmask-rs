mod mask;
mod maskable;

pub use fieldmask_derive::{Maskable, OptionMaskable, SelfMaskable};

pub use mask::{Mask, MaskInput};
pub use maskable::{DeserializeMaskError, Maskable, OptionMaskable, SelfMaskable, UpdateOptions};
