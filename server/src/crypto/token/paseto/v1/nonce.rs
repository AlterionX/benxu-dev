use ring::{hmac, digest};
use rand::{
    rngs::OsRng,
    RngCore,
};

#[derive(Clone)]
pub struct Randomness([u8; 32]);
impl Randomness {
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

#[derive(Clone)]
pub struct Nonce([u8; 32]);
impl Nonce {
    pub fn create_from(randomness: Randomness, msg: &[u8]) -> Nonce {
        let randomness = randomness.0;
        let key = hmac::SigningKey::new(&digest::SHA384, &randomness);
        let hash = hmac::sign(&key, msg);
        let mut free_buffer = randomness; // should I just alloc another one...?
        free_buffer[0..32].copy_from_slice(&hash.as_ref()[0..32]);
        Nonce(free_buffer)
    }
    pub fn get_salt<'a>(&'a self) -> &'a [u8] {
        &self.0[0..16]
    }
    pub fn get_crypt_nonce<'a>(&'a self) -> &'a [u8] {
        &self.0[16..32]
    }
    pub fn as_slice<'a>(&'a self) -> &'a[u8] {
        &self.0
    }
}
#[cfg(test)]
mod unit_tests {
    use super::*;

    impl<'a> HasMsgBuffer<'a> for &str {
        fn msg(&self) -> &'a [u8] {
            self.as_bytes()
        }
    }
    #[test]
    fn test_nonce_derivation() {
        // Constants copied directly from paseto source.
        let msg_a = String::from("The quick brown fox jumped over the lazy dog.");
        let msg_b = String::from("The quick brown fox jumped over the lazy dof.");
        let nonce = hex::decode(String::from("808182838485868788898a8b8c8d8e8f")).expect("Failed to decode nonce!");

        let calculated_nonce_a = Nonce::create_from(&Randomness::precomputed(nonce), msg_a.as_bytes());
        let calculated_nonce_b = Nonce::create_from(&Randomness::precomputed(nonce), msg_b.as_bytes());

        assert_eq!(
            "5e13b4f0fc111bf0cf9de4e97310b687858b51547e125790513cc1eaaef173cc".to_owned(),
            hex::encode(&calculated_nonce_a)
        );
        assert_eq!(
            "e1ba992f5cccd31714fd8c73adcdadabb00d0f23955a66907170c10072d66ffd".to_owned(),
            hex::encode(&calculated_nonce_b)
        )
    }
}

