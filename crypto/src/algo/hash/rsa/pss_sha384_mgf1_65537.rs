use crate::algo as base;
use base::hash::asymmetric::{self as asymm};

use openssl::{
    error::ErrorStack,
    hash::MessageDigest,
    pkey::{PKey, Private, Public},
    rsa::{Padding, Rsa},
    sign::{RsaPssSaltlen, Signer, Verifier},
};
use std::option::NoneError;

#[derive(Debug)]
pub enum KeyGenError {
    OpenSSL(ErrorStack),
}
impl From<ErrorStack> for KeyGenError {
    fn from(e: ErrorStack) -> Self {
        Self::OpenSSL(e)
    }
}

pub struct KeyPair {
    source: Vec<u8>,
    private: Option<PKey<Private>>,
    public: PKey<Public>,
}
impl KeyPair {
    fn create_from(der: Vec<u8>) -> Result<Self, ErrorStack> {
        let private = Rsa::private_key_from_der(der.as_slice())
            .map(|private_key| Some(private_key))
            .unwrap_or(None);
        let public = if let Some(private) = &private {
            Rsa::public_key_from_der(private.public_key_to_der()?.as_slice())?
        } else {
            Rsa::public_key_from_der(der.as_slice())?
        };
        Ok(Self {
            source: der,
            private: private.map(|private| PKey::from_rsa(private)).transpose()?,
            public: PKey::from_rsa(public)?,
        })
    }
}
impl Clone for KeyPair {
    /// panics under probably exceptional circumstances
    fn clone(&self) -> Self {
        // source should be valid if KeyPair had been successfully created, especially since it
        // worked last time.
        Self::create_from(self.source.clone()).unwrap()
    }
}
impl base::Key for KeyPair {
    type Settings = ();
    type Error = KeyGenError;
    fn generate(_: &Self::Settings) -> Result<Self, KeyGenError> {
        // rust openssl as of 0.10.24 automatically uses required 65537 for exponent
        let private = Rsa::generate(2048)?;
        Ok(Self::create_from(private.private_key_to_der()?)?)
    }
}
impl asymm::KeyPair for KeyPair {
    type Private = PKey<Private>;
    type Public = PKey<Public>;
    fn public_key(&self) -> &Self::Public {
        &self.public
    }
    fn private_key<'a>(&'a self) -> Option<&'a Self::Private> {
        self.private.as_ref()
    }
}

#[derive(Debug)]
pub enum AlgoError {
    DoesNotHavePrivateKey,
    OpenSSL,
}
impl From<NoneError> for AlgoError {
    fn from(_: NoneError) -> Self {
        Self::DoesNotHavePrivateKey
    }
}
impl From<ErrorStack> for AlgoError {
    fn from(_: ErrorStack) -> Self {
        AlgoError::OpenSSL
    }
}

pub struct Algo;
impl base::Algo for Algo {
    type Key = KeyPair;
    fn key_settings<'a>(&'a self) -> &'a <<Self as base::Algo>::Key as base::Key>::Settings {
        &()
    }
    fn new(_: Self::ConstructionData) -> Self {
        Self
    }
}
impl asymm::Algo for Algo {
    type SigningError = AlgoError;
    type VerifyError = AlgoError;

    /// unimplemented
    fn sign_public(
        _msg: &[u8],
        _key: &<Self::Key as asymm::KeyPair>::Public,
    ) -> Result<Vec<u8>, Self::SigningError> {
        unimplemented!("Unimplemented by ring");
    }
    fn verify_public(
        msg: &[u8],
        signature: &[u8],
        key: &<Self::Key as asymm::KeyPair>::Public,
    ) -> Result<bool, Self::VerifyError> {
        let mut verifier = Verifier::new(MessageDigest::sha384(), key)?;
        verifier.set_rsa_padding(Padding::PKCS1_PSS).unwrap();
        verifier
            .set_rsa_pss_saltlen(RsaPssSaltlen::DIGEST_LENGTH)
            .unwrap();
        verifier.set_rsa_mgf1_md(MessageDigest::sha384()).unwrap();
        verifier.update(msg)?;
        Ok(verifier.verify(signature)?)
    }
    fn sign_private(
        msg: &[u8],
        key: Option<&<Self::Key as asymm::KeyPair>::Private>,
    ) -> Result<Vec<u8>, Self::SigningError> {
        let mut signer = Signer::new(MessageDigest::sha384(), key?)?;
        signer.set_rsa_padding(Padding::PKCS1_PSS).unwrap();
        signer
            .set_rsa_pss_saltlen(RsaPssSaltlen::DIGEST_LENGTH)
            .unwrap();
        signer.set_rsa_mgf1_md(MessageDigest::sha384()).unwrap();
        signer.update(msg)?;
        Ok(signer.sign_to_vec()?)
    }
    /// unimplemented
    fn verify_private(
        _msg: &[u8],
        _signature: &[u8],
        _key: Option<&<Self::Key as asymm::KeyPair>::Private>,
    ) -> Result<bool, Self::VerifyError> {
        unimplemented!("Unimplemented by ring");
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::algo::{
        hash::{
            asymmetric::{
                Algo as AsymmAlgo,
                KeyPair as AsymmKeyPair, // naming conflict
            },
            rsa::pss_sha384_mgf1_65537::{Algo, KeyPair},
        },
        Key,
    };
    use openssl::pkey::PKey;

    #[test]
    fn test_externally_signed() {
        let pub_key = hex::decode(
            "\
             30820122300d06092a864886f70d01010105000382010f003082010a0282\
             010100b1670b261b5dc92ee8c14889ab87bc20e93e23dd96b5f670568da4\
             64686a72d39f8f9df68f7d02346fb225aa44a1c78fd89560cc60fa0620dc\
             532e54e0394d04ee8de7313db54ecdf8d7405f66664789336be5d37b1a31\
             8721308cb44f8dcf1a6b0ccee07ad62e9ecab6564450d3d3292561aab027\
             8e1da20469c6ce5613acc05ca1c0911a2c7712b210493dcba4075d104524\
             5377d7b31025debc3f59deaa114523002664f1f1aa789ef02ede0f6f851c\
             566a85b60e5e2f608e791f456ad4f1ba9746805b65fd88fa987030321a1d\
             caeb97db29987277fc81bacdf05cf65b3053d43a59fd6a19e42cf433e049\
             765217fdae334bafd64b94bd30ee65eb010b150203010001\
             ",
        )
        .unwrap();
        let sig = hex::decode(
            "\
             0d2ad574c742869fa1e8072dbd838bceb2772286a3cc4fa777067f314e8d\
             a81cdef0fdcf7a29d38a5b795f73698f01fd363f50c1299e1e09702332bf\
             fce4bc594a0863c70d27b8284b2c2edb523237de4ff582323b950617955b\
             f80bbf86fcbc4770579f09f2785d0ed6a12815c4e9ea8612611cc988c8da\
             6905a3c0cb6e1448de1d30b9ab073d36021cae0cb7883443ec6ebb729843\
             1e5c7d481134d5e0240e2f13d7c7636157118320da80f4b1d97233c8d130\
             49036ca73d4fd9ee8210c73f5653a22d05b511f93e1806b1cad176e7e634\
             ef1808537112d42be4d77ba3657abacaad598078ce566ebe360272a3a1cf\
             734d74d48dc5969a880b8d90ee6c20f8\
             ",
        )
        .unwrap();
        let verification = Algo::verify_public(
            b"hello\n",
            sig.as_slice(),
            &PKey::public_key_from_der(&pub_key).unwrap(),
        )
        .unwrap();
        assert!(verification == true);
    }

    #[test]
    fn test_sign_and_verify() {
        let key: KeyPair = KeyPair::generate(&()).unwrap();
        let sig: Vec<u8> = Algo::sign_private(b"hello", Some(key.private_key().unwrap())).unwrap();
        let verification = Algo::verify_public(b"hello", sig.as_slice(), key.public_key()).unwrap();
        assert!(verification == true);
    }
}
