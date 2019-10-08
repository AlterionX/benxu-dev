#![feature(
    proc_macro_hygiene,
    type_ascription,
    decl_macro,
    try_trait,
    associated_type_defaults,
    never_type,
)]

//! A crate gathering various algorithms from different crypto libraries and attempting to unify
//! their apis. Built to manage key rotations. Implements the PASETO token standard for standard
//! web cookie/token-based authorization. Repackages some encoding libraries as well.
//!
//! Mainly serves as a higher level crate reshipping other crates under an relatively cohesive,
//! easy to use api.

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
