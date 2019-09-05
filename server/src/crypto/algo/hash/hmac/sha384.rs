use std::{
    ops::Deref,
    sync::Arc,
};
use ring::{
    hmac,
    digest,
};
use rand::{
    rngs::OsRng,
    RngCore,
};
use crate::crypto::algo::{
    hash::symmetric as sym,
    self as base,
};

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
        Self(Arc::new(
            hmac::SigningKey::new(
                &digest::SHA384, randomness
            )
        ))
    }
}

pub struct Algo;
impl base::Algo for Algo {
    type Key = Key;
    fn key_settings<'a>(&'a self) -> &<<Self as base::Algo>::Key as base::Key>::Settings { &() }
}
impl sym::Algo for Algo {
    fn sign(msg: &[u8], key: &Self::Key) -> Vec<u8> {
        let key = &key;
        hmac::sign(&key, msg).as_ref().to_vec()
    }
    fn verify(msg: &[u8], signature: &[u8], key: &Self::Key) -> bool {
        hmac::verify_with_own_key(
            &key,
            msg,
            signature,
        ).is_ok()
    }
}

