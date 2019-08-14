pub mod local;
pub mod public;

pub mod nonce;

pub enum Type {
    Local(local::Token),
    Public(public::Token),
}

