use crate::crypto::algo as base;

pub trait Key: base::Key {}
#[derive(Debug)]
pub enum DecryptError {
    Base,
}
#[derive(Debug)]
pub enum EncryptError {
    Base,
}
pub trait Algo: base::Algo where <Self as base::Algo>::Key: Key {
    type EncryptArgs: ?Sized;
    type DecryptArgs: ?Sized;
    fn decrypt(key: &Self::Key, data: &Self::DecryptArgs) -> Result<Vec<u8>, DecryptError>;
    fn encrypt(key: &Self::Key, data: &Self::EncryptArgs) -> Result<Vec<u8>, EncryptError>;
}

