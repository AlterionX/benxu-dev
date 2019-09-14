pub mod error;
pub mod data;

use std::{
    sync::Arc,
    str,
};

use rocket::{
    response::{
        status,
        Redirect,
    },
    http::{Status, Cookie, Cookies},
    State,
};
use rocket_contrib::json::Json;

use crate::{
    crypto::{
        KeyStore,
        algo::{
            Algo as A,
            hash::argon2::d::Algo as ARGON2D,
        },
        token::paseto,
    },
    blog::{
        db,
        auth,
    },
};

/// Route handler for the log in page.
#[get("/login")]
pub fn get() -> &'static str {
    "a login screen, eventually"
}

#[must_use]
fn add_authz_tok(cookies: &mut Cookies, tok: auth::CredentialToken, key: &<paseto::v2::local::Algo as A>::Key) -> Result<(), ()> {
    cookies.add(Cookie::build(
        auth::AUTH_COOKIE_NAME,
        paseto::v2::local::Protocol.encrypt(tok, key)
            .map_err(|_| ())
            .and_then(|s| str::from_utf8(&s).map(|s| s.to_owned()).map_err(|_| ()))?,
    ).secure(true).http_only(true).finish());
    Ok(())
}
/// Route handler for creating a session as well as creating an user.
#[post("/login", format = "json", data = "<auth_data>")]
pub fn post(
    db: db::DB,
    mut cookies: Cookies,
    tok_key_store: State<Arc<KeyStore<paseto::v2::local::Algo>>>,
    pw_key_store: State<Arc<KeyStore<ARGON2D>>>,
    auth_data: Json<data::Authentication>,
) -> Result<Option<Redirect>, status::Custom<()>> {
    let (user, perms) = auth_data.authenticate(
        &db,
        &pw_key_store,
    ).map_err(|e| status::Custom::from(e))?;
    if cookies.get(auth::AUTH_COOKIE_NAME).is_some() {
        Ok(None) // TODO create a landing page + replace
    } else {
        auth::attach_credentials_token(
            &tok_key_store.curr_and_last().map_err(|_| status::Custom(Status::InternalServerError, ()))?.curr,
            auth::UnverifiedPermissionsCredential::new(user.id, perms).into_inner(),
            &mut cookies,
        ).map_err(|_| status::Custom(Status::InternalServerError, ()))?;
        Ok(Some(Redirect::to("/"))) // TODO create a landing page + replace
    }
}

/// Route handler for deleting a session.
#[delete("/login")]
pub fn delete(mut cookies: Cookies) -> status::Custom<()> {
    cookies.remove(Cookie::named(auth::AUTH_COOKIE_NAME));
    status::Custom(Status::Ok, ())
}

