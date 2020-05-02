//! Bcrypt implementation.

use crate::algo::{self as base, hash::symmetric as sym};

#[derive(Clone)]
pub struct Key(u8);
impl base::Key for Key {
    type Settings = u8;
    type Error = ();
    fn generate(cost: &Self::Settings) -> Result<Self, Self::Error> {
        Key::new(*cost)
    }
}
impl sym::Key for Key {}
impl std::ops::Deref for Key {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &(bcrypt::DEFAULT_COST as u8)
    }
}
impl Key {
    pub fn new(cost: u8) -> Result<Self, ()> {
        if cost < 4 || cost > 31 {
            Err(())
        } else {
            Ok(Self(cost))
        }
    }
}

// TODO zero copy
pub struct SigningData {
    msg: Vec<u8>,
    salt: [u8; Algo::SALT_LEN as usize],
}
impl SigningData {
    pub fn new(&self, msg: Vec<u8>, salt: Option<[u8; Algo::SALT_LEN as usize]>) -> Self {
        let salt = salt.unwrap_or_else(|| {
            use rand::{rngs::OsRng, RngCore};
            let mut generated_salt = [0; Algo::SALT_LEN as usize];
            OsRng.fill_bytes(&mut generated_salt);
            generated_salt
        });
        Self {
            msg: msg,
            salt: salt,
        }
    }
    fn truncated_msg(&self) -> &[u8] {
        if self.msg.len() > 72 {
            &self.msg[..72]
        } else {
            self.msg.as_slice()
        }
    }
}

pub struct Algo;
impl Algo {
    pub const SALT_LEN: u8 = 16;
}
impl base::Algo for Algo {
    type Key = Key;
    fn key_settings<'a>(&'a self) -> &<<Self as base::Algo>::Key as base::Key>::Settings {
        &(bcrypt::DEFAULT_COST as u8)
    }
    fn new(_: Self::ConstructionData) -> Self {
        Self
    }
}
impl sym::Algo for Algo {
    type SigningInput = SigningData;
    fn sign(&self, msg: &Self::SigningInput, key: &Self::Key) -> Vec<u8> {
        let cost = **key;
        let salt = &msg.salt[..];
        let trunc = msg.truncated_msg();
        let mut buffer = vec![0; 24];
        bcrypt::bcrypt(cost as u32, salt, trunc, buffer.as_mut_slice());
        buffer
    }
    type VerificationInput = SigningData;
    fn verify(&self, msg: &Self::VerificationInput, signature: &[u8], key: &Self::Key) -> bool {
        self.sign(msg, key).as_slice() == signature
    }
}

impl AsRef<Key> for &Key {
    fn as_ref(&self) -> &Key {
        self
    }
}
