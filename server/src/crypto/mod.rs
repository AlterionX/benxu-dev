pub mod token;
pub mod algo;
pub mod key_rotation;
pub use key_rotation::{
    CurrAndLastKey,
    KeyStore,
    KeyRotator,
};

// boolean to result convenience
pub trait BoolToResult {
    fn to_result(self) -> Result<(), ()>;
    fn ok_or<E>(self, e: E) -> Result<(), E>;
    fn ok_or_else<E, F: FnOnce() -> E>(self, f: F) -> Result<(), E>;
}
impl BoolToResult for bool {
    fn to_result(self) -> Result<(), ()> {
        if self {
            Ok(())
        } else {
            Err(())
        }
    }
    fn ok_or<E>(self, e: E) -> Result<(), E> {
        if self {
            Ok(())
        } else {
            Err(e)
        }
    }
    fn ok_or_else<E, F: FnOnce() -> E>(self, f: F) -> Result<(), E> {
        if self {
            Ok(())
        } else {
            Err(f())
        }
    }
}

