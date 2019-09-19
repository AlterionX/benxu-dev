//! The possible errors while using PASETO's protocols.

use crate::algo::cipher::symmetric as symm;
use serde_json as json;
use std::str::Utf8Error;

/// Errors that can occur when unpacking.
#[derive(Debug)]
pub enum UnpackError {
    /// More than the 4 possible sections of a PASETO token were provided.
    ExtraSections,
    /// More than the 3 mandatory sections of a PASETO token were provided.
    MissingSections,
    /// The protocol provided was incorrect.
    BadProtocol,
}

/// Error for compromised token integrity.
#[derive(Debug)]
pub struct BadSignature {}

/// Errors that can occur when deserializing.
#[derive(Debug)]
pub enum DeserializeError {
    /// Payload was not valid JSON data or the wrong struct was provided to be deserialized into.
    Json(json::Error),
    /// Payload was not valid utf8.
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

/// Errors that occur everywhere.
#[derive(Debug)]
pub enum Error {
    /// Error during serialization.
    Serialization(json::Error),
    /// Error during deserialization.
    Deserialization(DeserializeError),
    /// Error during signing.
    Signing(&'static str),
    /// Error during encryption (symmetric).
    SymmEncryption(symm::EncryptError),
    /// Error during decryption (symmetric).
    SymmDecryption(symm::DecryptError),
    /// Error when unpacking.
    Unpack(UnpackError),
    /// Signature validation failure.
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
