use serde_json as json;
use crate::crypto::algo::cipher::symmetric as symm;

pub enum Error {
    Serialization(json::Error),
    Signing(&'static str),
    SymmEncryption(symm::EncryptError),
}

