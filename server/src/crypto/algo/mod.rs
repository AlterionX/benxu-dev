pub mod cipher;
pub mod hash;
pub mod key_deriv;

pub trait Key {
    type Settings;
    fn generate(settings: &Self::Settings) -> Self;
}
pub trait Algo {
    type Key: Key + Clone + Send + Sync;
    fn key_settings<'a>(&'a self) -> &'a <<Self as Algo>::Key as Key>::Settings;
}

