//! Implementation of the PASETO version 1, opaque token.

mod local_prelude {
    pub(super) use crate::{
        algo::{
            cipher::{
                aes256::ctr,
                symmetric::{self as symm, CanDecrypt, CanEncrypt},
            },
            hash::{hmac::sha384::Algo as HMAC_SHA384, symmetric::Algo as SymmHashAlgo},
            key_deriv::hkdf::sha384::Algo as HKDF_SHA384,
            Algo as A,
        },
        token::paseto::{
            token,
            util::{collapse_to_vec, multi_part_pre_auth_encoding},
            v1::{
                local::{error::Error, split_key, AuthKey, EncryptionKey, HEADER},
                nonce::{Nonce, Randomness},
            },
        },
    };
    pub(super) use boolinator::Boolinator;
    pub(super) use serde::{de::DeserializeOwned, Serialize};
    pub(super) use std::{convert::TryFrom, ops::Deref, str};
}

mod decryption;
mod encryption;

mod error;

use crate::{
    algo::{Algo as A, Key as K, SafeGenerateKey},
    token::paseto::{
        v1::local::{decryption::BasicToken, encryption::SerializedRandToken, local_prelude::*},
        Protocol as ProtocolTrait,
    },
};

/// The version string for this protocol.
pub const VERSION: &'static str = "v1";
/// The purpose string for this protocol.
pub const PURPOSE: &'static str = "local";
/// The agglomerated [`Header`] for this protocol.
pub const HEADER: token::Header = token::Header::new(VERSION.as_bytes(), PURPOSE.as_bytes());

/// A newtype struct differentiating the auth and encryption keys.
pub struct AuthKey(Vec<u8>);
impl Deref for AuthKey {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}
/// A newtype struct differentiating the auth and encryption keys.
pub struct EncryptionKey(Vec<u8>);
impl Deref for EncryptionKey {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

/// Takes the nonce and a static key, creating the auth and encrpytion keys.
pub fn split_key(nonce: &Nonce, key: &[u8]) -> (EncryptionKey, AuthKey) {
    let hkdf = HKDF_SHA384::new((nonce.get_salt().to_vec(), vec![key.to_vec()]));
    let key_deriv_key =
        <<HKDF_SHA384 as A>::Key as SafeGenerateKey>::safe_generate(hkdf.key_settings());
    let mut keys = hkdf.generate(
        key_deriv_key,
        &[
            "paesto-encryption-key".as_bytes(),
            "paesto-auth-key-for".as_bytes(),
        ],
        32,
    );
    let ak = keys.pop().unwrap();
    let ek = keys.pop().unwrap();
    (EncryptionKey(ek), AuthKey(ak))
}

impl token::SerializedData {
    /// Converts a [`SerializedData`] to the [`SerializedRandToken`] needed for encryption.
    fn v1_local_init(self) -> SerializedRandToken {
        SerializedRandToken::from(self)
    }
}
impl token::Unpacked {
    /// Converts a [`Unpacked`] token to the [`BasicToken`] needed for decryption.
    fn v1_local_to_basic(self) -> Result<BasicToken, error::Error> {
        self.verify_header(HEADER).ok_or(Error::Unpack)?;
        Ok(BasicToken::create_from(self))
    }
}

#[derive(Clone)]
pub struct CombinedKey(Vec<u8>);

impl SafeGenerateKey for CombinedKey {
    type Settings = usize;
    fn safe_generate(len: &Self::Settings) -> Self {
        use rand::{rngs::OsRng, RngCore};
        let mut buffer = vec![0; *len];
        OsRng.fill_bytes(buffer.as_mut_slice());
        CombinedKey(buffer)
    }
}

impl AsRef<CombinedKey> for &CombinedKey {
    fn as_ref(&self) -> &CombinedKey {
        self
    }
}

pub struct CombinedAlgo(usize);

impl A for CombinedAlgo {
    type Key = CombinedKey;
    fn key_settings(&self) -> &<Self::Key as K>::Settings {
        &self.0
    }
    fn new(_: Self::ConstructionData) -> Self {
        Self(32)
    }
}

pub struct Protocol;

impl ProtocolTrait for Protocol {
    type CoreAlgo = CombinedAlgo;
    type Error = Error;
    fn encrypt<M: Serialize, F: Serialize, K: AsRef<<Self::CoreAlgo as A>::Key>>(
        tok: token::Data<M, F>,
        key: K,
    ) -> Result<token::Packed, Self::Error> {
        let key = key.as_ref().0.as_slice();
        Ok(tok
            .serialize()?
            .v1_local_init()
            .preprocess(key)
            .encrypt()?
            .sign()?
            .canonicalize()
            .pack())
    }
    fn decrypt<M: DeserializeOwned, F: DeserializeOwned, K: AsRef<<Self::CoreAlgo as A>::Key>>(
        tok: token::Packed,
        key: K,
    ) -> Result<token::Data<M, F>, Self::Error> {
        let key = key.as_ref().0.as_slice();
        Ok(tok
            .unpack()?
            .v1_local_to_basic()?
            .prime(key)
            .verify()?
            .decrypt()?
            .deserialize()?)
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::token::paseto::Protocol as P;

    #[test]
    fn v1_local_cycle() {
        let orig = token::Data {
            msg: "hello".to_owned(),
            footer: Some("weird thing".to_owned()),
        };
        let beginning = orig.clone();
        let key = CombinedKey(b"some arbitrary key".to_vec());
        let encrypted_tok = Protocol::encrypt(beginning, &key).unwrap();
        let decrypted_tok: token::Data<String, String> =
            Protocol::decrypt(encrypted_tok, &key).unwrap();
        assert!(orig == decrypted_tok);
    }
}
