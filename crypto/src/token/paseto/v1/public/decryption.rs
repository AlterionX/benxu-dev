use super::local_prelude::*;

// type exists for disambiguation purposes between different protocol versions
pub(super) struct VerifiedToken {
    msg: Vec<u8>,
    footer: Option<Vec<u8>>,
}

impl VerifiedToken {
    pub(super) fn canonicalize(self) -> token::SerializedData {
        token::SerializedData::from(self)
    }
}
impl TryFrom<(token::Unpacked, &pss_sha384_mgf1_65537::KeyPair)> for VerifiedToken {
    type Error = Error;
    fn try_from((tok, key): (token::Unpacked, &pss_sha384_mgf1_65537::KeyPair)) -> Result<Self, Self::Error> {
        if !tok.verify_header(HEADER) {
            return Err(Error::BadHeader);
        }

        let mut msg = tok.body;

        if msg.len() < 256 {
            return Err(Error::MalformedBody);
        }

        let sig = msg.split_off(msg.len() - 256);

        let signed = multi_part_pre_auth_encoding(&[
            HEADER.to_combined().as_slice(),
            msg.as_slice(),
            tok.footer.as_ref().map_or(&[], |f| f.as_slice()),
        ])
        .map_err(|_| Error::Signing)?;

        let is_valid_signature =
            pss_sha384_mgf1_65537::Algo::verify_public(
                signed.as_slice(),
                sig.as_slice(),
                key.public_key(),
            ).map_err(|_| Error::Verifying)?;

        if !is_valid_signature {
            return Err(Error::BadSignature).unwrap();
        }

        Ok(VerifiedToken {
            msg: msg,
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
