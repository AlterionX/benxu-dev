use super::local_prelude::*;

pub(super) struct SerializedRandToken {
    pub(super) msg: Vec<u8>,
    pub(super) footer: Option<Vec<u8>>,
    randomness: Randomness,
}
pub(super) struct PrimedToken {
    msg: Vec<u8>,
    footer: Option<Vec<u8>>,
    nonce: Nonce,
    auth_key: AuthKey,
    encryption_key: EncryptionKey,
}
pub(super) struct EncryptedToken {
    msg: Vec<u8>,
    footer: Option<Vec<u8>>,
    nonce: Nonce,
    auth_key: AuthKey,
}
pub(super) struct SignedEncryptedToken {
    msg: Vec<u8>,
    footer: Option<Vec<u8>>,
    nonce: Nonce,
    signature: Vec<u8>,
}

impl SerializedRandToken {
    pub(super) fn new(msg: Vec<u8>, footer: Option<Vec<u8>>) -> Self {
        let rand = Randomness::new();
        Self {
            msg: msg,
            footer: footer,
            randomness: rand,
        }
    }
    pub(super) fn preprocess(self, key: &[u8]) -> PrimedToken {
        PrimedToken::from((self, key))
    }
}
impl From<token::SerializedData> for SerializedRandToken {
    fn from(tok: token::SerializedData) -> Self {
        SerializedRandToken::new(tok.msg, tok.footer)
    }
}
impl PrimedToken {
    pub(super) fn encrypt(self) -> Result<EncryptedToken, symm::EncryptError> {
        EncryptedToken::try_from(self)
    }
}
impl From<(SerializedRandToken, &[u8])> for PrimedToken {
    fn from((tok, key): (SerializedRandToken, &[u8])) -> Self {
        let nonce = Nonce::create_from(tok.randomness, &tok.msg);

        let (encryption_key, auth_key) = split_key(&nonce, key);

        PrimedToken {
            msg: tok.msg,
            footer: tok.footer,
            nonce: nonce,
            auth_key: auth_key,
            encryption_key: encryption_key,
        }
    }
}
impl EncryptedToken {
    pub(super) fn sign(self) -> Result<SignedEncryptedToken, Error> {
        let pre_auth_encoding = multi_part_pre_auth_encoding(&[
            HEADER.to_combined().as_slice(),
            self.nonce.as_slice(),
            self.msg.as_slice(),
            self.footer.as_ref().map_or(&[], |f| f.as_slice()),
        ])
        .map_err(|_| Error::Sign)?;
        let signing_key = <HMAC_SHA384 as A>::Key::new(&self.auth_key);
        let sig = <HMAC_SHA384 as SymmHashAlgo>::sign(&pre_auth_encoding, &signing_key);
        Ok(SignedEncryptedToken {
            msg: self.msg,
            footer: self.footer,
            signature: sig,
            nonce: self.nonce,
        })
    }
}
impl TryFrom<PrimedToken> for EncryptedToken {
    type Error = symm::EncryptError;
    fn try_from(tok: PrimedToken) -> Result<Self, Self::Error> {
        let key = <AES256_CTR as A>::Key::new(&tok.encryption_key, tok.nonce.get_crypt_nonce());
        let encrypted_msg = <AES256_CTR as symm::Algo>::encrypt(&key, tok.msg.as_slice())?;

        Ok(EncryptedToken {
            msg: encrypted_msg,
            footer: tok.footer,
            auth_key: tok.auth_key,
            nonce: tok.nonce,
        })
    }
}
impl SignedEncryptedToken {
    pub(super) fn canonicalize(self) -> token::Unpacked {
        token::Unpacked::from(self)
    }
}
impl From<SignedEncryptedToken> for token::Unpacked {
    fn from(tok: SignedEncryptedToken) -> Self {
        let body = collapse_to_vec(&[
            tok.nonce.as_slice(),
            tok.msg.as_slice(),
            tok.signature.as_slice(),
        ]);
        token::Unpacked::new(HEADER, body, tok.footer)
    }
}

#[cfg(test)]
impl SerializedRandToken {
    fn init_with_rand(msg: Vec<u8>, footer: Option<Vec<u8>>, rand: Randomness) -> Self {
        Self {
            msg: msg,
            footer: footer,
            randomness: rand,
        }
    }
}
#[cfg(test)]
mod unit_tests {
    use super::*;

    const KEY: &[u8] = b"hooray-for-pie";

    #[test]
    fn flow_no_footer_test() {
        let token = SerializedRandToken::init_with_rand(
            b"hello".to_vec(),
            Some(b"world".to_vec()),
            Randomness::new(),
        );
        // TODO validate
        let token = token.preprocess(KEY);
        // TODO validate
        let token = token.encrypt();
        let token = token.unwrap();
        // TODO validate
        let token = token.sign();
        let token = token.unwrap();
        // TODO validate
        let token = token.canonicalize();
        // TODO validate
    }
    #[test]
    fn flow_footer_test() {
        let token = SerializedRandToken::init_with_rand(
            b"hello universe".to_vec(),
            None,
            Randomness::new(),
        );
        // TODO validate
        let token = token.preprocess(KEY);
        // TODO validate
        let token = token.encrypt();
        let token = token.unwrap();
        // TODO validate
        let token = token.sign();
        let token = token.unwrap();
        // TODO validate
        let token = token.canonicalize();
        // TODO validate
    }
}
