use crate::crypto::algo as base;

pub trait Key: base::Key {
    type Private;
    type Public;
    fn public_key(&self) -> &Self::Public;
    fn private_key(&self) -> &Self::Private;
}
pub trait Algo: base::Algo
    where <Self as base::Algo>::Key: Key
{
    fn sign_public(
        msg: &[u8],
        key: &<Self::Key as Key>::Public,
    ) -> Vec<u8>;
    fn verify_public(
        msg: &[u8],
        signature: &[u8],
        key: &<Self::Key as Key>::Public,
    ) -> bool;
    fn sign_private(
        msg: &[u8],
        key: &<Self::Key as Key>::Private,
    ) -> Vec<u8>;
    fn verify_private(
        msg: &[u8],
        signature: &[u8],
        key: &<Self::Key as Key>::Private,
    ) -> bool;
}
