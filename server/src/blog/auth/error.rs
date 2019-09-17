//! Error data and conversions.

use rocket::{
    response::status,
    http::Status,
};
use diesel::result::Error as DieselError;

use crypto::token::paseto::v2::local::error::Error as DecryptError;

/// Errors for authentication.
#[derive(Debug)]
pub enum Error {
    /// Database errored when attempting operation.
    Diesel(diesel::result::Error),
    /// Authenticated user lacks permissions to create a user.
    LackingPermissions,
    /// Credentials do not match user.
    BadCredentials,
    /// Credentials are lacking.
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
            Error::LackingPermissions => Status::InternalServerError,
            Error::BadCredentials => Status::InternalServerError,
            Error::KeyStorePoisoned => Status::InternalServerError,
            Error::Unauthorized => Status::Unauthorized,
            Error::KeyStoreAbsent => Status::InternalServerError,
        }
    }
}
impl From<Error> for Status {
    fn from(e: Error) -> Self {
        e.into()
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

