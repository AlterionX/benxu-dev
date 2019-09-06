mod local_prelude {
    pub use serde::{Serialize, Deserialize, de::DeserializeOwned};
    pub use serde_json as json;
    pub use std::{ops::Deref, str, convert::TryFrom};
    pub use crate::{
        crypto::{
            BoolToResult,
            token::paseto::{
                multi_part_pre_auth_encoding,
                collapse_to_vec,
                KnownClaims,
                token,
                v1::{
                    nonce::{Randomness, Nonce},
                    local::{
                        HEADER,
                        split_key,
                        AuthKey,
                        EncryptionKey,
                        error::Error,
                    },
                },
            },
            algo::{
                Algo as A,
                Key as K,
                SafeGenerateKey,
                hash::{
                    symmetric::{
                        Key as SymmHashKey,
                        Algo as SymmHashAlgo,
                    },
                    hmac::sha384::Algo as HMAC_SHA384,
                },
                key_deriv::hkdf::sha384::Algo as HKDF_SHA384,
                cipher::{
                    aes256::ctr::Algo as AES256_CTR,
                    symmetric as symm,
                },
            }
        },
        encoding::base64::{encode_no_padding as b64_encode, decode_no_padding as b64_decode},
    };
}
use self::{
    local_prelude::*,
    encryption::{SerializedRandToken},
    decryption::{BasicToken},
};

mod encryption;
mod decryption;

pub mod error;

pub const VERSION: &'static str = "v1";
pub const PURPOSE: &'static str = "local";
pub const HEADER: token::Header = token::Header::new(
    VERSION.as_bytes(),
    PURPOSE.as_bytes(),
);

pub struct AuthKey(Vec<u8>);
impl Deref for AuthKey {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}
pub struct EncryptionKey(Vec<u8>);
impl Deref for EncryptionKey {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

pub fn split_key(nonce: &Nonce, key: &[u8]) -> (EncryptionKey, AuthKey) {
    let hkdf = HKDF_SHA384::new(nonce.get_salt().to_vec(), vec![key.to_vec()]);
    let key_deriv_key = <<HKDF_SHA384 as A>::Key as SafeGenerateKey>::generate(hkdf.key_settings());
    let mut keys = HKDF_SHA384::generate(
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
    fn v1_local_init(self) -> SerializedRandToken {
        SerializedRandToken::from(self)
    }
}
impl token::Unpacked {
    fn v1_local_to_basic(self) -> Result<BasicToken, error::Error> {
        self.verify_header(HEADER).ok_or(Error::Unpack)?;
        BasicToken::create_from(self)
    }
}

pub struct Protocol;
impl Protocol {
    pub fn encrypt<T: Serialize + KnownClaims, F: Serialize>(self, tok: token::Data<T, F>, key: &[u8]) -> Result<token::Packed, error::Error> {
        Self::type_encrypt(tok, key)
    }
    fn type_encrypt<T: Serialize + KnownClaims, F: Serialize>(tok: token::Data<T, F>, key: &[u8]) -> Result<token::Packed, Error> {
        Ok(tok
           .serialize()?
           .v1_local_init()
           .preprocess(key)
           .encrypt()?
           .sign()?
           .canonicalize()
           .pack()
        )
    }
    pub fn decrypt<T: DeserializeOwned + KnownClaims, F: DeserializeOwned>(tok: token::Packed, key: &[u8]) -> Result<token::Data<T, F>, error::Error> {
        Self::type_decrypt(tok, key)
    }
    fn type_decrypt<T: DeserializeOwned + KnownClaims, F: DeserializeOwned>(tok: token::Packed, key: &[u8]) -> Result<token::Data<T, F>, error::Error> {
        Ok(tok
           .unpack()?
           .v1_local_to_basic()?
           .prime(key)
           .verify()?
           .decrypt()?
           .deserialize()?
        )
    }
}

#[cfg(test)]
mod unit_tests {
  use super::*;

  #[test]
  fn v1_local_cycle() {
        let orig = token::Data {
            msg: "hello".to_owned(),
            footer: Some("weird thing".to_owned()),
        };
        let beginning = orig.clone();
        let key = b"some arbitrary key";
        let encrypted_tok = Protocol::type_encrypt(beginning, &key[..]).unwrap();
        let decrypted_tok: token::Data<String, String> = Protocol::type_decrypt(encrypted_tok, &key[..]).unwrap();
        assert!(orig == decrypted_tok);
  }
}

