mod local_prelude {
    pub(super) use crate::{
        algo::hash::{
            asymmetric::{Algo as AsymmHashAlgo, KeyPair as AsymmHashKeyPair},
            rsa::pss_sha384_mgf1_65537,
        },
        token::paseto::{
            token,
            util::{collapse_to_vec, multi_part_pre_auth_encoding},
            v1::public::{error::Error, HEADER},
        },
    };
    pub(super) use boolinator::Boolinator;
    pub(super) use serde::{de::DeserializeOwned, Serialize};
    pub(super) use serde_json as json;
    pub(super) use std::{convert::TryFrom, str};
}

mod decryption;
mod encryption;
mod error;

use crate::{
    algo::Algo as A,
    token::paseto::{
        v1::public::{decryption::VerifiedToken, encryption::SignedToken, local_prelude::*},
        Protocol as ProtocolTrait,
    },
};

const VERSION: &'static str = "v1";
const PURPOSE: &'static str = "public";
const HEADER: token::Header = token::Header::new(VERSION.as_bytes(), PURPOSE.as_bytes());

impl token::SerializedData {
    // TODO replace with when Error::FailedToSign when enum variants become types
    /// Error, if Err, is always FailedToSign
    fn v1_public_sign(self, key: &pss_sha384_mgf1_65537::KeyPair) -> Result<SignedToken, Error> {
        SignedToken::try_from((self, key))
    }
}
impl token::Unpacked {
    fn v1_public_verify(
        self,
        key: &pss_sha384_mgf1_65537::KeyPair,
    ) -> Result<VerifiedToken, Error> {
        self.verify_header(HEADER).ok_or(Error::Unpacking)?;
        VerifiedToken::try_from((self, key))
    }
}

pub struct Protocol;

impl ProtocolTrait for Protocol {
    type CoreAlgo = pss_sha384_mgf1_65537::Algo;
    type Error = Error;
    fn encrypt<M: Serialize, F: Serialize, K: AsRef<<Self::CoreAlgo as A>::Key>>(
        tok: token::Data<M, F>,
        key: K,
    ) -> Result<token::Packed, Self::Error> {
        let key = key.as_ref();
        Ok(tok.serialize()?.v1_public_sign(key)?.canonicalize().pack())
    }
    fn decrypt<M: DeserializeOwned, F: DeserializeOwned, K: AsRef<<Self::CoreAlgo as A>::Key>>(
        tok: token::Packed,
        key: K,
    ) -> Result<token::Data<M, F>, Self::Error> {
        let key = key.as_ref();
        Ok(tok
            .unpack()?
            .v1_public_verify(key)?
            .canonicalize()
            .deserialize()?)
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::{algo::Key, token::paseto::Protocol as P};

    #[test]
    fn v1_public_cycle() {
        let orig = token::Data {
            msg: "hello".to_owned(),
            footer: Some("weird thing".to_owned()),
        };
        let beginning = orig.clone();
        let key = pss_sha384_mgf1_65537::KeyPair::generate(&()).unwrap();
        let encrypted_tok = Protocol::encrypt(beginning, &key).unwrap();
        let decrypted_tok: token::Data<String, String> =
            Protocol::decrypt(encrypted_tok, &key).unwrap();
        assert_eq!(orig, decrypted_tok);
    }
}
