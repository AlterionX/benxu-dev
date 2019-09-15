use crate::algo as base;

pub trait KeyPair: base::Key {
    type Private;
    type Public;
    fn public_key(&self) -> &Self::Public;
    fn private_key(&self) -> Option<&Self::Private>;
}
pub trait Algo: base::Algo
    where <Self as base::Algo>::Key: KeyPair
{
    type SigningError;
    type VerifyError;
    fn sign_public(
        msg: &[u8],
        key: &<Self::Key as KeyPair>::Public,
    ) -> Result<Vec<u8>, Self::SigningError>;
    fn verify_public(
        msg: &[u8],
        signature: &[u8],
        key: &<Self::Key as KeyPair>::Public,
    ) -> Result<bool, Self::VerifyError>;
    fn sign_private(
        msg: &[u8],
        // TODO decide if this should be Option
        key: Option<&<Self::Key as KeyPair>::Private>,
    ) -> Result<Vec<u8>, Self::SigningError>;
    fn verify_private(
        msg: &[u8],
        signature: &[u8],
        // TODO decide if this should be Option
        key: Option<&<Self::Key as KeyPair>::Private>,
    ) -> Result<bool, Self::VerifyError>;
}
