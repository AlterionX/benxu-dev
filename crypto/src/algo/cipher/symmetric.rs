use crate::algo as base;

pub trait Key: base::Key {}
#[derive(Debug)]
pub enum DecryptError {
    Base,
}
#[derive(Debug)]
pub enum EncryptError {
    Base,
}
pub trait Algo: base::Algo where <Self as base::Algo>::Key: Key {}
pub trait CanEncrypt: Algo where <Self as base::Algo>::Key: Key {
    type EKey = <Self as base::Algo>::Key;
    type Input: ?Sized;
    type Error;
    fn encrypt(&self, key: &Self::EKey, data: &Self::Input) -> Result<Vec<u8>, Self::Error>;
}
pub trait CanDecrypt: Algo where <Self as base::Algo>::Key: Key {
    type DKey = <Self as base::Algo>::Key;
    type Input: ?Sized;
    type Error;
    fn decrypt(&self, key: &Self::DKey, data: &Self::Input) -> Result<Vec<u8>, Self::Error>;
}
