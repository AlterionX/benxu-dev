use super::local_prelude::*;

#[derive(Debug)]
pub enum Error {
    // encryption
    Serialization,
    Signing,
    // decryption
    Unpacking,
    BadHeader,
    MalformedBody,
    Verifying,
    BadSignature,
    Deserialization,
}
impl From<json::Error> for Error {
    fn from(_: json::Error) -> Self {
        Error::Serialization
    }
}
impl From<token::UnpackingError> for Error {
    fn from(_: token::UnpackingError) -> Self {
        Error::Unpacking
    }
}
impl From<token::DeserializeError> for Error {
    fn from(_: token::DeserializeError) -> Self {
        Error::Deserialization
    }
}
