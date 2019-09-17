pub mod cipher;
pub mod hash;
pub mod key_deriv;

pub trait Key: Sized {
    type Settings;
    type Error: Sized;
    fn generate_with_err(settings: &Self::Settings) -> Result<Self, Self::Error>;
}
pub trait SafeGenerateKey {
    type Settings;
    fn generate(settings: &Self::Settings) -> Self;
}
impl<S, K: SafeGenerateKey<Settings = S>> Key for K {
    type Settings = S;
    type Error = ();
    fn generate_with_err(
        settings: &<Self as SafeGenerateKey>::Settings,
    ) -> Result<Self, <Self as Key>::Error> {
        Ok(Self::generate(settings))
    }
}
pub trait Algo {
    type Key: Key + Clone + Send + Sync;
    fn key_settings<'a>(&'a self) -> &'a <<Self as Algo>::Key as Key>::Settings;
}
