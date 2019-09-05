pub mod local;
pub mod public;

pub mod nonce;

pub enum Type {
    Local(local::Protocol),
    Public(public::Protocol),
}

