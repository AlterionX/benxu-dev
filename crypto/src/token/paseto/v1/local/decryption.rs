use super::local_prelude::*;

/// Struct holding a further unpacking of the unpacked token.
pub(super) struct BasicToken {
    /// Backing memory for token body.
    buffer: Vec<u8>,
    // idx of separation between signature and nonce
    /// The index of the first character of the signature.
    message_boundary: usize,
    /// The footer.
    footer: Option<Vec<u8>>,
    /// The extracted nonce.
    nonce: Nonce,
}
/// Struct holding keys.
pub(super) struct PrimedToken {
    /// Backing memory for token body.
    buffer: Vec<u8>,
    // idx of separation between signature and nonce
    /// The index of the first character of the signature.
    message_boundary: usize,
    /// The footer.
    footer: Option<Vec<u8>>,
    /// The extracted nonce.
    nonce: Nonce,
    /// An authentication key.
    auth_key: AuthKey,
    /// An encryption key.
    encryption_key: EncryptionKey,
}
/// A token whose signature has been verified.
pub(super) struct VerifiedToken {
    /// Backing memory for token body.
    buffer: Vec<u8>,
    // idx of separation between signature and nonce
    /// The index of the first character of the signature.
    message_boundary: usize,
    /// The footer.
    footer: Option<Vec<u8>>,
    /// The extracted nonce.
    nonce: Nonce,
    /// An encryption key.
    encryption_key: EncryptionKey,
}

impl BasicToken {
    /// Extract the nonce and repack the token.
    pub(super) fn create_from(tok: token::Unpacked) -> Self {
        let nonce = Nonce::recreate_nonce(&tok.body[0..32]);
        let body_end_idx = tok.body.len() - 48;
        Self {
            buffer: tok.body,
            message_boundary: body_end_idx,
            nonce: nonce,
            footer: tok.footer,
        }
    }
    /// Prime the token with the auth and encryption keys.
    pub(super) fn prime(self, key: &[u8]) -> PrimedToken {
        let (ek, ak) = split_key(&self.nonce, key);
        PrimedToken::new(self, ak, ek)
    }
}
impl PrimedToken {
    /// Repacks the auth and encryption keys with the old token.
    fn new(
        tok: BasicToken,
        auth_key: AuthKey,
        encryption_key: EncryptionKey,
    ) -> Self {
        Self {
            buffer: tok.buffer,
            message_boundary: tok.message_boundary,
            footer: tok.footer,
            nonce: tok.nonce,
            auth_key: auth_key,
            encryption_key: encryption_key,
        }
    }
    /// Returns a slice to the message.
    fn message(&self) -> &[u8] {
        &self.buffer[32..self.message_boundary]
    }
    /// Returns a slice to the signature.
    fn signature(&self) -> &[u8] {
        &self.buffer[self.message_boundary..]
    }

    /// Validates the signature, progressing onto the next stage if valid.
    pub(super) fn verify(self) -> Result<VerifiedToken, Error> {
        let encoded = multi_part_pre_auth_encoding(&[
            HEADER.to_combined().as_slice(),
            self.nonce.as_slice(),
            self.message(),
            self.footer.as_ref().map_or(&[], |f| f.as_slice()),
        ])
        .map_err(|_| Error::BadSignature {})?;

        let signing_key = <HMAC_SHA384 as A>::Key::new(&self.auth_key);
        if HMAC_SHA384::new(()).verify(
            encoded.as_slice(),
            self.signature(),
            &signing_key,
        ) {
            Ok(VerifiedToken::new(self))
        } else {
            Err(Error::BadSignature {})
        }
    }
}
impl VerifiedToken {
    /// Strips the auth key and marks the token as verified.
    fn new(tok: PrimedToken) -> Self {
        Self {
            buffer: tok.buffer,
            message_boundary: tok.message_boundary,
            footer: tok.footer,
            encryption_key: tok.encryption_key,
            nonce: tok.nonce,
        }
    }
    fn message(&self) -> &[u8] {
        &self.buffer[32..self.message_boundary]
    }
    /// Decrypts the encrypted message and repacks it into [`SerializedData`].
    pub(super) fn decrypt(self) -> Result<token::SerializedData, symm::DecryptError> {
        ENC_ALGO::new(())
            .decrypt(
                &ENC_KEY::new(&self.encryption_key, self.nonce.get_crypt_nonce()),
                self.message(),
            )
            .map(|decrypted| token::SerializedData {
                msg: decrypted,
                footer: self.footer,
            })
    }
}

