use std::ops::Deref;
use rand::{
    rngs::OsRng,
    RngCore,
};
use blake2_rfc::blake2b::{blake2b};
use crate::crypto::algo::{
    hash::symmetric as sym,
    self as base,
};

#[derive(Clone)]
pub struct Key(Vec<u8>, usize);
impl base::SafeGenerateKey for Key {
    type Settings = usize;
    fn generate(hash_len: &usize) -> Self {
        let mut nonce = vec![0; *hash_len];
        OsRng.fill_bytes(nonce.as_mut_slice());
        Key::new(nonce, *hash_len)
    }
}
impl sym::Key for Key {}
impl Deref for Key {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Key {
    pub fn new(randomness: Vec<u8>, hash_len: usize) -> Self {
        Self(randomness, hash_len)
    }
    pub fn hash_len(&self) -> usize {
        self.1
    }
}

pub struct Algo(usize);
impl base::Algo for Algo {
    type Key = Key;
    fn key_settings<'a>(&'a self) -> &<<Self as base::Algo>::Key as base::Key>::Settings { &self.0 }
}
impl sym::Algo for Algo {
    type SigningInput = [u8];
    fn sign(msg: &Self::SigningInput, key: &Self::Key) -> Vec<u8> {
        let key = &key;
        blake2b(key.hash_len(), &key, msg).as_bytes().to_vec()
    }
    type VerificationInput = [u8];
    fn verify(msg: &Self::VerificationInput, signature: &[u8], key: &Self::Key) -> bool {
        blake2b(key.hash_len(), &key, msg) == *signature
    }
}

