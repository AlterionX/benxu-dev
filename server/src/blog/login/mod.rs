//! Handlers and functions for handling logins/seessions.

pub mod data;
use data::CanAuthenticate;

use rocket::{
    http::{Cookies, Status},
    State,
};
use rocket_contrib::json::Json;

use crate::{
    blog::{auth, db},
    PWKeyFixture, TokenKeyFixture,
};
use blog_db::models::*;
use crypto::Generational;

/// Route handler for creating a session. Credentials passed in will be ignored if caller is
/// already logged in.
#[post("/login", format = "json", data = "<auth_data>")]
pub fn post(
    auth_data: Json<data::Authentication>,
    tok_key_store: State<TokenKeyFixture>,
    pw_key_store: State<PWKeyFixture>,
    mut cookies: Cookies,
    db: db::DB,
) -> Result<Json<users::DataNoMeta>, Status> {
    use log::*;
    info!("Processing data.");
    let (user, perms) = match auth_data.authenticate(&db, &pw_key_store) {
        Err(e) => {
            error!("{:?}", e);
            let e = Err(e.into());
            error!("Converted to: {:?}", e);
            return e;
        }
        Ok(user_and_p) => user_and_p,
    };
    debug!("Resolved to user {}.", user.user_name);
    auth::attach_credentials_token(
        &tok_key_store
            .get_store()
            .map_err(|_| Status::InternalServerError)?
            .curr,
        auth::UnverifiedPermissionsCredential::new(user.id, perms).into_inner(),
        &mut cookies,
    )
    .map_err(|_| Status::InternalServerError)?;
    debug!("Attached credential.");
    Ok(Json(user.strip_meta()))
}

/// Route handler for deleting a session. Will do nothing if not already in a session and will
/// alaways return OK.
#[delete("/login")]
pub fn delete(mut cookies: Cookies) {
    auth::detach_credentials_token_if_exists(&mut cookies);
}
