//! A module for that adds a method for converting from [`rocket_contrib::uuid::Uuid`] back to
//! the vanilla [`uuid::Uuid`]. Implemented as a trait since both objects are external to the
//! crate. The function [`rocket_contrib::uuid::Uuid::into_inner()`] could not be used since
//! [`rocket_contrib::uuid::Uuid`] is based off a different version of the [`uuid`] crate.
//!
//! This can be implemented as such:
//! ```rust
//! impl FromRUuid for uuid::Uuid {
//!     fn from_ruuid(ruuid: rocket_contrib::uuid::Uuid) -> Self {
//!         // nasty conversion code (not really that nasty, just awkward)
//!     }
//! }
//! ```

/// Trait encoding the conversion between different [`uuid`] library versions.
pub trait FromRUuid {
    /// This converts [`rocket_contrib::uuid::Uuid`] into the older [`uuid::Uuid`].
    fn from_ruuid(ruuid: rocket_contrib::uuid::Uuid) -> Self;
}
impl FromRUuid for uuid::Uuid {
    fn from_ruuid(ruuid: rocket_contrib::uuid::Uuid) -> Self {
        uuid::Uuid::from_uuid_bytes(*ruuid.into_inner().as_bytes())
    }
}

