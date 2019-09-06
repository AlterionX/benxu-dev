use crate::crypto::algo::{
    self as base,
    cipher::{
        asymmetric as asymm,
        symmetric as symm,
    },
};

pub struct PlainTextAlgo;

impl base::SafeGenerateKey for () {
    type Settings = ();
    fn generate(_: &()) -> Self { () }
}
impl base::cipher::symmetric::Key for () {}

impl base::Algo for PlainTextAlgo {
    type Key = ();
    fn key_settings<'a>(&'a self) -> &'a <<Self as base::Algo>::Key as base::SafeGenerateKey>::Settings { &() }
}
impl symm::Algo for PlainTextAlgo {
    type EncryptArgs = [u8];
    type DecryptArgs = [u8];
    fn decrypt(key: &Self::Key, data: &[u8]) -> Result<Vec<u8>, super::symmetric::DecryptError> { Ok(data.to_vec()) }
    fn encrypt(key: &Self::Key, data: &[u8]) -> Result<Vec<u8>, super::symmetric::EncryptError> { Ok(data.to_vec()) }
}

impl super::asymmetric::Key for () {
    type PublicKey = ();
    type PrivateKey = ();
    fn public_key<'a>(&'a self) -> &'a Self::PublicKey { self }
    fn private_key<'a>(&'a self) -> &'a Self::PrivateKey { self }
}
impl super::asymmetric::Algo for PlainTextAlgo {
    fn public_decrypt(key: &<Self::Key as asymm::Key>::PublicKey, data: &[u8]) -> Result<Vec<u8>, asymm::DecryptError> { Ok(data.to_vec()) }
    fn public_encrypt(key: &<Self::Key as asymm::Key>::PublicKey, data: &[u8]) -> Result<Vec<u8>, asymm::EncryptError> { Ok(data.to_vec()) }
    fn private_decrypt(key: &<Self::Key as asymm::Key>::PrivateKey, data: &[u8]) -> Result<Vec<u8>, asymm::DecryptError> { Ok(data.to_vec()) }
    fn private_encrypt(key: &<Self::Key as asymm::Key>::PrivateKey, data: &[u8]) -> Result<Vec<u8>, asymm::EncryptError> { Ok(data.to_vec()) }
}


