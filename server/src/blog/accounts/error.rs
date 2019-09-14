use diesel::result::Error as DieselError;
use serde_json::error::Error as JsonError;

use crate::blog::auth;

pub enum Create {
    Diesel(DieselError),
    Auth(auth::Error),
}
impl From<DieselError> for Create {
    fn from(e: DieselError) -> Self {
        Self::Diesel(e)
    }
}
impl From<auth::Error> for Create {
    fn from(e: auth::Error) -> Self {
        Self::Auth(e)
    }
}

pub enum Extract {
    Json(JsonError),
    Auth(auth::Error),
}
impl From<JsonError> for Extract {
    fn from(e: JsonError) -> Self {
        Self::Json(e)
    }
}
impl From<auth::Error> for Extract {
    fn from(e: auth::Error) -> Self {
        Self::Auth(e)
    }
}
impl From<Extract> for (rocket::http::Status, Extract) {
    fn from(e: Extract) -> Self {
        (match e {
            Extract::Json(_) => rocket::http::Status::BadRequest,
            Extract::Auth(_) => rocket::http::Status::Unauthorized,
        }, e)
    }
}

