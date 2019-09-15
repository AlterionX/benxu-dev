use crate::algo as base;

pub trait Key: base::Key {
    type PublicKey;
    type PrivateKey;
    fn public_key(&self) -> &Self::PublicKey;
    fn private_key(&self) -> &Self::PrivateKey;
}
pub enum DecryptError {}
pub enum EncryptError {}
pub trait Algo: base::Algo where <Self as base::Algo>::Key: Key {
    fn public_decrypt(key: &<Self::Key as Key>::PublicKey, data: &[u8]) -> Result<Vec<u8>, DecryptError>;
    fn public_encrypt(key: &<Self::Key as Key>::PublicKey, data: &[u8]) -> Result<Vec<u8>, EncryptError>;
    fn private_decrypt(key: &<Self::Key as Key>::PrivateKey, data: &[u8]) -> Result<Vec<u8>, DecryptError>;
    fn private_encrypt(key: &<Self::Key as Key>::PrivateKey, data: &[u8]) -> Result<Vec<u8>, EncryptError>;
}

