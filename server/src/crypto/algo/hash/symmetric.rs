use crate::crypto::algo as base;

pub trait Key: base::Key {}
pub trait Algo: base::Algo where <Self as base::Algo>::Key : Key {
    type SigningInput: ?Sized;
    fn sign(input: &Self::SigningInput, key: &Self::Key) -> Vec<u8>;
    type VerificationInput: ?Sized;
    fn verify(input: &Self::VerificationInput, signature: &[u8], key: &Self::Key) -> bool;
}
