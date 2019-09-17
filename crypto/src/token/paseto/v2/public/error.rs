use crate::token::paseto::token::{DeserializeError, UnpackingError};

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
