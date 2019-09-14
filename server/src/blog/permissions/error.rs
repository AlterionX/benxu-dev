pub(super) use diesel::result::Error as Diesel;

pub enum Error {
    DB(Diesel),
    Unauthorized,
}
impl From<Diesel> for Error {
    fn from(e: Diesel) -> Self {
        Self::DB(e)
    }
}
impl From<Error> for rocket::response::status::Custom<()> {
    fn from(e: Error) -> Self {
        Self(match e {
            Error::DB(_) => rocket::http::Status::InternalServerError,
            Error::Unauthorized => rocket::http::Status::Unauthorized,
        }, ())
    }
}

