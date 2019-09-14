use rocket::{
    response::status,
    http::Status,
};

/// Errors for authentication.
#[derive(Debug)]
pub enum Error {
    /// Database errored when attempting operation.
    Diesel(diesel::result::Error),
    /// Authenticated user lacks permissions to create a user.
    LackingPermissions,
    /// Credentials do not match user.
    BadCredentials,
    Unauthorized,
    /// KeyStore is poisoned.
    KeyStorePoisoned,
    KeyStoreAbsent,
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
            Error::Diesel(_) => Status::InternalServerError,
            Error::LackingPermissions => Status::InternalServerError,
            Error::BadCredentials => Status::InternalServerError,
            Error::KeyStorePoisoned => Status::InternalServerError,
            Error::Unauthorized => Status::Unauthorized,
            Error::KeyStoreAbsent => Status::InternalServerError,
        }
    }
}
impl From<Error> for (Status, Error) {
    fn from(e: Error) -> Self {
        (Status::from(&e), e)
    }
}
impl From<Error> for status::Custom<()> {
    fn from(e: Error) -> Self {
        status::Custom(Status::from(&e), ())
    }
}
impl From<Error> for status::Custom<Error> {
    fn from(e: Error) -> Self {
        status::Custom(Status::from(&e), e)
    }
}

#[cfg(test)]
mod unit_tests {
}

