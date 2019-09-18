//! A collection of traits and types to handle asymmetric block cipher algorithms.

use crate::algo as base;

/// The key of the algorithm. The minimum requirement is to implement the [`HasPublic`] trait.
pub trait Key: base::Key + HasPublic {}
/// Denotes that a key contains the public components.
pub trait HasPublic {
    /// The public component of the key.
    type PublicKey;
    /// Gets a reference to the public component.
    fn public_key(&self) -> &Self::PublicKey;
}
/// Denotes that a key contains the private components.
pub trait HasPrivate: HasPublic {
    /// The private component of the key.
    type PrivateKey;
    /// Gets a reference to the private component.
    fn private_key(&self) -> &Self::PrivateKey;
}

/// Trait marking a block cipher algorithm algo
pub trait Algo: base::Algo where <Self as base::Algo>::Key: Key {}
/// Trait marking that a block cipher algorithm can encrypt with a public key.
pub trait CanEncryptPublic: Algo where <Self as base::Algo>::Key: Key + HasPublic {
    type PublicKey = <Self::Key as HasPublic>::PublicKey;
    type Error;
    type Input: ?Sized;
    fn public_encrypt(key: &Self::PublicKey, data: &Self::Input) -> Result<Vec<u8>, Self::Error>;
}
/// Trait marking that a block cipher algorithm can decrypt with a public key.
pub trait CanDecryptPublic: Algo where <Self as base::Algo>::Key: Key + HasPublic {
    type PublicKey = <Self::Key as HasPublic>::PublicKey;
    type Error;
    type Input: ?Sized;
    fn public_decrypt(key: &Self::PublicKey, data: &Self::Input) -> Result<Vec<u8>, Self::Error>;
}
/// Trait marking that a block cipher algorithm can encrypt with a private key.
pub trait CanEncryptPrivate: Algo where <Self as base::Algo>::Key: Key + HasPrivate {
    type PrivateKey = <Self::Key as HasPrivate>::PrivateKey;
    type Error;
    type Input: ?Sized;
    fn private_encrypt(key: &Self::PrivateKey, data: &Self::Input) -> Result<Vec<u8>, Self::Error>;
}
/// Trait marking that a block cipher algorithm can decrypt with a private key.
pub trait CanDecryptPrivate: Algo where <Self as base::Algo>::Key: Key + HasPrivate {
    type PrivateKey = <Self::Key as HasPrivate>::PrivateKey;
    type Error;
    type Input: ?Sized;
    fn private_decrypt(key: &Self::PrivateKey, data: &Self::Input) -> Result<Vec<u8>, Self::Error>;
}

// TODO macro for tests?
