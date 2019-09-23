//! Handlers and functions for utilizing the permissions system of the blog.

pub mod error;
use error::Error;
pub mod data;

use rocket::http::Status;
use rocket_contrib::{json::Json, uuid::Uuid as RUuid};

use crate::blog::{auth, db};
use blog_db::models::*;

/// Checks if credentials allows for creation of requested permissions.
///
/// Only allows for requested permissions to be created if the user logged in has all the requested
/// permissions as well as [`CanGrantPermissions`](crate::blog::auth::perms::CanGrantPermission).
pub fn validate_and_create_all(
    db: &db::DB,
    credentials: auth::Credentials<auth::perms::CanGrantPermission>,
    target_user_id: uuid::Uuid,
    permissions_to_create: Json<Vec<auth::Permission>>,
) -> Result<Vec<permissions::Data>, Error> {
    if !credentials.has_permissions(permissions_to_create.as_slice()) {
        return Err(Error::Unauthorized);
    }
    let permissions_to_create = permissions_to_create
        .iter()
        .map(|p| permissions::New {
            created_by: credentials.user_id(),
            user_id: target_user_id,
            permission: p.as_str(),
        })
        .collect();
    Ok(db.create_all_permissions(permissions_to_create)?)
}
/// Create a list of credentials. Requires caller to have the
/// [`CanGrantPermission`](crate::blog::auth::perms::CanGrantPermission) permission as well as any
/// permissions they wish to grant.
#[post(
    "/permissions/<target_user_id>",
    format = "json",
    data = "<permissions_to_create>"
)]
pub fn post(
    db: db::DB,
    credentials: auth::Credentials<auth::perms::CanGrantPermission>,
    target_user_id: RUuid,
    permissions_to_create: Json<Vec<auth::Permission>>,
) -> Status {
    let target_user_id = target_user_id.into_inner();
    validate_and_create_all(&db, credentials, target_user_id, permissions_to_create)
        .map_or_else(|e| e.into(), |_| Status::Ok)
}

/// Deletes credentials satisfying the provided [`Query`](crate::blog::permissions::data::Query).
/// Requires caller to have the
/// [`CanDeletePermission`](crate::blog::auth::perms::CanDeletePermission) permission.
#[delete("/permissions", format = "json", data = "<to_delete>")]
pub fn delete(
    db: db::DB,
    _credentials: auth::Credentials<auth::perms::CanDeletePermission>,
    to_delete: Json<data::Query>,
) -> Result<Json<Vec<permissions::Data>>, Status> {
    let to_delete = to_delete.into_inner();
    let permissions = vec![
        to_delete
            .user_id()
            .map(|id| db.delete_permissions_by_user_id(id))
            .transpose()
            .map_err(Error::from)?
            .unwrap_or(vec![]),
        to_delete
            .permission_ids()
            .map(|id| db.delete_permissions_with_ids(id))
            .transpose()
            .map_err(Error::from)?
            .unwrap_or(vec![]),
    ]
    .into_iter()
    .flatten()
    .collect();
    Ok(Json(permissions))
}

/// Handlers and functions for managing individual permissions.
pub mod permission {
    use rocket::http::Status;
    use rocket_contrib::{json::Json, uuid::Uuid as RUuid};

    use crate::blog::{auth, db, permissions::Error};
    use blog_db::models::*;

    /// Gets the permission with the requested id. Requires caller to have the
    /// [`CanViewPermission`](crate::blog::auth::perms::CanViewPermission`) permission.
    #[get("/permissions/<id>")]
    pub fn get(
        db: db::DB,
        _credentials: auth::Credentials<auth::perms::CanViewPermission>,
        id: RUuid,
    ) -> Result<Json<permissions::Data>, Status> {
        db.get_permission_with_id(id.into_inner())
            .map(|p| Json(p))
            .map_err(|e| (e.into(): Error).into())
    }
    /// Deletes the permission with the requested id. Requires caller to have the
    /// [`CanDeletePermission`](crate::blog::auth::perms::CanDeletePermission`) permission.
    #[delete("/permissions/<id>")]
    pub fn delete(
        db: db::DB,
        _credentials: auth::Credentials<auth::perms::CanDeletePermission>,
        id: RUuid,
    ) -> Result<Json<permissions::Data>, Status> {
        db.delete_permission_with_id(id.into_inner())
            .map(|p| Json(p))
            .map_err(|e| (e.into(): Error).into())
    }
}
