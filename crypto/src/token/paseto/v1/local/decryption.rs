use super::local_prelude::*;

pub(super) struct BasicToken {
    buffer: Vec<u8>,
    // idx of separation between signature and nonce
    message_boundary: usize,
    footer: Option<Vec<u8>>,
    nonce: Nonce,
}
pub(super) struct PrimedToken {
    buffer: Vec<u8>,
    // idx of separation between signature and nonce
    message_boundary: usize,
    footer: Option<Vec<u8>>,
    nonce: Nonce,
    auth_key: AuthKey,
    encryption_key: EncryptionKey,
}
pub(super) struct VerifiedToken {
    encrypted_message: Vec<u8>,
    footer: Option<Vec<u8>>,
    encryption_key: EncryptionKey,
    nonce: Nonce,
}

impl BasicToken {
    pub(super) fn create_from(tok: token::Unpacked) -> Result<Self, Error> {
        let nonce = Nonce::recreate_nonce(&tok.body[0..32]);
        let body_end_idx = tok.body.len() - 48;
        Ok(Self {
            buffer: tok.body,
            message_boundary: body_end_idx,
            nonce: nonce,
            footer: tok.footer,
        })
    }

    fn message(&self) -> &[u8] {
        &self.buffer[32..self.message_boundary]
    }
    fn signature(&self) -> &[u8] {
        &self.buffer[self.message_boundary..]
    }

    pub(super) fn prime(self, key: &[u8]) -> PrimedToken {
        let (ek, ak) = split_key(&self.nonce, key);
        PrimedToken::new(
            self.buffer,
            self.message_boundary,
            self.footer,
            self.nonce,
            ak,
            ek,
        )
    }
}
impl PrimedToken {
    fn new(
        buffer: Vec<u8>,
        msg_boundary: usize,
        footer: Option<Vec<u8>>,
        nonce: Nonce,
        auth_key: AuthKey,
        encryption_key: EncryptionKey,
    ) -> Self {
        Self {
            buffer: buffer,
            message_boundary: msg_boundary,
            footer: footer,
            nonce: nonce,
            auth_key: auth_key,
            encryption_key: encryption_key,
        }
    }
    pub(super) fn verify(self) -> Result<VerifiedToken, Error> {
        let mut message = self.buffer;
        let signature = message.split_off(self.message_boundary);
        let message = message.split_off(32);
        let encoded = multi_part_pre_auth_encoding(&[
            HEADER.to_combined().as_slice(),
            self.nonce.as_slice(),
            message.as_slice(),
            self.footer.as_ref().map_or(&[], |f| f.as_slice()),
        ])
        .map_err(|_| Error::BadSignature {})?;

        let signing_key = <HMAC_SHA384 as A>::Key::new(&self.auth_key);
        if <HMAC_SHA384 as SymmHashAlgo>::verify(
            encoded.as_slice(),
            signature.as_slice(),
            &signing_key,
        ) {
            Ok(VerifiedToken::new(
                message,
                self.footer,
                self.encryption_key,
                self.nonce,
            ))
        } else {
            Err(Error::BadSignature {})
        }
    }
}
impl VerifiedToken {
    fn new(
        encrypted_message: Vec<u8>,
        footer: Option<Vec<u8>>,
        encryption_key: EncryptionKey,
        nonce: Nonce,
    ) -> Self {
        Self {
            encrypted_message: encrypted_message,
            footer: footer,
            encryption_key: encryption_key,
            nonce: nonce,
        }
    }
    pub(super) fn decrypt(self) -> Result<token::SerializedData, symm::DecryptError> {
        let decrypted_msg = <AES256_CTR as symm::Algo>::decrypt(
            &<AES256_CTR as A>::Key::new(&self.encryption_key, self.nonce.get_crypt_nonce()),
            self.encrypted_message.as_slice(),
        )?;
        Ok(token::SerializedData {
            msg: decrypted_msg,
            footer: self.footer,
        })
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn v1_local_encrypt() {}
    #[test]
    fn v1_local_decrypt() {}
}
