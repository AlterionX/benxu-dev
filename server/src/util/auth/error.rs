//! Error data and conversions.

use diesel::result::Error as DieselError;
use rocket::{http::Status, response::status};

use crypto::token::paseto::V2LocalError as DecryptError;

/// Errors for authentication.
#[derive(Debug)]
pub enum Error {
    /// Database errored when attempting operation.
    Diesel(diesel::result::Error),
    /// Authenticated user lacks capabilities to create a user.
    LackingCapabilities,
    /// Capabilities do not match user.
    BadCredentials,
    /// Capabilities are lacking.
    Unauthorized,
    /// KeyStore is poisoned.
    KeyStorePoisoned,
    /// Did not initialize a key store. Probably forgot to [`rocket::Rocket::manage()`] it.
    KeyStoreAbsent,
}
impl From<DecryptError> for Error {
    fn from(_: DecryptError) -> Self {
        Self::Unauthorized
    }
}
impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        Self::Diesel(e)
    }
}
impl<G> From<std::sync::PoisonError<G>> for Error {
    fn from(_: std::sync::PoisonError<G>) -> Self {
        Self::KeyStorePoisoned
    }
}
impl From<&Error> for Status {
    fn from(e: &Error) -> Self {
        match e {
            Error::Diesel(DieselError::NotFound) => Status::NotFound,
            Error::Diesel(_) => Status::InternalServerError,
            Error::LackingCapabilities => Status::InternalServerError,
            Error::BadCredentials => Status::InternalServerError,
            Error::KeyStorePoisoned => Status::InternalServerError,
            Error::Unauthorized => Status::Unauthorized,
            Error::KeyStoreAbsent => Status::InternalServerError,
        }
    }
}
impl From<Error> for Status {
    fn from(e: Error) -> Self {
        (&e).into()
    }
}
impl From<Error> for (Status, Error) {
    fn from(e: Error) -> Self {
        ((&e).into(), e)
    }
}
impl From<Error> for status::Custom<()> {
    fn from(e: Error) -> Self {
        status::Custom(e.into(), ())
    }
}
impl From<Error> for status::Custom<Error> {
    fn from(e: Error) -> Self {
        status::Custom((&e).into(), e)
    }
}

//TODO unit tests?
