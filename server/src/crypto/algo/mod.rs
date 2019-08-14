pub mod cipher;
pub mod hash;
pub mod key_deriv;

pub trait Key {
}
pub trait Algo {
    type Key: Key + Clone + Send + Sync;
    fn generate_key() -> <Self as Algo>::Key;
}

