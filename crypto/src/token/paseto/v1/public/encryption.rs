use super::local_prelude::*;

pub(super) struct SignedToken {
    msg: Vec<u8>,
    footer: Option<Vec<u8>>,
    signature: Vec<u8>,
}

impl SignedToken {
    pub(super) fn canonicalize(self) -> token::Unpacked {
        token::Unpacked::from(self)
    }
}
impl TryFrom<(token::SerializedData, &pss_sha384_mgf1_65537::KeyPair)> for SignedToken {
    type Error = Error;
    fn try_from(
        (tok, key): (token::SerializedData, &pss_sha384_mgf1_65537::KeyPair),
    ) -> Result<Self, Self::Error> {
        let to_sign = multi_part_pre_auth_encoding(&[
            HEADER.to_combined().as_slice(),
            tok.msg.as_slice(),
            tok.footer.as_ref().map_or(&[], |f| f.as_slice()),
        ])
        .map_err(|_| Error::Signing)?;

        let signature =
            pss_sha384_mgf1_65537::Algo::sign_private(to_sign.as_slice(), key.private_key())
                .map_err(|_| Error::Signing)?;

        Ok(SignedToken {
            msg: tok.msg,
            footer: tok.footer,
            signature: signature,
        })
    }
}
impl From<SignedToken> for token::Unpacked {
    fn from(tok: SignedToken) -> Self {
        let msg_with_sig = collapse_to_vec(&[tok.msg.as_slice(), tok.signature.as_slice()]);
        token::Unpacked::new(HEADER, msg_with_sig, tok.footer)
    }
}
