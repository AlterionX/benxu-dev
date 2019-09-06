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
                v2::{
                    public::{
                        HEADER,
                        error::Error,
                    },
                },
            },
            algo::{
                Algo as A,
                Key as K,
                SafeGenerateKey,
                hash::{
                    asymmetric::{
                        KeyPair as AsymmHashKeyPair,
                        Algo as HashA,
                    },
                    ecc::ed25519::Algo as ED25519,
                },
            }
        },
        encoding::base64::{encode_no_padding as b64_encode, decode_no_padding as b64_decode},
    };
}
use self::{
    local_prelude::*,
    encryption::{SignedToken},
    decryption::{SeparatedToken},
};

mod error;
mod encryption;
mod decryption;

const VERSION: &'static str = "v2";
const PURPOSE: &'static str = "public";
pub const HEADER: token::Header = token::Header::new(
    VERSION.as_bytes(),
    PURPOSE.as_bytes(),
);


impl token::SerializedData {
    fn v2_public_sign(self, key: &<ED25519 as A>::Key) -> Result<SignedToken, Error> {
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
impl Protocol {
    pub fn encrypt<T: Serialize + KnownClaims, F: Serialize>(self, tok: token::Data<T, F>, key: &<ED25519 as A>::Key) -> Result<token::Packed, Error> {
        Self::type_encrypt(tok, key)
    }
    fn type_encrypt<T: Serialize + KnownClaims, F: Serialize>(tok: token::Data<T, F>, key: &<ED25519 as A>::Key) -> Result<token::Packed, Error> {
        Ok(tok
           .serialize()?
           .v2_public_sign(key)?
           .canonicalize()
           .pack()
        )
    }
    pub fn decrypt<T: DeserializeOwned + KnownClaims, F: DeserializeOwned>(tok: token::Packed, key: &<ED25519 as A>::Key) -> Result<token::Data<T, F>, Error> {
        Self::type_decrypt(tok, key)
    }
    fn type_decrypt<T: DeserializeOwned + KnownClaims, F: DeserializeOwned>(tok: token::Packed, key: &<ED25519 as A>::Key) -> Result<token::Data<T, F>, Error> {
        Ok(tok
           .unpack()?
           .v2_public_separate()?
           .verify(key)?
           .canonicalize()
           .deserialize()?
        )
    }
}

#[cfg(test)]
mod unit_tests {
  use super::*;
  use crate::crypto::algo::SafeGenerateKey;

  #[test]
  fn v2_public_cycle() {
        let orig = token::Data {
            msg: "hello".to_owned(),
            footer: Some("weird thing".to_owned()),
        };
        let beginning = orig.clone();
        let key = <ED25519 as A>::Key::generate(&None);
        let encrypted_tok = Protocol::type_encrypt(beginning, &key).unwrap();
        let decrypted_tok: token::Data<String, String> = Protocol::type_decrypt(encrypted_tok, &key).unwrap();
        assert!(orig == decrypted_tok);
  }
}

