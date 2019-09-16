//! Handlers and functions for account management.

use rocket::{
    http::{Status, Cookies},
    State,
};
use rocket_contrib::{
    json::Json,
    uuid::Uuid as RUuid,
};

use blog_db::models::*;
use crypto::Generational;
use crate::{
    TokenKeyFixture,
    uuid_conv::FromRUuid,
    blog::{
        db,
        auth,
    },
};

/// Handler for creating an account.
///
/// Creates the `user_to_create` as stated in [`create_account`]. Also logs the user in question
/// into the newly created account provided that they are not already logged in. Must have perms
/// for [`CanCreateUser`][crate::blog::auth::perms::CanCreateUser] if already logged in.
///
/// As of now, no default account permissions are provided on creation on the server side.
#[post("/accounts", format = "json", data = "<user_to_create>")]
pub fn post(
    db: db::DB,
    mut cookies: Cookies,
    tok_key_store: State<TokenKeyFixture>,
    credentials: Option<auth::UnverifiedPermissionsCredential>,
    user_to_create: Json<users::NewNoMeta>,
) -> Result<Status, Status> {
    let user_to_create = user_to_create.into_inner();
    let creator = credentials
        .map(auth::UnverifiedPermissionsCredential::into_inner)
        .map(auth::Credentials::change_level::<auth::perms::CanCreateUser>)
        .transpose()
        .map_err(|_| Status::Unauthorized)?
        .map(|cr| cr.user_id());
    let created = create_account(
        &db,
        creator,
        user_to_create,
    ).map_err(|_| Status::InternalServerError)?;
    // Add token if not already logged in to facilitate credential creation.
    // If a credential is not created in the first session, they will currently need to contact the
    // site admin to log in again.
    if creator.is_none() {
        let key = &tok_key_store
            .get_store()
            .map_err(|_| Status::InternalServerError)?
            .curr;
        let new_credentials = auth::Credentials::<()>::safe_new(created.id, vec![]);
        auth::attach_credentials_token(key, new_credentials, &mut cookies);
    }
    Ok(Status::Ok)
}
/// Creates an account.
///
/// Creates the `user_to_create` in the provided `db`. If the `creator` is not `None`, the
/// created_by field will be set to the creator.
#[must_use]
pub fn create_account(
    db: &db::DB,
    creator: Option<uuid::Uuid>,
    user_to_create: users::NewNoMeta,
) -> Result<users::Data, diesel::result::Error> {
    Ok(db.create_user(
        users::New::from((&user_to_create, creator))
    )?)
}

/// Handlers and functions for managing individual accounts.
pub mod account {
    use super::*;

    /// Handler to get the account info page. Accounts are private only for now -- you can only
    /// view this page if you're logged in as the correct user.
    #[get("/accounts/<id>")]
    pub fn get(
        db: db::DB,
        id: RUuid,
        credentials: auth::UnverifiedPermissionsCredential,
    ) -> Result<Json<users::DataNoMeta>, Status> {
        let id = uuid::Uuid::from_ruuid(id);
        if credentials.user_id() != id {
            return Err(Status::Unauthorized);
        }
        db.find_user_by_id(id)
            .map(users::Data::strip_meta).map(Json)
            .map_err(|_| Status::InternalServerError)
    }
    /// Handler to allow editing of user information if logged in as same user or has permissions
    /// to edit users.
    #[patch("/accounts/<id>", format = "json", data = "<changes>")]
    pub fn patch(
        db: db::DB,
        id: RUuid,
        credentials: auth::UnverifiedPermissionsCredential,
        changes: Json<users::ChangedNoMeta>,
    ) -> Result<Json<users::DataNoMeta>, Status> {
        let id = uuid::Uuid::from_ruuid(id);
        let changes = changes.into_inner();
        let updater = credentials
            .into_inner()
            .change_level::<auth::perms::CanEditUser>()
            .map(|cr| cr.user_id())
            .or_else(|cr| if id == cr.user_id() {
                Ok(cr.user_id())
            } else {
                Err(Status::Unauthorized)
            })?;
        let changes = (&changes, Some(updater)).into();
        db.update_user_by_id(id, changes)
            .map(users::Data::strip_meta).map(Json)
            .map_err(|_| Status::InternalServerError)
    }
    /// Handler to allow for the deletion of accounts if logged in as same user or has permissions
    /// to delete users.
    #[delete("/accounts/<id>")]
    pub fn delete(
        db: db::DB,
        id: RUuid,
        credentials: auth::UnverifiedPermissionsCredential,
    ) -> Result<Status, Status> {
        let id = uuid::Uuid::from_ruuid(id);
        credentials
            .into_inner()
            .change_level::<auth::perms::CanDeleteUser>()
            .map(|cr| cr.user_id())
            .or_else(|cr| if id == cr.user_id() {
                Ok(cr.user_id())
            } else {
                Err(Status::Unauthorized)
            })?;
        db.delete_user_by_id(id)
            .map(|_| Status::Ok)
            .map_err(|_| Status::InternalServerError)
    }
}

