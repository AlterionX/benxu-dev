pub mod error;

use std::{
    sync::Arc,
};
use rocket::{
    response::status,
    http::{Status, Cookies},
    State,
};
use rocket_contrib::{
    json::Json,
    uuid::Uuid as RUuid,
};

use blog_db::models::*;
use crate::{
    uuid_conv::FromRUuid,
    blog::{
        db,
        auth,
    },
    crypto::{
        KeyStore,
        token::paseto,
    },
};

#[post("/accounts", format = "json", data = "<user_to_create>")]
pub fn post(
    db: db::DB,
    mut cookies: Cookies,
    tok_key_store: State<Arc<KeyStore<paseto::v2::local::Algo>>>,
    credentials: Option<auth::UnverifiedPermissionsCredential>,
    user_to_create: Json<users::NewNoMeta>,
) -> status::Custom<()> {
    let user_to_create = user_to_create.into_inner();
    let created = match create_account(
        &db,
        credentials.as_ref().map(|cr| cr.user_id()),
        user_to_create,
    ) {
        Ok(u) => u,
        Err(_) => return status::Custom(Status::InternalServerError , ()),
    };
    if credentials.is_none() {
        let key = &match tok_key_store.curr_and_last() {
            Ok(key) => key,
            Err(_) => return status::Custom(Status::InternalServerError, ()),
        }.curr;
        let new_credentials = auth::Credentials::<()>::safe_new(created.id, vec![]);
        auth::attach_credentials_token(key, new_credentials, &mut cookies);
    }
    status::Custom(Status::Ok, ())
}
#[must_use]
pub fn create_account(
    db: &db::DB,
    creator: Option<uuid::Uuid>,
    user_to_create: users::NewNoMeta,
) -> Result<users::Data, error::Create> {
    Ok(db.create_user(
        users::New::from((&user_to_create, creator))
    )?)
}

pub mod account {
    use super::*;

    #[get("/accounts/<id>")]
    pub fn get(db: db::DB, id: RUuid) -> Result<Json<users::DataNoMeta>, status::Custom<()>> {
        let id = uuid::Uuid::from_ruuid(id);
        db.find_user_by_id(id)
            .map(users::Data::strip_meta).map(Json)
            .map_err(|_| status::Custom(Status::InternalServerError, ()))
    }
    #[patch("/accounts/<id>", format = "json", data = "<changes>")]
    pub fn patch(
        db: db::DB,
        id: RUuid,
        credentials: auth::UnverifiedPermissionsCredential,
        changes: Json<users::ChangedNoMeta>,
    ) -> Result<Json<users::DataNoMeta>, status::Custom<()>> {
        let id = uuid::Uuid::from_ruuid(id);
        let changes = changes.into_inner();
        let updater = match credentials.into_inner().change_level::<auth::perms::CanEditUser>() {
            Ok(cr) => cr.user_id(),
            Err(cr) => if id == cr.user_id() {
                cr.user_id()
            } else {
                return Err(status::Custom(Status::Unauthorized, ()));
            },
        };
        let changes = (&changes, Some(updater)).into();
        db.update_user_by_id(id, changes)
            .map(users::Data::strip_meta).map(Json)
            .map_err(|_| status::Custom(Status::InternalServerError, ()))
    }
    #[delete("/accounts/<id>")]
    pub fn delete(db: db::DB, id: RUuid) -> status::Custom<()> {
        let id = uuid::Uuid::from_ruuid(id);
        match db.delete_user_by_id(id) {
            Ok(_) => status::Custom(Status::Ok, ()),
            Err(_) => status::Custom(Status::InternalServerError, ()),
        }
    }
}

