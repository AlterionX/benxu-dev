use crate::algo::{
    hash::{blake::two_b::{Algo as BLAKE2B, Key as BLAKE2B_KEY}, symmetric::Algo as HashA},
    Algo as A,
};
use rand::{rngs::OsRng, RngCore};

pub struct Randomness([u8; 24]);

// TODO use const generics for how much randomness once it becomes available
impl Randomness {
    pub fn new() -> Randomness {
        let mut nonce = [0; 24];
        OsRng.fill_bytes(&mut nonce);
        Randomness(nonce)
    }
    #[cfg(test)]
    pub fn precomputed(nonce: [u8; 24]) -> Randomness {
        Randomness(nonce)
    }
}

#[derive(Clone)]
pub struct Nonce([u8; 24]);
impl Nonce {
    pub fn recreate_nonce(old_nonce: &[u8]) -> Nonce {
        let mut nonce_data = [0; 24];
        for (idx, val) in old_nonce.iter().enumerate() {
            nonce_data[idx] = *val;
        }
        Nonce(nonce_data)
    }
    pub fn create_from(randomness: Randomness, msg: &[u8]) -> Nonce {
        let randomness = randomness.0;
        let key = BLAKE2B_KEY::new(randomness.to_vec(), 24);
        let hash = BLAKE2B::new(24).sign(msg, &key);
        let mut free_buffer = randomness; // should I just alloc another one...? Oh well.
        free_buffer[0..24].copy_from_slice(&hash[0..24]);
        Nonce(free_buffer)
    }
    pub fn as_slice<'a>(&'a self) -> &'a [u8] {
        &self.0
    }
}
