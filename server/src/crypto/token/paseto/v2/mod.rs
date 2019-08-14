pub mod local;
pub mod public;

pub enum Type {
    Local(local::Token),
    Public(public::Token),
}

