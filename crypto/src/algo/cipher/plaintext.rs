//! A basic "cipher" that simply returns what was provided.

use crate::algo::{
    self as base,
    cipher::{asymmetric as asymm, symmetric as symm},
};

pub struct Algo;

impl base::SafeGenerateKey for () {
    type Settings = ();
    fn safe_generate(_: &()) -> Self {
        ()
    }
}

impl base::Algo for Algo {
    type Key = ();
    type ConstructionData = ();
    fn key_settings<'a>(
        &'a self,
    ) -> &'a <<Self as base::Algo>::Key as base::SafeGenerateKey>::Settings {
        &()
    }
    fn new(_: ()) -> Self {
        Self
    }
}

impl symm::Key for () {}
impl symm::Algo for Algo {}
impl symm::CanDecrypt for Algo {
    type Input = [u8];
    type Error = symm::EncryptError;
    type DKey = ();
    fn decrypt(&self, _: &Self::DKey, data: &Self::Input) -> Result<Vec<u8>, Self::Error> {
        Ok(data.to_vec())
    }
}
impl symm::CanEncrypt for Algo {
    type Input = [u8];
    type Error = symm::EncryptError;
    type EKey = ();
    fn encrypt(&self, _: &Self::EKey, data: &Self::Input) -> Result<Vec<u8>, Self::Error> {
        Ok(data.to_vec())
    }
}

impl asymm::Key for () {}
impl asymm::HasPublic for () {
    type PublicKey = ();
    fn public_key<'a>(&'a self) -> &'a Self::PublicKey {
        self
    }
}
impl asymm::HasPrivate for () {
    type PrivateKey = ();
    fn private_key<'a>(&'a self) -> &'a Self::PrivateKey {
        self
    }
}

impl asymm::Algo for Algo {}
impl asymm::CanDecryptPublic for Algo {
    type Error = !;
    type Input = [u8];
    type PublicKey = ();
    fn public_decrypt(_: &Self::PublicKey, data: &Self::Input) -> Result<Vec<u8>, Self::Error> {
        Ok(data.to_vec())
    }
}
impl asymm::CanEncryptPublic for Algo {
    type Error = !;
    type Input = [u8];
    type PublicKey = ();
    fn public_encrypt(_: &Self::PublicKey, data: &Self::Input) -> Result<Vec<u8>, Self::Error> {
        Ok(data.to_vec())
    }
}
impl asymm::CanDecryptPrivate for Algo {
    type Error = !;
    type Input = [u8];
    type PrivateKey = ();
    fn private_decrypt(_: &Self::PrivateKey, data: &Self::Input) -> Result<Vec<u8>, Self::Error> {
        Ok(data.to_vec())
    }
}
impl asymm::CanEncryptPrivate for Algo {
    type Error = !;
    type Input = [u8];
    type PrivateKey = ();
    fn private_encrypt(_: &Self::PrivateKey, data: &Self::Input) -> Result<Vec<u8>, Self::Error> {
        Ok(data.to_vec())
    }
}

#[cfg(test)]
mod unit_test {
    use super::Algo;

    mod key_gen {
        use crate::algo::{
            SafeGenerateKey,
            Algo as BaseAlgo,
            cipher::{
                symmetric::Algo as SymmAlgo,
                plaintext::Algo,
            },
        };
        #[test]
        fn key_generation() {
            let alg = Algo;
            let settings = alg.key_settings();
            assert_eq!(settings, &());
            let key = <Algo as BaseAlgo>::Key::safe_generate(settings);
            assert_eq!(key, ());
        }
    }
    mod symm {
        use crate::algo::{
            SafeGenerateKey,
            Algo as BaseAlgo,
            cipher::{
                symmetric::{
                    Algo as SymmAlgo,
                    CanDecrypt,
                    CanEncrypt,
                },
                plaintext::Algo,
            },
        };
        const KEY: () = ();
        #[test]
        fn encrypt() {
            const TO_ENCRYPT: &'static[u8] = b"Hello World";
            let encrypted = Algo::new(()).encrypt(&(), TO_ENCRYPT).expect("No error when encrypting.");
            assert_eq!(encrypted, TO_ENCRYPT)
        }
        #[test]
        fn decrypt() {
            const TO_DECRYPT: &'static[u8] = b"Hello World";
            let decrypted = Algo::new(()).decrypt(&(), TO_DECRYPT).expect("No error when decrypting.");
            assert_eq!(decrypted, TO_DECRYPT)
        }
    }
    mod asymm_public {
        use crate::algo::{
            SafeGenerateKey,
            Algo as BaseAlgo,
            cipher::{
                symmetric::{
                    Algo as SymmAlgo,
                    CanDecrypt,
                    CanEncrypt,
                },
                plaintext::Algo,
            },
        };
        #[test]
        fn encrypt() {
            const TO_ENCRYPT: &'static[u8] = b"Hello World";
            let encrypted = Algo::new(()).encrypt(&(), TO_ENCRYPT).expect("No error when encrypting.");
            assert_eq!(encrypted, TO_ENCRYPT)
        }
        #[test]
        fn decrypt() {
            const TO_DECRYPT: &'static[u8] = b"Hello World";
            let decrypted = Algo::new(()).decrypt(&(), TO_DECRYPT).expect("No error when decrypting.");
            assert_eq!(decrypted, TO_DECRYPT)
        }
    }
    mod asymm_private {
        use crate::algo::{
            SafeGenerateKey,
            Algo as BaseAlgo,
            cipher::{
                symmetric::{
                    Algo as SymmAlgo,
                    CanDecrypt,
                    CanEncrypt,
                },
                plaintext::Algo,
            },
        };
        #[test]
        fn encrypt() {
            const TO_ENCRYPT: &'static[u8] = b"Hello World";
            let encrypted = Algo::new(()).encrypt(&(), TO_ENCRYPT).expect("No error when encrypting.");
            assert_eq!(encrypted, TO_ENCRYPT)
        }
        #[test]
        fn decrypt() {
            const TO_DECRYPT: &'static[u8] = b"Hello World";
            let decrypted = Algo::new(()).decrypt(&(), TO_DECRYPT).expect("No error when decrypting.");
            assert_eq!(decrypted, TO_DECRYPT)
        }
    }
}
