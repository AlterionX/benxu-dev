//! Handlers and functions for account management.

use rocket::{
    http::{Cookies, Status},
    State,
};
use rocket_contrib::{json::Json, uuid::Uuid as RUuid};
use tap::*;

use crate::{
    cfg::TokenKeyFixture,
    util::{
        auth,
        blog::{db::UserQuery, DB},
        uuid_compat::ruuid_to_uuid,
    },
};
use blog_db::models::*;
use crypto::Generational;

/// Handler for creating an account.
///
/// Creates the `user_to_create` as stated in [`create_account`]. Also logs the user in question
/// into the newly created account provided that they are not already logged in. Must have caps
/// for [`CreateUser`][crate::blog::auth::caps::CreateUser] if already logged in.
///
/// As of now, no default account capabilities are provided on creation on the server side.
#[post("/accounts", format = "json", data = "<user_to_create>")]
pub fn post(
    capabilities: Option<auth::UnverifiedCapabilities>,
    user_to_create: Json<users::NewNoMeta>,
    db: DB,
    mut cookies: Cookies,
    tok_key_store: State<TokenKeyFixture>,
) -> Result<Json<users::DataNoMeta>, Status> {
    let user_to_create = user_to_create.into_inner();
    log::debug!("Attempting to create account {:?}.", user_to_create);
    let creator = capabilities
        .map(auth::UnverifiedCapabilities::into_inner)
        .map(auth::Capabilities::change_level::<auth::caps::CreateUser>)
        .transpose()
        .map_err(|_| Status::Unauthorized)?
        .map(|cr| cr.user_id());
    let created = create_account(&db, creator, user_to_create)
        .tap_err(|e| log::error!("Account creation failed due to {:?}", e))
        .map_err(|_| Status::InternalServerError)?;
    // Add token if not already logged in to facilitate credential creation.
    // If a credential is not created in the first session, they will currently need to contact the
    // site admin to log in again.
    if creator.is_none() {
        let key = &tok_key_store
            .get_store()
            .tap_err(|_| log::error!("Token key service crashed."))
            .map_err(|_| Status::InternalServerError)?
            .curr;
        let new_capabilities = auth::Capabilities::<()>::safe_new(created.id, vec![]);
        auth::attach_capabilities_token(key, new_capabilities, &mut cookies)
            .map_err(|_| Status::InternalServerError)?;
    }
    Ok(Json(created.strip_meta()))
}
/// Creates an account.
///
/// Creates the `user_to_create` in the provided `db`. If the `creator` is not `None`, the
/// created_by field will be set to the creator.
#[must_use]
pub fn create_account(
    db: &DB,
    creator: Option<uuid::Uuid>,
    user_to_create: users::NewNoMeta,
) -> Result<users::Data, diesel::result::Error> {
    Ok(db.create_user(users::New::from((&user_to_create, creator)))?)
}

/// Handlers and functions for managing individual accounts.
pub mod account {
    use super::*;

    /// Handler to get the account info. Accounts are private only for now -- you can only
    /// view this page if you're logged in as the correct user.
    #[get("/accounts/<id>")]
    pub fn get(
        db: DB,
        id: RUuid,
        capabilities: auth::UnverifiedCapabilities,
    ) -> Result<Json<users::DataNoMeta>, Status> {
        let id = ruuid_to_uuid(id);
        if capabilities.user_id() != id {
            return Err(Status::Unauthorized);
        }
        db.find_user_by_id(id)
            .map(users::Data::strip_meta)
            .map(Json)
            .map_err(|_| Status::InternalServerError)
    }
    /// Handler to get the account info page. Accounts are private only for now -- you can only
    /// view this page if you're logged in as the correct user.
    #[get("/accounts/me")]
    pub fn get_self(
        db: DB,
        capabilities: Option<auth::UnverifiedCapabilities>,
    ) -> Result<Json<users::DataNoMeta>, Status> {
        let capabilities = capabilities.ok_or(Status::Unauthorized)?;
        let id = capabilities.user_id();
        db.find_user_by_id(id)
            .map(users::Data::strip_meta)
            .map(Json)
            .map_err(|_| Status::InternalServerError)
    }
    /// Handler to allow editing of user information if logged in as same user or has capabilities
    /// to edit users.
    #[patch("/accounts/<id>", format = "json", data = "<changes>")]
    pub fn patch(
        db: DB,
        id: RUuid,
        capabilities: auth::UnverifiedCapabilities,
        changes: Json<users::ChangedNoMeta>,
    ) -> Result<Json<users::DataNoMeta>, Status> {
        let id = ruuid_to_uuid(id);
        let changes = changes.into_inner();
        let updater = capabilities
            .into_inner()
            .change_level::<auth::caps::EditUser>()
            .map(|cr| cr.user_id())
            .or_else(|cr| {
                if id == cr.user_id() {
                    Ok(cr.user_id())
                } else {
                    Err(Status::Unauthorized)
                }
            })?;
        let changes = (&changes, Some(updater)).into();
        db.update_user_by_id(id, changes)
            .map(users::Data::strip_meta)
            .map(Json)
            .map_err(|_| Status::InternalServerError)
    }
    /// Handler to allow for the deletion of accounts if logged in as same user or has capabilities
    /// to delete users.
    #[delete("/accounts/<id>")]
    pub fn delete(
        db: DB,
        id: RUuid,
        capabilities: auth::UnverifiedCapabilities,
    ) -> Result<Status, Status> {
        let id = ruuid_to_uuid(id);
        capabilities
            .into_inner()
            .change_level::<auth::caps::DeleteUser>()
            .map(|cr| cr.user_id())
            .or_else(|cr| {
                if id == cr.user_id() {
                    Ok(cr.user_id())
                } else {
                    Err(Status::Unauthorized)
                }
            })?;
        db.delete_user_by_id(id)
            .map(|_| Status::Ok)
            .map_err(|_| Status::InternalServerError)
    }
}
