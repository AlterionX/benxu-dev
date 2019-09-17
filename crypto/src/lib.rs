#![feature(
    const_str_as_bytes,
    proc_macro_hygiene,
    type_ascription,
    decl_macro,
    try_trait,
    associated_type_defaults
)]

pub mod algo;
pub mod encoding;
pub mod key_rotation;
pub mod token;
pub use key_rotation::{
    Generational, KeyRotator, RotatingKeyFixture, RotatingKeyStore, StableKeyStore,
};

// boolean to result convenience
pub use boolinator::Boolinator as BoolToResult;

/// Always call this if you need the sodiumoxide-implemented things to work multithreaded.
pub fn multithread_init() -> Result<(), ()> {
    sodiumoxide::init()
}
