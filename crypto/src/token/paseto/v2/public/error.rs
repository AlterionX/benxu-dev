use crate::{
    // algo::hash::ecc::ed25519::AlgoError,
    token::paseto::token::{UnpackingError, DeserializeError},
};

#[derive(Debug)]
pub enum Error {
    // Encrypt
    Serialization,
    Signing,
    // Decrypt
    Unpack,
    BadSignature,
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
impl From<DeserializeError> for Error {
    fn from(_: DeserializeError) -> Self {
        Error::Deserialize
    }
}

