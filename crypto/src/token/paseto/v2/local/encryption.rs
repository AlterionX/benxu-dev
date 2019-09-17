use super::local_prelude::*;

pub(super) struct PrimedToken {
    msg: Vec<u8>,
    footer: Option<Vec<u8>>,
    nonce: Nonce,
}
pub(super) struct EncryptedToken {
    msg: Vec<u8>,
    footer: Option<Vec<u8>>,
}

impl From<token::SerializedData> for PrimedToken {
    fn from(tok: token::SerializedData) -> Self {
        Self {
            nonce: Nonce::create_from(Randomness::new(), tok.msg.as_slice()),
            msg: tok.msg,
            footer: tok.footer,
        }
    }
}
impl PrimedToken {
    pub(super) fn encrypt(
        self,
        key: &<XCHACHA20_POLY1305 as A>::Key,
    ) -> Result<EncryptedToken, Error> {
        EncryptedToken::try_from((self, key)).map_err(|_| Error::Encryption)
    }
}
impl TryFrom<(PrimedToken, &<XCHACHA20_POLY1305 as A>::Key)> for EncryptedToken {
    type Error = Error;
    fn try_from(
        (tok, key): (PrimedToken, &<XCHACHA20_POLY1305 as A>::Key),
    ) -> Result<Self, Self::Error> {
        let aad = multi_part_pre_auth_encoding(&[
            HEADER.to_combined().as_slice(),
            tok.nonce.as_slice(),
            tok.footer.as_ref().map(|f| f.as_slice()).unwrap_or(b""),
        ])
        .map_err(|_| Error::Encryption)?;
        let encryption_args = ChaChaEncryptArgs {
            plaintext: tok.msg,
            aad: Some(aad),
            nonce: Some(ChaChaNonce::from_slice(tok.nonce.as_slice()).ok_or(Error::Encryption)?),
        };
        let ciphertext = <XCHACHA20_POLY1305 as CipherA>::encrypt(key, &encryption_args)?;
        Ok(Self {
            msg: collapse_to_vec(&[tok.nonce.as_slice(), ciphertext.as_slice()]),
            footer: tok.footer,
        })
    }
}
impl From<EncryptedToken> for token::Unpacked {
    fn from(tok: EncryptedToken) -> Self {
        Self::new(HEADER, tok.msg, tok.footer)
    }
}
impl EncryptedToken {
    pub(super) fn canonicalize(self) -> token::Unpacked {
        token::Unpacked::from(self)
    }
}
