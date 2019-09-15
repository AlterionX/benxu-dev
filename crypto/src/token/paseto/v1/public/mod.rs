mod local_prelude {
    pub use serde::{Serialize, Deserialize, de::DeserializeOwned};
    pub use serde_json as json;
    pub use std::{ops::Deref, str, convert::TryFrom};
    pub use crate::{
        BoolToResult,
        token::paseto::{
            multi_part_pre_auth_encoding,
            collapse_to_vec,
            KnownClaims,
            token,
            v1::{
                nonce::{Randomness, Nonce},
                public::{
                    error::Error,
                    HEADER,
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
                    Algo as AsymmHashAlgo,
                },
                rsa::pss_sha384_mgf1_65537::{
                    KeyPair as RSAKey,
                    Algo as RSA,
                },
            },
            key_deriv::hkdf::sha384::Algo as HKDF_SHA384,
            cipher::{
                aes256::ctr::Algo as AES256_CTR,
                symmetric as symm,
            },
        },
        encoding::base64::{encode_no_padding as b64_encode, decode_no_padding as b64_decode},
    };
}
use self::{
    local_prelude::*,
    decryption::{VerifiedToken},
    encryption::{SignedToken},
};

mod encryption;
mod decryption;
mod error;

pub const VERSION: &'static str = "v1";
pub const PURPOSE: &'static str = "public";
pub const HEADER: token::Header = token::Header::new(VERSION.as_bytes(), PURPOSE.as_bytes());

impl token::SerializedData {
    // TODO replace with when Error::FailedToSign when enum variants become types
    /// Error, if Err, is always FailedToSign
    fn v1_public_sign(self, key: &RSAKey) -> Result<SignedToken, Error> {
        SignedToken::try_from((self, key))
    }
}
impl token::Unpacked {
    fn v1_public_verify(self, key: &RSAKey) -> Result<VerifiedToken, Error> {
        self.verify_header(HEADER).ok_or(Error::Unpacking)?;
        VerifiedToken::try_from((self, key))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Protocol;
impl Protocol {
    pub fn encrypt<T, F>(self, tok: token::Data<T, F>, key: &RSAKey) -> Result<token::Packed, error::Error>
        where
            T: Serialize,
            F: Serialize,
    {
        Self::type_encrypt(tok, key)
    }
    fn type_encrypt<T, F>(tok: token::Data<T, F>, private_key: &RSAKey) -> Result<token::Packed, error::Error>
        where
            T: Serialize,
            F: Serialize,
    {
        Ok(tok
            .serialize()?
            .v1_public_sign(private_key)?
            .canonicalize()
            .pack()
        )
    }
    pub fn decrypt<T, F>(tok: token::Packed, key: &RSAKey) -> Result<token::Data<T, F>, error::Error>
        where
            T: DeserializeOwned,
            F: DeserializeOwned,
    {
        Self::type_decrypt(tok, key)
    }
    fn type_decrypt<T, F>(tok: token::Packed, key: &RSAKey) -> Result<token::Data<T, F>, error::Error> where
            T: DeserializeOwned,
            F: DeserializeOwned,
    {
        Ok(tok
           .unpack()?
           .v1_public_verify(key)?
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
    fn v1_public_cycle() {
        let orig = token::Data {
            msg: "hello".to_owned(),
            footer: Some("weird thing".to_owned()),
        };
        let beginning = orig.clone();
        let key = RSAKey::generate_with_err(&()).unwrap();
        let encrypted_tok = Protocol::type_encrypt(beginning, &key).unwrap();
        let decrypted_tok: token::Data<String, String> = Protocol::type_decrypt(encrypted_tok, &key).unwrap();
        assert!(orig == decrypted_tok);
    }
}

