use crate::crypto::algo as base;

pub struct PlainTextAlgo;

impl base::Key for () {}
impl base::cipher::symmetric::Key for () {}

impl base::Algo for PlainTextAlgo {
    fn generate_key() -> Self::Key { () }
    type Key = ();
}

impl super::symmetric::Algo for PlainTextAlgo {
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
    fn public_decrypt(key: &<Self::Key as super::asymmetric::Key>::PublicKey, data: &[u8]) -> Result<Vec<u8>, super::asymmetric::DecryptError> { Ok(data.to_vec()) }
    fn public_encrypt(key: &<Self::Key as super::asymmetric::Key>::PublicKey, data: &[u8]) -> Result<Vec<u8>, super::asymmetric::EncryptError> { Ok(data.to_vec()) }
    fn private_decrypt(key: &<Self::Key as super::asymmetric::Key>::PrivateKey, data: &[u8]) -> Result<Vec<u8>, super::asymmetric::DecryptError> { Ok(data.to_vec()) }
    fn private_encrypt(key: &<Self::Key as super::asymmetric::Key>::PrivateKey, data: &[u8]) -> Result<Vec<u8>, super::asymmetric::EncryptError> { Ok(data.to_vec()) }
}


