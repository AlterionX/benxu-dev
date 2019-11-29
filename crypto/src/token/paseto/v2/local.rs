mod local_prelude {
    pub(super) use crate::{
        algo::{
            cipher::{
                symmetric::{CanDecrypt, CanEncrypt},
                xchacha20::poly1305,
            },
            Algo as A,
        },
        token::paseto::{
            util::{
                collapse_to_vec,
                multi_part_pre_auth_encoding,
            },
            token,
            v2::local::{
                error::Error,
                nonce::{
                    Nonce,
                    Randomness,
                },
                HEADER,
            },
        },
    };
    pub(super) use boolinator::Boolinator;
    pub(super) use serde::{de::DeserializeOwned, Serialize};
    pub(super) use std::{
        convert::TryFrom,
        ops::{RangeFrom, RangeTo},
        str,
    };
}

mod decryption;
mod encryption;

pub mod error;
mod nonce;

use crate::token::paseto::{
    Protocol as ProtocolTrait,
    v2::local::{
        decryption::BasicToken,
        encryption::PrimedToken,
        local_prelude::*
    },
};

const VERSION: &'static str = "v2";
const PURPOSE: &'static str = "local";
pub const HEADER: token::Header = token::Header::new(VERSION.as_bytes(), PURPOSE.as_bytes());

impl token::SerializedData {
    fn v2_local_init(self) -> PrimedToken {
        PrimedToken::from(self)
    }
}
impl token::Unpacked {
    fn v2_local_to_basic(self) -> Result<BasicToken, Error> {
        self.verify_header(HEADER).ok_or(Error::Unpack)?;
        Ok(BasicToken::try_from(self).unwrap())
    }
}

pub struct Protocol;

impl ProtocolTrait for Protocol {
    type CoreAlgo = poly1305::Algo;
    type Error = Error;
    fn encrypt<M: Serialize, F: Serialize, K: AsRef<<Self::CoreAlgo as A>::Key>>(
        tok: token::Data<M, F>,
        key: K,
    ) -> Result<token::Packed, Self::Error> {
        let key = key.as_ref();
        Ok(tok
            .serialize()?
            .v2_local_init()
            .encrypt(key)?
            .canonicalize()
            .pack())
    }
    fn decrypt<M: DeserializeOwned, F: DeserializeOwned, K: AsRef<<Self::CoreAlgo as A>::Key>>(
        tok: token::Packed,
        key: K,
    ) -> Result<token::Data<M, F>, Self::Error> {
        let key = key.as_ref();
        Ok(tok
            .unpack()?
            .v2_local_to_basic()?
            .decrypt(key)?
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
    fn v2_local_cycle() {
        let orig = token::Data {
            msg: "hello".to_owned(),
            footer: Some("weird thing".to_owned()),
        };
        let beginning = orig.clone();
        let key = poly1305::Key::safe_generate(&());
        let encrypted_tok = Protocol::encrypt(beginning, &key).unwrap();
        let decrypted_tok: token::Data<String, String> =
            Protocol::decrypt(encrypted_tok, &key).unwrap();
        assert!(orig == decrypted_tok);
    }
}
