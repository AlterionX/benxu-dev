use crate::{
    algo::cipher::symmetric::{DecryptError, EncryptError},
    token::paseto::token::{DeserializeError, UnpackingError},
};

#[derive(Debug)]
pub enum Error {
    // encryption process
    Unpack,
    Sign,
    Encrypt,
    // decryption process
    BadHeader,
    BadSignature,
    Decrypt,
    Deserialize,
}
impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Error {
        Error::Unpack
    }
}
impl From<UnpackingError> for Error {
    fn from(_: UnpackingError) -> Error {
        Error::Unpack
    }
}
impl From<EncryptError> for Error {
    fn from(_: EncryptError) -> Error {
        Error::Encrypt
    }
}
impl From<DecryptError> for Error {
    fn from(_: DecryptError) -> Error {
        Error::Decrypt
    }
}
impl From<DeserializeError> for Error {
    fn from(_: DeserializeError) -> Error {
        Error::Deserialize
    }
}
