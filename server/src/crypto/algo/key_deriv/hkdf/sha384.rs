use crate::crypto::algo::{
    self as base,
};

use std::{
    ops::{ Deref },
    sync::Arc,
};
use ring::{
    hmac::SigningKey,
    digest::SHA384,
    hkdf,
};

pub struct KeySettings {
    salt: Vec<u8>,
    extracted: Vec<Vec<u8>>,
}

#[derive(Clone)]
pub struct Key(Arc<SigningKey>);
impl base::SafeGenerateKey for Key {
    type Settings = KeySettings;
    fn generate<'a>(settings: &'a Self::Settings) -> Self {
        let mut extracted_key = SigningKey::new(&SHA384, settings.salt.as_slice());
        for k in settings.extracted.iter() {
            extracted_key = hkdf::extract(&extracted_key, k.as_slice());
        }
        Self(Arc::new(extracted_key))
    }
}
impl Deref for Key {
    type Target = SigningKey;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
pub struct Algo {
    settings: KeySettings,
}
impl base::Algo for Algo {
    type Key = Key;
    fn key_settings<'a>(&'a self) -> &'a <<Self as base::Algo>::Key as base::SafeGenerateKey>::Settings {
        &self.settings
    }
}
impl Algo {
    pub fn new(salt: Vec<u8>, extracted: Vec<Vec<u8>>) -> Self {
        Self {
            settings: KeySettings {
                salt: salt,
                extracted: extracted,
            }
        }
    }
    pub fn generate(key: <Self as base::Algo>::Key, data: &[&[u8]], size: usize) -> Vec<Vec<u8>> {
        data.iter().map(|d| {
            let mut out_buffer = vec![0; size];
            hkdf::expand(&key, d, out_buffer.as_mut_slice());
            out_buffer
        }).collect()
    }
}
