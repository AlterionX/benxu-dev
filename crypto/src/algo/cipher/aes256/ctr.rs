use crate::algo::{self as base, cipher::symmetric as symm};

use openssl::symm::{decrypt, encrypt, Cipher};
use rand::{rngs::OsRng, RngCore};

#[derive(Clone)]
pub struct Key {
    key: Vec<u8>,
    nonce: Vec<u8>,
}
impl base::SafeGenerateKey for Key {
    type Settings = ();
    fn generate(setting: &Self::Settings) -> Self {
        let mut key = vec![0u8; 32];
        OsRng.fill_bytes(key.as_mut_slice());
        let mut nonce = vec![0u8; 16];
        OsRng.fill_bytes(nonce.as_mut_slice());
        Self::new(key.as_slice(), nonce.as_slice())
    }
}
impl symm::Key for Key {}
impl Key {
    pub fn new(key: &[u8], nonce: &[u8]) -> Self {
        Self {
            key: key.to_vec(),
            nonce: nonce.to_vec(),
        }
    }
    fn as_key<'a>(&'a self) -> &'a [u8] {
        self.key.as_slice()
    }
    fn as_nonce<'a>(&'a self) -> &'a [u8] {
        self.nonce.as_slice()
    }
}

pub struct Algo(Cipher);
impl base::Algo for Algo {
    type Key = Key;
    fn key_settings<'a>(
        &'a self,
    ) -> &'a <<Self as base::Algo>::Key as base::SafeGenerateKey>::Settings {
        &()
    }
}
impl symm::Algo for Algo {
    type EncryptArgs = [u8];
    type DecryptArgs = [u8];
    fn encrypt(key: &Key, msg: &[u8]) -> Result<Vec<u8>, symm::EncryptError> {
        encrypt(
            Cipher::aes_256_ctr(),
            key.as_key(),
            Some(key.as_nonce()),
            msg,
        )
        .map_err(|_| symm::EncryptError::Base)
    }
    fn decrypt(key: &Key, msg: &[u8]) -> Result<Vec<u8>, symm::DecryptError> {
        decrypt(
            Cipher::aes_256_ctr(),
            key.as_key(),
            Some(key.as_nonce()),
            msg,
        )
        .map_err(|_| symm::DecryptError::Base)
    }
}
