use crate::algo as base;

use ring::{hkdf, hmac::SigningKey};
use std::sync::Arc;

pub struct KeySettings {
    salt: Vec<u8>,
    extracted: Vec<Vec<u8>>,
}

#[derive(Clone)]
pub struct Key(Arc<SigningKey>);
impl base::SafeGenerateKey for Key {
    type Settings = KeySettings;
    fn safe_generate<'a>(settings: &'a Self::Settings) -> Self {
        let mut extracted_key = SigningKey::new(&ring::digest::SHA384, settings.salt.as_slice());
        for k in settings.extracted.iter() {
            extracted_key = hkdf::extract(&extracted_key, k.as_slice());
        }
        Self(Arc::new(extracted_key))
    }
}
impl std::ops::Deref for Key {
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
    type ConstructionData = (Vec<u8>, Vec<Vec<u8>>);
    fn key_settings<'a>(&'a self) -> &'a <Self::Key as base::SafeGenerateKey>::Settings {
        &self.settings
    }
    fn new((salt, extracted): (Vec<u8>, Vec<Vec<u8>>)) -> Self {
        Self {
            settings: KeySettings {
                salt: salt,
                extracted: extracted,
            },
        }
    }
}
impl Algo {
    pub fn generate(
        &self,
        key: <Self as base::Algo>::Key,
        data: &[&[u8]],
        size: usize,
    ) -> Vec<Vec<u8>> {
        data.iter()
            .map(|d| {
                let mut out_buffer = vec![0; size];
                hkdf::expand(&key, d, out_buffer.as_mut_slice());
                out_buffer
            })
            .collect()
    }
}

impl AsRef<Key> for &Key {
    fn as_ref(&self) -> &Key {
        self
    }
}
