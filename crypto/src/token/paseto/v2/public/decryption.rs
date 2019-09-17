use super::local_prelude::*;

pub(super) struct SeparatedToken {
    source: Vec<u8>,
    footer: Option<Vec<u8>>,
}
pub(super) struct VerifiedToken {
    msg: Vec<u8>,
    footer: Option<Vec<u8>>,
}

impl SeparatedToken {
    fn sig_msg_boundary(&self) -> usize {
        self.source.len() - 64
    }
    fn sig(&self) -> &[u8] {
        &self.source[self.sig_msg_boundary()..]
    }
    fn msg(&self) -> &[u8] {
        &self.source[..self.sig_msg_boundary()]
    }

    pub(super) fn verify(self, key: &<ED25519 as A>::Key) -> Result<VerifiedToken, Error> {
        VerifiedToken::try_from((self, key))
    }
}
impl From<token::Unpacked> for SeparatedToken {
    fn from(tok: token::Unpacked) -> Self {
        Self {
            source: tok.body,
            footer: tok.footer,
        }
    }
}

impl VerifiedToken {
    pub(super) fn canonicalize(self) -> token::SerializedData {
        token::SerializedData::from(self)
    }
}
impl TryFrom<(SeparatedToken, &<ED25519 as A>::Key)> for VerifiedToken {
    type Error = Error;
    fn try_from((tok, key): (SeparatedToken, &<ED25519 as A>::Key)) -> Result<Self, Self::Error> {
        let signed_plaintext = multi_part_pre_auth_encoding(&[
            HEADER.to_combined().as_slice(),
            tok.msg(),
            tok.footer.as_ref().map_or(b"", |f| f.as_slice()),
        ])
        .map_err(|_| Error::Signing)?;
        <ED25519 as HashA>::verify_public(signed_plaintext.as_slice(), tok.sig(), key.public_key())
            .map_err(|_| Error::Signing)?
            .ok_or(Error::BadSignature)?;
        Ok(Self {
            msg: tok.msg().to_vec(),
            footer: tok.footer,
        })
    }
}

impl From<VerifiedToken> for token::SerializedData {
    fn from(tok: VerifiedToken) -> Self {
        Self {
            msg: tok.msg,
            footer: tok.footer,
        }
    }
}
