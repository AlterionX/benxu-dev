//! Nonce related functions.

use crate::algo::{
    hash::{
        hmac::sha384::{Algo as HMAC_SHA384, Key as HMAC_SHA384_KEY},
        symmetric::Algo as HashA,
    },
    Algo as A,
};
use rand::{rngs::OsRng, RngCore};

/// A 32-byte wide chunk of random bytes.
#[derive(Clone)]
pub struct Randomness([u8; 32]);
// TODO use const generics for how much randomness once it becomes available
impl Randomness {
    /// Generates the random bytes.
    pub fn new() -> Randomness {
        let mut nonce = [0; 32];
        OsRng.fill_bytes(&mut nonce);
        Randomness(nonce)
    }
    #[cfg(test)]
    pub fn precomputed(nonce: [u8; 32]) -> Randomness {
        Randomness(nonce)
    }
}

/// A 32-byte wide chunk of bytes, forming a nonce.
#[derive(Clone)]
pub struct Nonce([u8; 32]);
impl Nonce {
    /// Create the nonce with the provided bytes.
    pub fn recreate_nonce(old_nonce: &[u8]) -> Nonce {
        let mut nonce_data = [0; 32];
        for (idx, val) in old_nonce.iter().enumerate() {
            nonce_data[idx] = *val;
        }
        Nonce(nonce_data)
    }
    /// Creates the nonce out of the random bytes provided.
    pub fn create_from(randomness: Randomness, msg: &[u8]) -> Nonce {
        let randomness = randomness.0;
        let key = HMAC_SHA384_KEY::new(&randomness);
        let hash = HMAC_SHA384::new(()).sign(msg, &key);
        let mut free_buffer = randomness; // should I just alloc another one...? Oh well.
        free_buffer[0..32].copy_from_slice(&hash[0..32]);
        Nonce(free_buffer)
    }
    /// Gets the component of the nonce representing the salt used for hashing.
    pub fn get_salt<'a>(&'a self) -> &'a [u8] {
        &self.0[0..16]
    }
    /// Gets the component of the nonce representing the salt used for encryption.
    pub fn get_crypt_nonce<'a>(&'a self) -> &'a [u8] {
        &self.0[16..32]
    }
    /// Gets the entire nonce as a slice.
    pub fn as_slice<'a>(&'a self) -> &'a [u8] {
        &self.0
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_nonce_derivation() {
        // Constants copied directly from paseto source.
        let msg_a = String::from("The quick brown fox jumped over the lazy dog.");
        let msg_b = String::from("The quick brown fox jumped over the lazy dof.");
        let nonce = hex::decode(String::from(
            "808182838485868788898a8b8c8d8e8f00000000000000000000000000000000",
        ))
        .expect("Failed to decode nonce!");
        let mut nonce_arr = [0u8; 32];
        println!("{:?}", nonce.len());
        debug_assert!(nonce.len() == 32, "Original nonce has incorrect length.");
        for (i, b) in nonce.as_slice().iter().enumerate() {
            nonce_arr[i] = *b;
        }

        let calculated_nonce_a =
            Nonce::create_from(Randomness::precomputed(nonce_arr.clone()), msg_a.as_bytes());
        let calculated_nonce_b =
            Nonce::create_from(Randomness::precomputed(nonce_arr), msg_b.as_bytes());

        assert_eq!(
            "5e13b4f0fc111bf0cf9de4e97310b687858b51547e125790513cc1eaaef173cc".to_owned(),
            hex::encode(calculated_nonce_a.as_slice())
        );
        assert_eq!(
            "e1ba992f5cccd31714fd8c73adcdadabb00d0f23955a66907170c10072d66ffd".to_owned(),
            hex::encode(calculated_nonce_b.as_slice())
        )
    }
}
