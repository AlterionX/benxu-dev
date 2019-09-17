use crate::algo::{self as base, hash::symmetric as sym};
use rand::{rngs::OsRng, RngCore};
use ring::{digest, hmac};
use std::{ops::Deref, sync::Arc};

#[derive(Clone)]
pub struct Key(Arc<hmac::SigningKey>);
impl base::SafeGenerateKey for Key {
    type Settings = ();
    fn generate(_: &()) -> Self {
        let mut nonce = [0; 32];
        OsRng.fill_bytes(&mut nonce);
        Key::new(&nonce)
    }
}
impl sym::Key for Key {}
impl Deref for Key {
    type Target = hmac::SigningKey;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Key {
    pub fn new(randomness: &[u8]) -> Self {
        Self(Arc::new(hmac::SigningKey::new(&digest::SHA384, randomness)))
    }
}

pub struct Algo;
impl base::Algo for Algo {
    type Key = Key;
    fn key_settings<'a>(&'a self) -> &<<Self as base::Algo>::Key as base::Key>::Settings {
        &()
    }
}
impl sym::Algo for Algo {
    type SigningInput = [u8];
    fn sign(input: &Self::SigningInput, key: &Self::Key) -> Vec<u8> {
        let key = &key;
        hmac::sign(&key, input).as_ref().to_vec()
    }
    type VerificationInput = [u8];
    fn verify(input: &Self::VerificationInput, signature: &[u8], key: &Self::Key) -> bool {
        hmac::verify_with_own_key(&key, input, signature).is_ok()
    }
}
