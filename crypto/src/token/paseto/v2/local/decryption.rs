use super::local_prelude::*;

pub(super) struct BasicToken {
    source: Vec<u8>,
    nonce: Nonce,
    footer: Option<Vec<u8>>,
}
pub(super) struct DecryptedToken {
    msg: Vec<u8>,
    footer: Option<Vec<u8>>,
}

impl BasicToken {
    const NONCE_LENGTH: usize = 24;
    const NONCE_RANGE: RangeTo<usize> = (..24);
    const MSG_RANGE: RangeFrom<usize> = (24..);
    pub(super) fn decrypt(self, key: &ENC_KEY) -> Result<DecryptedToken, Error> {
        DecryptedToken::try_from((self, key))
    }

    fn msg(&self) -> &[u8] {
        &self.source[Self::MSG_RANGE]
    }
}
impl TryFrom<token::Unpacked> for BasicToken {
    type Error = Error;
    fn try_from(tok: token::Unpacked) -> Result<Self, Self::Error> {
        if tok.body.len() < Self::NONCE_LENGTH {
            Err(Error::Unpack)?;
        }
        let nonce = &tok.body[Self::NONCE_RANGE];
        let nonce = Nonce::recreate_nonce(nonce);
        Ok(Self {
            source: tok.body,
            nonce: nonce,
            footer: tok.footer,
        })
    }
}
impl DecryptedToken {
    pub(super) fn canonicalize(self) -> token::SerializedData {
        token::SerializedData::from(self)
    }
}
impl TryFrom<(BasicToken, &ENC_KEY)> for DecryptedToken {
    type Error = Error;
    fn try_from((tok, key): (BasicToken, &ENC_KEY)) -> Result<Self, Self::Error> {
        let aad = multi_part_pre_auth_encoding(&[
            HEADER.to_combined().as_slice(),
            tok.nonce.as_slice(),
            tok.footer.as_ref().map(|f| f.as_slice()).unwrap_or(b""),
        ])
        .map_err(|_| Error::Encryption)?;
        let decryption_args = DArgs {
            ciphertext: tok.msg().to_vec(),
            aad: Some(aad),
            nonce: ChaChaNonce::from_slice(tok.nonce.as_slice()).ok_or(Error::Decryption)?,
        };
        let ciphertext = ENC_ALGO::new(()).decrypt(key, &decryption_args)?;
        Ok(Self {
            msg: ciphertext,
            footer: tok.footer,
        })
    }
}

impl From<DecryptedToken> for token::SerializedData {
    fn from(tok: DecryptedToken) -> Self {
        Self {
            msg: tok.msg,
            footer: tok.footer,
        }
    }
}
