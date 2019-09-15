mod local_prelude {
    pub use serde::{Serialize, Deserialize, de::DeserializeOwned};
    pub use serde_json as json;
    pub use std::{ops::{Deref, RangeTo, RangeFrom}, str, convert::TryFrom};
    pub use crate::{
        BoolToResult,
        token::paseto::{
            multi_part_pre_auth_encoding,
            collapse_to_vec,
            KnownClaims,
            token,
            v2::{
                local::{
                    nonce::{Randomness, Nonce},
                    HEADER,
                    error::Error,
                },
            },
        },
        algo::{
            Algo as A,
            Key as K,
            cipher::{
                xchacha20::poly1305::{Algo as XCHACHA20_POLY1305, EncryptArgs as ChaChaEncryptArgs, DecryptArgs as ChaChaDecryptArgs, Nonce as ChaChaNonce},
                symmetric::{self as symm, Algo as CipherA},
            },
        },
        encoding::base64::{encode_no_padding as b64_encode, decode_no_padding as b64_decode},
    };
}
use self::{
    local_prelude::*,
    encryption::{PrimedToken},
    decryption::{BasicToken},
};

pub mod encryption;
pub mod decryption;

mod nonce;
pub mod error;

pub use crate::algo::cipher::xchacha20::poly1305::{Algo, Key};

const VERSION: &'static str = "v2";
const PURPOSE: &'static str = "local";
pub const HEADER: token::Header = token::Header::new(
    VERSION.as_bytes(),
    PURPOSE.as_bytes(),
);

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
    pub fn encrypt<T: Serialize, F: Serialize>(self, tok: token::Data<T, F>, key: &<XCHACHA20_POLY1305 as A>::Key) -> Result<token::Packed, error::Error> {
        Self::type_encrypt(tok, key)
    }
    fn type_encrypt<T: Serialize, F: Serialize>(tok: token::Data<T, F>, key: &<XCHACHA20_POLY1305 as A>::Key) -> Result<token::Packed, Error> {
        Ok(tok
           .serialize()?
           .v2_local_init()
           .encrypt(key)?
           .canonicalize()
           .pack()
        )
    }
    pub fn decrypt<T: DeserializeOwned, F: DeserializeOwned>(tok: token::Packed, key: &<XCHACHA20_POLY1305 as A>::Key) -> Result<token::Data<T, F>, error::Error> {
        Self::type_decrypt(tok, key)
    }
    fn type_decrypt<T: DeserializeOwned, F: DeserializeOwned>(tok: token::Packed, key: &<XCHACHA20_POLY1305 as A>::Key) -> Result<token::Data<T, F>, error::Error> {
        Ok(tok
           .unpack()?
           .v2_local_to_basic()?
           .decrypt(key)?
           .canonicalize()
           .deserialize()?
        )
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
        let key = <XCHACHA20_POLY1305 as A>::Key::generate(&());
        let encrypted_tok = Protocol::type_encrypt(beginning, &key).unwrap();
        let decrypted_tok: token::Data<String, String> = Protocol::type_decrypt(encrypted_tok, &key).unwrap();
        assert!(orig == decrypted_tok);
  }
}

