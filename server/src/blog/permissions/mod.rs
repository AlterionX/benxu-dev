pub mod error;
pub use error::Error as Error;
pub mod data;

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

fn create_all(db: &db::DB, to_create: Vec<permissions::New>) -> Result<Vec<permissions::Data>, error::Diesel> {
    db.create_all_permissions(to_create)
}
pub fn validate_and_create_all(
    db: &db::DB,
    credentials: auth::Credentials<auth::perms::CanGrantPermission>,
    target_user_id: uuid::Uuid,
    permissions_to_create: Json<Vec<auth::Permission>>,
) -> Result<Vec<permissions::Data>, Error> {
    if !credentials.has_permissions(permissions_to_create.as_slice()) {
        return Err(Error::Unauthorized);
    }
    let permissions_to_create = permissions_to_create.iter().map(|p| {
        permissions::New {
            created_by: credentials.user_id(),
            user_id: target_user_id,
            permission: p.as_str(),
        }
    }).collect();
    Ok(create_all(db, permissions_to_create)?)
}
#[post("/permissions/<target_user_id>", format = "json", data = "<permissions_to_create>")]
pub fn post(
    db: db::DB,
    mut cookies: Cookies,
    tok_key_store: State<Arc<KeyStore<paseto::v2::local::Algo>>>,
    credentials: auth::Credentials<auth::perms::CanGrantPermission>,
    target_user_id: RUuid,
    permissions_to_create: Json<Vec<auth::Permission>>,
) -> status::Custom<()> {
    let target_user_id = uuid::Uuid::from_ruuid(target_user_id);
    match validate_and_create_all(
        &db,
        credentials,
        target_user_id,
        permissions_to_create,
    ) {
        Ok(_) => status::Custom(Status::Ok, ()),
        Err(e) => e.into(),
    }
}

#[delete("/permissions", format = "json", data = "<to_delete>")]
pub fn delete(
    db: db::DB,
    to_delete: Json<data::Query>,
    credentials: auth::Credentials<auth::perms::CanDeletePermission>,
) -> Result<Json<Vec<permissions::Data>>, status::Custom<()>> {
    let to_delete = to_delete.into_inner();
    let mut permissions = if let Some(user_id) = to_delete.user_id() {
        db.delete_permissions_by_user_id(user_id)
            .map_err(|e| e.into(): Error)?
    } else {
        vec![]
    };
    permissions.append(&mut if let Some(permission_ids) = to_delete.permission_ids() {
        db.delete_permissions_with_ids(permission_ids)
            .map_err(|e| e.into(): Error)?
    } else {
        vec![]
    });
    Ok(Json(permissions))
}

pub mod permission {
    use rocket::response::status;
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
            permissions::Error,
        },
    };

    #[get("/permissions/<id>")]
    pub fn get(
        db: db::DB,
        id: RUuid,
        credentials: auth::Credentials<auth::perms::CanViewPermission>,
    ) -> Result<Json<permissions::Data>, status::Custom<()>> {
        db.get_permission_with_id(uuid::Uuid::from_ruuid(id))
            .map(|p| Json(p))
            .map_err(|e| (e.into(): Error).into())
    }
    #[delete("/permissions/<id>")]
    pub fn delete(
        db: db::DB,
        id: RUuid,
        credentials: auth::Credentials<auth::perms::CanDeletePermission>,
    ) -> Result<Json<permissions::Data>, status::Custom<()>> {
        db.delete_permission_with_id(uuid::Uuid::from_ruuid(id))
            .map(|p| Json(p))
            .map_err(|e| (e.into(): Error).into())
    }
}

