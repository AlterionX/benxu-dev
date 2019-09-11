use rand::{
    RngCore,
    rngs::OsRng,
};
use argon2rs::{
    Argon2,
    Variant,
};

use crate::crypto::algo::{
    hash::symmetric as sym,
    self as base,
};

#[derive(Clone)]
pub struct Key {
    /// 0 < len < 33
    secret_key: Vec<u8>,
}
impl base::SafeGenerateKey for Key {
    type Settings = ();
    fn generate(_: &Self::Settings) -> Self {
        let mut generated_secret = vec![0; Algo::SECRET_LEN as usize];
        OsRng.fill_bytes(generated_secret.as_mut_slice());
        Key::new(&generated_secret)
    }
}
impl sym::Key for Key {}
impl Key {
    pub fn new(secret: &Vec<u8>) -> Self {
        Self {
            secret_key: secret.clone(),
        }
    }
}

// TODO zero copy
pub struct SigningData {
    msg: Vec<u8>,
    salt: [u8; Algo::SALT_LEN as usize],
    /// 4 <= hash_len < 2^32
    hash_len: u32,
}
impl SigningData {
    pub fn new(msg: Vec<u8>, salt: Option<[u8; Algo::SALT_LEN as usize]>, hash_len: Option<u32>) -> Result<Self, ()> {
        let salt = salt.unwrap_or_else(|| {
            let mut generated_salt = [0; Algo::SALT_LEN as usize];
            OsRng.fill_bytes(&mut generated_salt);
            generated_salt
        });
        let hash_len = hash_len.unwrap_or(32u32);
        if hash_len < 4 {
            return Err(());
        }
        Ok(Self {
            msg: msg,
            salt: salt,
            hash_len: hash_len,
        })
    }
    pub fn new_default_hash_len(msg: Vec<u8>, salt: Option<[u8; Algo::SALT_LEN as usize]>) -> Self {
        Self::new(msg, salt, None).unwrap()
    }
    pub fn salt(&self) -> &[u8] {
        &self.salt[..]
    }
}

pub struct Algo(Option<Vec<u8>>);
impl Algo {
    pub const SALT_LEN: u8 = 16;
    pub const SECRET_LEN: u8 = 32;
    pub const HASH_LEN: u8 = 32;
}
impl base::Algo for Algo {
    type Key = Key;
    fn key_settings<'a>(&'a self) -> &<<Self as base::Algo>::Key as base::Key>::Settings {
        &()
    }
}
impl sym::Algo for Algo {
    type SigningInput = SigningData;
    fn sign(msg: &Self::SigningInput, key: &Self::Key) -> Vec<u8> {
        let mut buffer = vec![0; Self::HASH_LEN as usize];
        Argon2::default(Variant::Argon2d).hash(
            buffer.as_mut_slice(),
            msg.msg.as_slice(),
            &msg.salt[..],
            key.secret_key.as_slice(),
            &[],
        );
        buffer
    }
    type VerificationInput = SigningData;
    fn verify(msg: &Self::VerificationInput, signature: &[u8], key: &Self::Key) -> bool {
        let mut buffer = vec![0; Self::HASH_LEN as usize];
        Argon2::default(Variant::Argon2d).hash(
            buffer.as_mut_slice(),
            msg.msg.as_slice(),
            &msg.salt[..],
            key.secret_key.as_slice(),
            &[],
        );
        buffer.as_slice() == signature
    }
}

