#![feature(const_str_as_bytes, proc_macro_hygiene, type_ascription, decl_macro, try_trait, associated_type_defaults)]

pub mod encoding;
pub mod token;
pub mod algo;
pub mod key_rotation;
pub use key_rotation::{
    CurrAndLastKey,
    KeyStore,
    KeyRotator,
};

// boolean to result convenience
pub use boolinator::Boolinator as BoolToResult;

