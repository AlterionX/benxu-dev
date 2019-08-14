use crate::crypto::algo::{
    self as base,
};

use std::ops::{Deref, DerefMut};
use ring::{
    hmac::SigningKey,
    digest::SHA384,
    hkdf,
};

#[derive(Clone)]
pub struct Key(Vec<Vec<u8>>, Vec<u8>);
impl base::Key for Key {}
impl Key {
    pub fn create(data: &[u8]) -> Self {
        Key(Vec::new(), data.to_vec())
    }
    fn init(&self) -> SigningKey {
        SigningKey::new(&SHA384, &self.1[..])
    }
    pub fn extract(&mut self, randomness: &[u8]) {
        (*self).push(randomness.to_vec());
    }
}
impl Deref for Key {
    type Target = Vec<Vec<u8>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Key {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
pub struct Algo;
impl base::Algo for Algo {
    type Key = Key;
    fn generate_key() -> Self::Key {
        Key(Vec::new(), Vec::new())
    }
}
impl Algo {
    pub fn generate(key: <Self as base::Algo>::Key, data: &[&[u8]], size: usize) -> Vec<Vec<u8>> {
        let mut extracted_key = key.init();
        for k in (*key).iter() {
            extracted_key = hkdf::extract(&extracted_key, &k[..]);
        }
        data.iter().map(|d| {
            let mut out_buffer = vec![0; size];
            hkdf::expand(&extracted_key, d, &mut out_buffer[..]);
            out_buffer
        }).collect()
    }
}
