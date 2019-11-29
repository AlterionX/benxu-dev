use crate::algo::{self as base, cipher::symmetric as symm};

#[derive(Clone)]
pub struct Key {
    key: Vec<u8>,
    nonce: Vec<u8>,
}
impl base::SafeGenerateKey for Key {
    type Settings = ();
    fn safe_generate(_: &Self::Settings) -> Self {
        use rand::{rngs::OsRng, RngCore};
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

pub struct Algo(openssl::symm::Cipher);
impl base::Algo for Algo {
    type Key = Key;
    type ConstructionData = ();
    fn key_settings<'a>(
        &'a self,
    ) -> &'a <<Self as base::Algo>::Key as base::SafeGenerateKey>::Settings {
        &()
    }
    fn new(_: ()) -> Self {
        Self(openssl::symm::Cipher::aes_256_ctr())
    }
}
impl symm::Algo for Algo {}
impl symm::CanEncrypt for Algo {
    type EKey = Key;
    type Input = [u8];
    type Error = symm::EncryptError;
    fn encrypt(&self, key: &Self::EKey, msg: &Self::Input) -> Result<Vec<u8>, Self::Error> {
        openssl::symm::encrypt(self.0, key.as_key(), Some(key.as_nonce()), msg)
            .map_err(|_| symm::EncryptError::Base)
    }
}
impl symm::CanDecrypt for Algo {
    type DKey = Key;
    type Input = [u8];
    type Error = symm::DecryptError;
    fn decrypt(&self, key: &Self::DKey, msg: &Self::Input) -> Result<Vec<u8>, Self::Error> {
        openssl::symm::decrypt(self.0, key.as_key(), Some(key.as_nonce()), msg)
            .map_err(|_| symm::DecryptError::Base)
    }
}

impl AsRef<Key> for &Key {
    fn as_ref(&self) -> &Key {
        self
    }
}
