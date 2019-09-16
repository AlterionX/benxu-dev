//! Errors that can occur while using the permission endpoints.

pub(super) use diesel::result::Error as Diesel;

/// Represents possible errors from using the database for permissions.
pub enum Error {
    /// Database errors of many kinds.
    DB(Diesel),
    /// Insufficient permissions for accessing an endpoint for permissions.
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

