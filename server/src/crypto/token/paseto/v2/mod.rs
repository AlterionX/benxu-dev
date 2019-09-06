pub mod local;
pub mod public;

pub enum Type {
    Local(local::Protocol),
    Public(public::Protocol),
}

