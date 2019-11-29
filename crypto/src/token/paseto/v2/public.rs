mod local_prelude {
    pub(super) use crate::{
        algo::{
            hash::{
                asymmetric::{Algo as HashA, KeyPair as AsymmHashKeyPair},
                ecc::ed25519,
            },
            Algo as A,
        },
        token::paseto::{
            util::{collapse_to_vec, multi_part_pre_auth_encoding}, token,
            v2::public::{error::Error, HEADER},
        },
    };
    pub(super) use boolinator::Boolinator;
    pub(super) use serde::{de::DeserializeOwned, Serialize};
    pub(super) use std::{convert::TryFrom, str};
}

mod decryption;
mod encryption;

mod error;

use crate::{
    token::paseto::{
        Protocol as ProtocolTrait,
        v2::public::{
            decryption::SeparatedToken,
            encryption::SignedToken,
            local_prelude::*,
        },
    },
};

const VERSION: &'static str = "v2";
const PURPOSE: &'static str = "public";
pub const HEADER: token::Header = token::Header::new(VERSION.as_bytes(), PURPOSE.as_bytes());

impl token::SerializedData {
    fn v2_public_sign(self, key: &<ed25519::Algo as A>::Key) -> Result<SignedToken, Error> {
        SignedToken::try_from((self, key))
    }
}
impl token::Unpacked {
    fn v2_public_separate(self) -> Result<SeparatedToken, Error> {
        self.verify_header(HEADER).ok_or(Error::Signing)?;
        Ok(SeparatedToken::from(self))
    }
}

pub struct Protocol;

impl ProtocolTrait for Protocol {
    type CoreAlgo = ed25519::Algo;
    type Error = Error;
    fn encrypt<M: Serialize, F: Serialize, K: AsRef<<Self::CoreAlgo as A>::Key>>(
        tok: token::Data<M, F>,
        key: K,
    ) -> Result<token::Packed, Self::Error> {
        let key = key.as_ref();
        Ok(tok.serialize()?.v2_public_sign(key)?.canonicalize().pack())
    }
    fn decrypt<M: DeserializeOwned, F: DeserializeOwned, K: AsRef<<Self::CoreAlgo as A>::Key>>(
        tok: token::Packed,
        key: K,
    ) -> Result<token::Data<M, F>, Self::Error> {
        let key = key.as_ref();
        Ok(tok
            .unpack()?
            .v2_public_separate()?
            .verify(key)?
            .canonicalize()
            .deserialize()?)
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::{
        algo::SafeGenerateKey,
        token::paseto::Protocol as P,
    };

    #[test]
    fn v2_public_cycle() {
        let orig = token::Data {
            msg: "hello".to_owned(),
            footer: Some("weird thing".to_owned()),
        };
        let beginning = orig.clone();
        let key = <ed25519::Algo as A>::Key::safe_generate(&None);
        let encrypted_tok = Protocol::encrypt(beginning, &key).unwrap();
        let decrypted_tok: token::Data<String, String> =
            Protocol::decrypt(encrypted_tok, &key).unwrap();
        assert!(orig == decrypted_tok);
    }
}
