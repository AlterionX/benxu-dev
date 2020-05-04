//! Errors that can occur while using the capability endpoints.

pub(super) use diesel::result::Error as Diesel;

/// Represents possible errors from using the database for capabilities.
pub enum Error {
    /// Database errors of many kinds.
    DB(Diesel),
    /// Insufficient capabilities for accessing an endpoint for capabilities.
    Unauthorized,
}
impl From<Diesel> for Error {
    fn from(e: Diesel) -> Self {
        Self::DB(e)
    }
}
impl From<Error> for rocket::http::Status {
    fn from(e: Error) -> Self {
        match e {
            Error::DB(_) => Self::InternalServerError,
            Error::Unauthorized => Self::Unauthorized,
        }
    }
}
