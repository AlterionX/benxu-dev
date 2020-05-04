//! Handlers and functions for password capabilities.

mod data;

use rocket::{http::Status, State};
use rocket_contrib::{json::Json, uuid::Uuid as RUuid};

use crate::{
    cfg::PWKeyFixture,
    util::{
        auth::{self, credentials::SavableCredential},
        blog::{db::PWQuery, DB},
        uuid_compat::ruuid_to_uuid,
    },
};

/// Allows for the creation of new passwords. Only functions if attempting to create a password
/// for self or if the caller possesses the
/// [`EditUserCapabilities`](crate::blog::auth::caps::EditUserCapabilities) capabilities.
///
/// Can only use this to create passwords, not update them.
#[post("/credentials/pws", format = "json", data = "<to_create>")]
pub fn post(
    db: DB,
    capabilities: auth::UnverifiedCapabilities,
    pw_key_store: State<PWKeyFixture>,
    to_create: Json<data::CreatePassword>,
) -> Status {
    use log::*;
    let key = pw_key_store.key();
    let to_create = data::PasswordWithBackingInfo {
        db: &db,
        capabilities: &capabilities,
        argon2d_key: &key,
        pw: &to_create,
    };
    let res = to_create.convert_and_save_with_capabilities();
    debug!("Running query resulted in: {:?}", res);
    res.map_or_else(|_| Status::InternalServerError, |_| Status::Ok)
}

/// Handlers for manipulating password records.
pub mod pw {
    use super::*;

    /// Handler for changing a password. Must be chaning own capabilities or have the
    /// [`EditUserCapability`](crate::blog::auth::caps::EditUserCapability) capabilities.
    #[patch("/credentials/pws/<id>", format = "json", data = "<changed_pw>")]
    pub fn patch(
        db: DB,
        pw_key_store: State<PWKeyFixture>,
        capabilities: auth::UnverifiedCapabilities,
        id: RUuid,
        changed_pw: Json<String>,
    ) -> Result<Status, Status> {
        let id = ruuid_to_uuid(id);
        let target_user_id =
            db.find_pw_by_id(id)
                .map(|pw_rec| pw_rec.user_id)
                .map_err(|e| match e {
                    diesel::result::Error::NotFound => Status::NotFound,
                    _ => Status::InternalServerError,
                })?;
        let capabilities: auth::UnverifiedCapabilities = capabilities
            .into_inner()
            .change_level::<auth::caps::EditUserCredentials>()
            .map(|cr| cr.back_to_any())
            .or_else(|cr| {
                if target_user_id == cr.user_id() {
                    Ok(cr)
                } else {
                    Err(Status::Unauthorized)
                }
            })?
            .into();
        let update = data::CreatePassword {
            user_id: capabilities.user_id(),
            password: changed_pw.into_inner(),
        };
        let key = pw_key_store.key();
        let to_create = data::PasswordWithBackingInfo {
            db: &db,
            capabilities: &capabilities,
            argon2d_key: &key,
            pw: &update,
        };
        to_create
            .convert_and_update_with_capabilities()
            .map(|_| Status::Ok)
            .map_err(|_| Status::InternalServerError)
    }
    /// Handler for deleting a password. Must be changing own credentials or have the
    /// [`EditUserCredentials`](crate::blog::auth::caps::EditUserCredentials) capabilities.
    ///
    /// An example use case is when you wish to utilize only FIDO or OAuth to log in.
    #[delete("/credentials/pws/<id>")]
    pub fn delete(
        db: DB,
        capabilities: auth::UnverifiedCapabilities,
        id: RUuid,
    ) -> Result<Status, Status> {
        let id = ruuid_to_uuid(id);
        let target_user_id =
            db.find_pw_by_id(id)
                .map(|pw_rec| pw_rec.user_id)
                .map_err(|e| match e {
                    diesel::result::Error::NotFound => Status::NotFound,
                    _ => Status::InternalServerError,
                })?;
        capabilities
            .into_inner()
            .change_level::<auth::caps::EditUserCredentials>()
            .map(|_| ())
            .or_else(|cr| {
                if target_user_id == cr.user_id() {
                    Ok(())
                } else {
                    Err(Status::Unauthorized)
                }
            })?;
        db.delete_pw_by_id(id)
            .map(|_| Status::Ok)
            .map_err(|_| Status::InternalServerError)
    }
}
