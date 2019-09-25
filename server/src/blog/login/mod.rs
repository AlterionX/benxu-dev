//! Handlers and functions for handling logins/seessions.

pub mod data;
use data::CanAuthenticate;

use rocket::{
    http::{Cookies, Status},
    response::Redirect,
    State,
};
use rocket_contrib::json::Json;

use crate::{
    blog::{auth, db},
    PWKeyFixture, TokenKeyFixture,
};
use crypto::Generational;

/// Route handler for creating a session. Credentials passed in will be ignored if caller is
/// already logged in.
#[post("/login", format = "json", data = "<auth_data>")]
pub fn post(
    db: db::DB,
    mut cookies: Cookies,
    tok_key_store: State<TokenKeyFixture>,
    pw_key_store: State<PWKeyFixture>,
    auth_data: Json<data::Authentication>,
) -> Result<Option<Redirect>, Status> {
    use log::*;
    if cookies.get(auth::AUTH_COOKIE_NAME).is_some() {
        Ok(None) // TODO create a landing page + replace
    } else {
        info!("Processing data.");
        let (user, perms) = auth_data.authenticate(&db, &pw_key_store)?;
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
        Ok(Some(Redirect::to("/"))) // TODO create a landing page + replace
    }
}

/// Route handler for deleting a session. Will do nothing if not already in a session and will
/// alaways return OK.
#[delete("/login")]
pub fn delete(mut cookies: Cookies) {
    auth::detach_credentials_token_if_exists(&mut cookies);
}
