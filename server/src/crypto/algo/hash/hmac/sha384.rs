use ring::{
    hmac,
    digest,
};
use rand::{
    rngs::OsRng,
    RngCore,
};
use std::ops::Deref;
use crate::crypto::algo::{
    hash::symmetric as sym,
    self as base,
};

#[derive(Clone)]
pub struct Key(Vec<u8>);
impl base::Key for Key {}
impl sym::Key for Key {}
impl Deref for Key {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0
    }
}
impl Key {
    pub fn new(randomness: &[u8]) -> Key {
        Self(randomness.to_vec())
    }
    fn create_signing_key(&self) -> hmac::SigningKey {
        hmac::SigningKey::new(&digest::SHA384, &self)
    }
}

pub struct Algo;
impl base::Algo for Algo {
    type Key = Key;
    fn generate_key() -> Self::Key {
        let mut nonce = [0; 32];
        OsRng.fill_bytes(&mut nonce);
        Key(nonce.to_vec())
    }
}
impl sym::Algo for Algo {
    fn sign(msg: &[u8], key: &Self::Key) -> Vec<u8> {
        let key = key.create_signing_key();
        hmac::sign(&key, msg).as_ref().to_vec()
    }
    fn verify(msg: &[u8], signature: &[u8], key: &Self::Key) -> bool {
        hmac::verify_with_own_key(
            &key.create_signing_key(),
            msg,
            signature,
        ).is_ok()
    }
}

