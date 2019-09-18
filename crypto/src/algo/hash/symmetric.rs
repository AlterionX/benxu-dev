use crate::algo as base;

pub trait Key: base::Key {}
pub trait Algo: base::Algo
where
    <Self as base::Algo>::Key: Key,
{
    type SigningInput: ?Sized;
    fn sign(&self, input: &Self::SigningInput, key: &Self::Key) -> Vec<u8>;
    type VerificationInput: ?Sized;
    fn verify(&self, input: &Self::VerificationInput, signature: &[u8], key: &Self::Key) -> bool;
}
