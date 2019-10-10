mod local_prelude {
    pub use crate::{
        algo::{
            cipher::{
                symmetric::{self as symm, Algo as CipherA, CanDecrypt, CanEncrypt},
                xchacha20::poly1305::{
                    Algo as ENC_ALGO, DecryptArgs as DArgs, EncryptArgs as EArgs, Key as ENC_KEY,
                    Nonce as ChaChaNonce,
                },
            },
            Algo as A, Key as K,
        },
        encoding::base64::{decode_no_padding as b64_decode, encode_no_padding as b64_encode},
        token::paseto::{
            collapse_to_vec, multi_part_pre_auth_encoding, token,
            v2::local::{
                error::Error,
                nonce::{Nonce, Randomness},
                HEADER,
            },
            KnownClaims,
        },
        BoolToResult,
    };
    pub use serde::{de::DeserializeOwned, Deserialize, Serialize};
    pub use serde_json as json;
    pub use std::{
        convert::TryFrom,
        ops::{Deref, RangeFrom, RangeTo},
        str,
    };
}
use self::{decryption::BasicToken, encryption::PrimedToken, local_prelude::*};

pub mod decryption;
pub mod encryption;

pub mod error;
mod nonce;

pub use crate::algo::cipher::xchacha20::poly1305::{Algo, Key};

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

// TODO make paseto return original on failure
pub struct Protocol;
impl Protocol {
    pub fn encrypt<T: Serialize, F: Serialize>(
        self,
        tok: token::Data<T, F>,
        key: &ENC_KEY,
    ) -> Result<token::Packed, error::Error> {
        Self::type_encrypt(tok, key)
    }
    fn type_encrypt<T: Serialize, F: Serialize>(
        tok: token::Data<T, F>,
        key: &ENC_KEY,
    ) -> Result<token::Packed, Error> {
        Ok(tok
            .serialize()?
            .v2_local_init()
            .encrypt(key)?
            .canonicalize()
            .pack())
    }
    pub fn decrypt<T: DeserializeOwned, F: DeserializeOwned>(
        tok: token::Packed,
        key: &ENC_KEY,
    ) -> Result<token::Data<T, F>, error::Error> {
        Self::type_decrypt(tok, key)
    }
    fn type_decrypt<T: DeserializeOwned, F: DeserializeOwned>(
        tok: token::Packed,
        key: &ENC_KEY,
    ) -> Result<token::Data<T, F>, error::Error> {
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
    use crate::algo::SafeGenerateKey;

    #[test]
    fn v2_local_cycle() {
        let orig = token::Data {
            msg: "hello".to_owned(),
            footer: Some("weird thing".to_owned()),
        };
        let beginning = orig.clone();
        let key = ENC_KEY::safe_generate(&());
        let encrypted_tok = Protocol::type_encrypt(beginning, &key).unwrap();
        let decrypted_tok: token::Data<String, String> =
            Protocol::type_decrypt(encrypted_tok, &key).unwrap();
        assert!(orig == decrypted_tok);
    }
}
