use std::sync::Arc;
use rocket::{
    response::status,
    http::Status,
    request::{
        Request,
        FromRequest,
        Outcome,
    },
    State,
};
use serde::{
    Serialize,
    Deserialize,
};
use crate::{
    crypto::{
        KeyStore,
        token::paseto,
    },
    DefaultAlgo,
};

#[derive(Serialize, Deserialize)]
enum Permission {
    User,
    Editor,
    Admin,
    SuperAdmin,
    Custom { name: String },
}
#[derive(Serialize, Deserialize)]
struct Credentials<'r> {
    permissions: Vec<Permission>,
    hash: &'r str,
}
impl<'a, 'r> FromRequest<'a, 'r> for Credentials<'r> {
    type Error = ();
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        // TODO make this a const eventually
        let EMPTY_CREDENTIALS: Credentials<'static> = Credentials {
            permissions: vec![],
            hash: "",
        };
        let cookies = req.cookies();
        let ref key_store = &*req.guard::<State<Arc<KeyStore<DefaultAlgo>>>>()?;
        if let Some(encrypted_token) = cookies.get("") {
            if let Ok(json) = paseto::decrypt_phase_one::<Credentials>(encrypted_token.value().as_bytes()) {
                // parse json
            } else {
                // 403
            }
            Outcome::Success(EMPTY_CREDENTIALS)
        } else {
            Outcome::Success(EMPTY_CREDENTIALS)
        }
    }
}
#[get("/login")]
pub fn get() -> &'static str {
    "a login screen, eventually"
}
#[post("/login")]
pub fn post() -> status::Custom<()> {
    status::Custom(Status::new(501, "Not yet implemented"), ())
}
#[delete("/login")]
pub fn delete() -> status::Custom<()> {
    status::Custom(Status::new(501, "Not yet implemented"), ())
}

