use super::local_prelude::*;

pub(super) struct SignedToken {
    sig: Vec<u8>,
    msg: Vec<u8>,
    footer: Option<Vec<u8>>,
}

impl TryFrom<(token::SerializedData, &<ED25519 as A>::Key)> for SignedToken {
    type Error = Error;
    fn try_from((tok, key): (token::SerializedData, &<ED25519 as A>::Key)) -> Result<Self, Self::Error> {
        let sign_target = multi_part_pre_auth_encoding(&[
            HEADER.to_combined().as_slice(),
            tok.msg.as_slice(),
            tok.footer.as_ref().map_or(b"", |f| f.as_slice()),
        ]).map_err(|_| Error::Signing)?;
        let sig = <ED25519 as HashA>::sign_private(sign_target.as_slice(), key.private_key()).map_err(|_| Error::Signing)?;
        Ok(Self {
            sig: sig,
            msg: tok.msg,
            footer: tok.footer,
        })
    }
}
impl SignedToken {
    pub(super) fn canonicalize(self) -> token::Unpacked {
        token::Unpacked::from(self)
    }
}
impl From<SignedToken> for token::Unpacked {
    fn from(tok: SignedToken) -> Self {
        let body = collapse_to_vec(&[
            tok.msg.as_slice(),
            tok.sig.as_slice(),
        ]);
        Self::new(HEADER, body, tok.footer)
    }
}

