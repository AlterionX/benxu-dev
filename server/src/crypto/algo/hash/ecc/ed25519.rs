use std::{
    option::NoneError,
};
use sodiumoxide::crypto::sign::ed25519::{Signature, Seed, SecretKey, PublicKey, gen_keypair, keypair_from_seed, sign_detached as ed25519_sign, verify_detached as ed25519_verify};

use crate::crypto::algo as base;
use base::hash::asymmetric as asymm;


pub struct KeyPair {
    private: Option<SecretKey>,
    public: PublicKey,
}
impl KeyPair {
    fn create_from(sk: Option<SecretKey>, pk: PublicKey) -> Self {
        Self {
            private: sk,
            public: pk,
        }
    }
}
impl Clone for KeyPair {
    fn clone(&self) -> Self {
        // source should be valid if KeyPair had been successfully created, especially since it
        // worked last time.
        Self::create_from(self.private.clone(), self.public.clone())
    }
}
impl base::SafeGenerateKey for KeyPair {
    type Settings = Option<Seed>;
    fn generate(seed: &Self::Settings) -> Self {
        // rust openssl as of 0.10.24 automatically uses required 65537 for exponent
        let (public, private) = if let Some(seed) = seed.as_ref() {
            keypair_from_seed(seed) // TODO probably a bad idea
        } else {
            gen_keypair()
        };
        Self::create_from(Some(private), public)
    }
}
impl asymm::KeyPair for KeyPair {
    type Private = SecretKey;
    type Public = PublicKey;
    fn public_key(&self) -> &Self::Public {
        &self.public
    }
    fn private_key<'a>(&'a self) -> Option<&'a Self::Private> {
        self.private.as_ref()
    }
}

#[derive(Debug)]
pub enum AlgoError {
    DoesNotHavePrivateKey,
    MismatchedSignatureLength,
}
impl From<NoneError> for AlgoError {
    fn from(_: NoneError) -> Self {
        Self::DoesNotHavePrivateKey
    }
}

pub struct Algo {
}
impl base::Algo for Algo {
    type Key = KeyPair;
    fn key_settings<'a>(&'a self) -> &'a <<Self as base::Algo>::Key as base::Key>::Settings {
        &None
    }
}
impl asymm::Algo for Algo {
    type SigningError = AlgoError;
    type VerifyError = AlgoError;

    /// unimplemented
    fn sign_public(
        _msg: &[u8],
        _key: &<Self::Key as asymm::KeyPair>::Public,
    ) -> Result<Vec<u8>, Self::SigningError> {
        unimplemented!("Unimplemented by ring");
    }
    fn verify_public(
        msg: &[u8],
        signature: &[u8],
        key: &<Self::Key as asymm::KeyPair>::Public,
    ) -> Result<bool, Self::VerifyError> {
        let signature = Signature::from_slice(signature).ok_or(Self::VerifyError::MismatchedSignatureLength)?;
        Ok(ed25519_verify(&signature, msg, key))
    }
    fn sign_private(
        msg: &[u8],
        key: Option<&<Self::Key as asymm::KeyPair>::Private>,
    ) -> Result<Vec<u8>, Self::SigningError> {
        Ok(ed25519_sign(msg, key.as_ref()?).as_ref().to_vec())
    }
    /// unimplemented
    fn verify_private(
        _msg: &[u8],
        _signature: &[u8],
        _key: Option<&<Self::Key as asymm::KeyPair>::Private>,
    ) -> Result<bool, Self::VerifyError> {
        unimplemented!("Unimplemented by ring");
    }
}

