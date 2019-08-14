use crate::crypto::algo as base;

pub trait Key: base::Key {}
pub trait Algo: base::Algo where <Self as base::Algo>::Key : Key {
    fn sign(msg: &[u8], key: &Self::Key) -> Vec<u8>;
    fn verify(msg: &[u8], signature: &[u8], key: &Self::Key) -> bool;
}
