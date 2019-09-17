use crate::algo::cipher::symmetric as symm;
use serde_json as json;
use std::str::Utf8Error;
#[derive(Debug)]
pub enum UnpackError {
    ExtraSections,
    MissingSections,
    MissingHeader,
    BadProtocol,
    BadPayloadLength,
}

#[derive(Debug)]
pub struct BadSignature {}

#[derive(Debug)]
pub enum DeserializeError {
    Json(json::Error),
    Utf8(Utf8Error),
}
impl From<json::Error> for DeserializeError {
    fn from(e: json::Error) -> Self {
        Self::Json(e)
    }
}
impl From<Utf8Error> for DeserializeError {
    fn from(e: Utf8Error) -> Self {
        Self::Utf8(e)
    }
}

#[derive(Debug)]
pub enum Error {
    Serialization(json::Error),
    Deserialization(DeserializeError),
    Signing(&'static str),
    SymmEncryption(symm::EncryptError),
    SymmDecryption(symm::DecryptError),
    Unpack(UnpackError),
    BadSignature(BadSignature),
}
impl From<json::Error> for Error {
    fn from(e: json::Error) -> Self {
        Error::from(DeserializeError::from(e))
    }
}
impl From<UnpackError> for Error {
    fn from(e: UnpackError) -> Self {
        Self::Unpack(e)
    }
}
impl From<BadSignature> for Error {
    fn from(e: BadSignature) -> Self {
        Self::BadSignature(e)
    }
}
impl From<DeserializeError> for Error {
    fn from(e: DeserializeError) -> Self {
        Self::Deserialization(e)
    }
}
impl From<symm::EncryptError> for Error {
    fn from(e: symm::EncryptError) -> Self {
        Self::SymmEncryption(e)
    }
}
impl From<symm::DecryptError> for Error {
    fn from(e: symm::DecryptError) -> Self {
        Self::SymmDecryption(e)
    }
}
