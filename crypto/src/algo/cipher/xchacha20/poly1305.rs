pub use sodiumoxide::crypto::aead::xchacha20poly1305_ietf::Nonce;
use sodiumoxide::crypto::aead::xchacha20poly1305_ietf::{
    gen_key, gen_nonce, open, seal, Key as UnderlyingKey,
};

use crate::algo::{self as base, cipher::symmetric as symm};

#[derive(Clone)]
pub struct Key {
    store: Vec<u8>,
    key: UnderlyingKey,
}
impl base::SafeGenerateKey for Key {
    type Settings = ();
    fn safe_generate(_: &Self::Settings) -> Self {
        let key = gen_key();
        Self::new(key)
    }
}
impl symm::Key for Key {}
impl Key {
    pub fn new(key: UnderlyingKey) -> Self {
        Self {
            store: key.as_ref().to_vec(),
            key: key,
        }
    }
    fn underlying(&self) -> &'_ UnderlyingKey {
        &self.key
    }
}

pub struct EncryptArgs {
    pub plaintext: Vec<u8>,
    pub aad: Option<Vec<u8>>,
    pub nonce: Option<Nonce>,
}
pub struct DecryptArgs {
    pub ciphertext: Vec<u8>,
    pub aad: Option<Vec<u8>>,
    pub nonce: Nonce,
}

pub struct Algo;
impl base::Algo for Algo {
    type Key = Key;
    type ConstructionData = ();
    fn key_settings<'a>(
        &'a self,
    ) -> &'a <<Self as base::Algo>::Key as base::SafeGenerateKey>::Settings {
        &()
    }
    fn new(_: ()) -> Self {
        Self
    }
}
impl symm::Algo for Algo {}
impl symm::CanEncrypt for Algo {
    type EKey = Key;
    type Input = EncryptArgs;
    type Error = symm::EncryptError;
    fn encrypt(&self, key: &Self::EKey, msg: &Self::Input) -> Result<Vec<u8>, Self::Error> {
        let nonce = if let Some(n) = msg.nonce {
            n
        } else {
            gen_nonce()
        };
        Ok(seal(
            msg.plaintext.as_slice(),
            msg.aad.as_ref().map(|aad| aad.as_slice()),
            &nonce,
            key.underlying(),
        ))
    }
}
impl symm::CanDecrypt for Algo {
    type DKey = Key;
    type Input = DecryptArgs;
    type Error = symm::DecryptError;
    fn decrypt(&self, key: &Self::DKey, msg: &Self::Input) -> Result<Vec<u8>, Self::Error> {
        open(
            msg.ciphertext.as_slice(),
            msg.aad.as_ref().map(|aad| aad.as_slice()),
            &msg.nonce,
            key.underlying(),
        )
        .map_err(|_| symm::DecryptError::Base)
    }
}
