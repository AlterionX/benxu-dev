use crate::crypto::{
    algo::cipher::symmetric::{EncryptError, DecryptError},
    token::paseto::token::{UnpackingError, DeserializeError},
};

#[derive(Debug)]
pub enum Error {
    // Encrypt
    Serialization,
    Encryption,
    // Decrypt
    Unpack,
    Decryption,
    Deserialize,
}
impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        Error::Serialization
    }
}
impl From<UnpackingError> for Error {
    fn from(_: UnpackingError) -> Self {
        Error::Unpack
    }
}
impl From<EncryptError> for Error {
    fn from(_: EncryptError) -> Self {
        Error::Encryption
    }
}
impl From<DecryptError> for Error {
    fn from(_: DecryptError) -> Self {
        Error::Decryption
    }
}
impl From<DeserializeError> for Error {
    fn from(_: DeserializeError) -> Self {
        Error::Deserialize
    }
}

