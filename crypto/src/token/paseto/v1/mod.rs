/// The PASETO version one protocol.
pub mod local;
pub mod public;

pub mod nonce;

/// The purposes of the version one protocol.
pub enum Type {
    /// The local protocol. Used for opaque tokens.
    Local,
    /// The public protocol. Used for transparent tokens.
    Public,
}
