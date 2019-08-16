use serde::Serialize;
use serde_json as json;
use std::ops::Deref;

use super::{
    super::error,
    nonce::{Randomness, Nonce},
};

use crate::crypto::{
    token::paseto::shared::multi_part_pre_auth_encoding,
    algo::{
        Algo as A,
        Key as K,
        hash::hmac::sha384::Algo as HMAC_SHA384,
        key_deriv::hkdf::sha384::Algo as HKDF_SHA384,
        cipher::{
            aes256::ctr::Algo as AES256_CTR,
            symmetric as symm,
        },
    }
};

const HEADER: &'static str = "v1.local.";

pub trait KnownClaims {}
impl KnownClaims for String {}

pub struct NakedToken<'tok, T: Serialize + KnownClaims, F: Serialize> {
    msg: T,
    footer: Option<F>,
    key: &'tok[u8],
    rand: Randomness,
}
impl<'tok, T: Serialize + KnownClaims, F: Serialize> NakedToken<'tok, T, F> {
    pub fn new(msg: T, footer: Option<F>, key: &'tok[u8]) -> Self {
        Self {
            msg: msg,
            footer: footer,
            key: key,
            rand: Randomness::new(),
        }
    }
    fn serialize(self) -> Result<SerializedToken<'tok>, json::Error> {
        SerializedToken::from_naked_tok(self)
    }
}
#[cfg(test)]
impl<'tok, T: Serialize + KnownClaims, F: Serialize> NakedToken<'tok, T, F> {
    pub fn new_with_rand(msg: T, footer: Option<F>, key: &'tok[u8]) {
    }
}
struct SerializedToken<'ser> {
    msg: Vec<u8>,
    footer: Option<Vec<u8>>,
    key: &'ser[u8],
    randomness: Randomness,
}
impl<'ser> SerializedToken<'ser> {
    fn from_naked_tok<T: Serialize + KnownClaims, F: Serialize>(tok: NakedToken<'ser, T, F>) -> Result<Self, json::Error> {
        let msg = json::to_string(&tok.msg)?.as_bytes().to_vec();
        let footer = tok.footer.map(|f| json::to_string(&f)).transpose()?.map(|s| s.as_bytes().to_vec());
        Ok(Self {
            msg: msg,
            footer: footer,
            key: tok.key,
            randomness: tok.rand,
        })
    }
    fn preprocess(self) -> PrimedToken {
        PrimedToken::create_from(self)
    }
}
struct PrimedToken {
    msg: Vec<u8>,
    footer: Option<Vec<u8>>,
    nonce: Nonce,
    auth_key: Vec<u8>,
    encryption_key: Vec<u8>,
}
impl PrimedToken {
    fn create_keys<'a>(nonce: &'a Nonce) -> (Vec<u8>, Vec<u8>) {
        let hkdf = HKDF_SHA384::new(nonce.get_salt().to_vec(), vec![]);
        let key_deriv_key = <<HKDF_SHA384 as A>::Key as K>::generate(hkdf.key_settings());
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
        (ek, ak)
    }
    fn create_from(tok: SerializedToken) -> Self {
        let nonce = Nonce::create_from(tok.randomness, &tok.msg);
        let (encryption_key, auth_key) = Self::create_keys(&nonce);
        Self {
            msg: tok.msg,
            footer: tok.footer,
            nonce: nonce,
            auth_key: auth_key,
            encryption_key: encryption_key,
        }
    }
    fn encrypt(self) -> Result<EncryptedToken, symm::EncryptError> {
        EncryptedToken::create_from(self)
    }
}
struct EncryptedToken {
    msg: Vec<u8>,
    footer: Option<Vec<u8>>,
    auth_key: Vec<u8>,
    nonce: Nonce,
}
impl EncryptedToken {
    fn create_from(tok: PrimedToken) -> Result<Self, symm::EncryptError>  {
        let encrypted_message = <AES256_CTR as symm::Algo>::encrypt(&<AES256_CTR as A>::Key::new(tok.encryption_key.as_slice(), tok.nonce.get_crypt_nonce()), tok.msg.as_slice())?;
        Ok(Self {
            msg: encrypted_message,
            footer: tok.footer,
            auth_key: tok.auth_key,
            nonce: tok.nonce,
        })
    }
    fn sign(self) -> Result<SignedEncryptedToken, error::Error> {
        SignedEncryptedToken::create_from(self)
    }
}
struct SignedEncryptedToken {
    msg: Vec<u8>,
    footer: Option<Vec<u8>>,
    signature: Vec<u8>,
    nonce: Nonce,
}
impl SignedEncryptedToken {
    fn create_from(tok: EncryptedToken) -> Result<Self, error::Error> {
        let default_vec = vec![];
        let message_parts = vec![
            HEADER.as_bytes(),
            tok.nonce.as_slice(),
            tok.msg.as_slice(),
            tok.footer.as_ref().unwrap_or(&default_vec).as_slice(),
        ];
        let pre_auth_encoding = multi_part_pre_auth_encoding(message_parts.as_slice()).map_err(|s| error::Error::Signing(s))?;
        let signing_key = <HMAC_SHA384 as A>::Key::new(&tok.auth_key);
        let sig = <HMAC_SHA384 as crate::crypto::algo::hash::symmetric::Algo>::sign(&pre_auth_encoding, &signing_key);
        Ok(Self {
            msg: tok.msg,
            footer: tok.footer,
            signature: sig,
            nonce: tok.nonce,
        })
    }
    fn pack(self) -> PasetoToken {
        PasetoToken::create_from(self)
    }
}
pub struct PasetoToken(Vec<u8>);
impl PasetoToken {
    pub fn new(buffer: Vec<u8>) -> Self {
        Self(buffer)
    }
    fn create_from(tok: SignedEncryptedToken) -> Self {
        Self(
            [
                HEADER.as_bytes(),
                tok.nonce.as_slice(),
                tok.msg.as_slice(),
                tok.signature.as_slice(),
                tok.footer.as_ref().map_or(&[], |_| &[b'.']),
                tok.footer.as_ref().map_or(&[], |f| f.as_slice()),
            ].iter()
            .map(|s| s.iter())
            .flatten()
            .map(|b| *b)
            .collect()
        )
    }
}
impl Deref for PasetoToken {
    type Target = [u8];
    fn deref<'a>(&'a self) -> &'a Self::Target {
        &self.0
    }
}



pub struct Token;
impl Token {
    pub fn encrypt<T: Serialize + KnownClaims, F: Serialize>(self, tok: NakedToken<T, F>) -> Result<PasetoToken, error::Error> {
        Self::type_encrypt(tok)
    }
    fn type_encrypt<T: Serialize + KnownClaims, F: Serialize>(tok: NakedToken<T, F>) -> Result<PasetoToken, error::Error> {
        Ok(tok
            .serialize().map_err(|e| error::Error::Serialization(e))?
            .preprocess()
            .encrypt().map_err(|e| error::Error::SymmEncryption(e))?
            .sign()?
            .pack()
        )
    }
}


#[cfg(test)]
mod unit_tests {
  use super::*;

  #[test]
  fn v1_local_encrypt() {
      let tok = NakedToken::<String, ()>::new("Hello".to_owned(), None, "WOW".as_bytes());
      Token.encrypt(tok).map_err(|_| ()).unwrap();
  }
  #[test]
  fn v1_local_decrypt() {
  }
}


