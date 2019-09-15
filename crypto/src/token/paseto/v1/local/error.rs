use crate::{
    token::paseto::token::{UnpackingError, DeserializeError},
    algo::cipher::symmetric::{EncryptError, DecryptError},
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
    fn from(e: serde_json::Error) -> Error {
        Error::Unpack
    }
}
impl From<UnpackingError> for Error {
    fn from(e: UnpackingError) -> Error {
        Error::Unpack
    }
}
impl From<EncryptError> for Error {
    fn from(e: EncryptError) -> Error {
        Error::Encrypt
    }
}
impl From<DecryptError> for Error {
    fn from(e: DecryptError) -> Error {
        Error::Decrypt
    }
}
impl From<DeserializeError> for Error {
    fn from(e: DeserializeError) -> Error {
        Error::Deserialize
    }
}

