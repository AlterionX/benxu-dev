//! Base module for various cryptographic algorithms.

pub mod cipher;
pub mod hash;
pub mod key_deriv;

/// A trait implemented by all cryptographic algorithm keys. Allows for keys to be generated
/// provided an instance of the algorithm exists.
///
/// Intended to be used in conjunction with [`Algo`](crate::algo::Algo).
pub trait Key: Sized {
    /// Information required to generate a new [`Key`].
    type Settings;
    /// Error resulting in generating a new key.
    type Error: Sized;
    /// Generate a key based on the provided settings.
    fn generate(settings: &Self::Settings) -> Result<Self, Self::Error>;
}
/// A trait representing a [`Key`] that can be generated without errors.
///
/// [`Key`] is automatically derived for all structs implementing this trait.
pub trait SafeGenerateKey {
    /// Information required to generate a new [`Key`].
    type Settings;
    /// Generate a key based on the provided settings.
    fn safe_generate(settings: &Self::Settings) -> Self;
}
impl<S, K: SafeGenerateKey<Settings = S>> Key for K {
    type Settings = S;
    type Error = !;
    fn generate(
        settings: &<Self as SafeGenerateKey>::Settings,
    ) -> Result<Self, Self::Error> {
        Ok(Self::safe_generate(settings))
    }
}
/// A trait implemented by all cryptographic algorithms. Allows for getting the "key settings"
/// during key generation.
pub trait Algo {
    /// The [`Key`] associated with the [`Algo`].
    type Key: Key + Clone + Send + Sync;
    type ConstructionData = ();
    /// Fetch the settings for key generation from the [`Algo`].
    fn key_settings<'a>(&'a self) -> &'a <<Self as Algo>::Key as Key>::Settings;
    fn new(data: Self::ConstructionData) -> Self;
}
