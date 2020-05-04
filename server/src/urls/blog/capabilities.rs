//! Handlers and functions for utilizing the capabilities system of the blog.

mod error;
use error::Error;
mod data;

use rocket::http::Status;
use rocket_contrib::{json::Json, uuid::Uuid as RUuid};

use crate::util::{
    auth,
    blog::{db::CapabilityQuery, DB},
    uuid_compat::ruuid_to_uuid,
};
use blog_db::models::*;

/// Checks if capabilities allows for creation of requested capabilities.
///
/// Only allows for requested capabilities to be created if the user logged in has all the requested
/// capabilities as well as [`GrantCapabilities`](crate::blog::auth::caps::GrantCapability).
pub fn validate_and_create_all(
    db: &DB,
    capabilities: auth::Capabilities<auth::caps::GrantCapability>,
    target_user_id: uuid::Uuid,
    capabilities_to_create: Json<Vec<auth::Capability>>,
) -> Result<Vec<capabilities::Data>, Error> {
    if !capabilities.has_capabilities(capabilities_to_create.as_slice()) {
        return Err(Error::Unauthorized);
    }
    let capabilities_to_create = capabilities_to_create
        .iter()
        .map(|p| capabilities::New {
            created_by: capabilities.user_id(),
            user_id: target_user_id,
            capability: p.as_str(),
        })
        .collect();
    Ok(db.create_all_capabilities(capabilities_to_create)?)
}
/// Create a list of capabilities. Requires caller to have the
/// [`GrantCapability`](crate::blog::auth::caps::GrantCapability) capability as well as any
/// capabilities they wish to grant.
#[post(
    "/capabilities/<target_user_id>",
    format = "json",
    data = "<capabilities_to_create>"
)]
pub fn post(
    db: DB,
    capabilities: auth::Capabilities<auth::caps::GrantCapability>,
    target_user_id: RUuid,
    capabilities_to_create: Json<Vec<auth::Capability>>,
) -> Status {
    let target_user_id = ruuid_to_uuid(target_user_id);
    validate_and_create_all(&db, capabilities, target_user_id, capabilities_to_create)
        .map_or_else(|e| e.into(), |_| Status::Ok)
}

/// Deletes capabilities satisfying the provided [`Query`](crate::blog::capabilities::data::Query).
/// Requires caller to have the
/// [`DeleteCapability`](crate::blog::auth::caps::DeleteCapability) capability.
#[delete("/capabilities", format = "json", data = "<to_delete>")]
pub fn delete(
    db: DB,
    _capabilities: auth::Capabilities<auth::caps::DeleteCapability>,
    to_delete: Json<data::Query>,
) -> Result<Json<Vec<capabilities::Data>>, Status> {
    let to_delete = to_delete.into_inner();
    let capabilities = vec![
        to_delete
            .user_id()
            .map(|id| db.delete_capabilities_by_user_id(id))
            .transpose()
            .map_err(Error::from)?
            .unwrap_or_else(Vec::new),
        to_delete
            .capability_ids()
            .map(|id| db.delete_capabilities_with_ids(id))
            .transpose()
            .map_err(Error::from)?
            .unwrap_or_else(Vec::new),
    ]
    .into_iter()
    .flatten()
    .collect();
    Ok(Json(capabilities))
}

/// Handlers and functions for managing individual capabilities.
pub mod capability {
    use rocket::http::Status;
    use rocket_contrib::{json::Json, uuid::Uuid as RUuid};

    use crate::{
        urls::blog::capabilities::Error,
        util::{
            auth,
            blog::{db::CapabilityQuery, DB},
            uuid_compat::ruuid_to_uuid,
        },
    };
    use blog_db::models::*;

    /// Gets the capability with the requested id. Requires caller to have the
    /// [`ViewCapability`](crate::blog::auth::caps::ViewCapability`) capability.
    #[get("/capabilities/<id>")]
    pub fn get(
        db: DB,
        _capabilities: auth::Capabilities<auth::caps::ViewCapability>,
        id: RUuid,
    ) -> Result<Json<capabilities::Data>, Status> {
        let id = ruuid_to_uuid(id);
        db.get_capability_with_id(id)
            .map(Json)
            .map_err(|e| Error::from(e).into())
    }
    /// Deletes the capability with the requested id. Requires caller to have the
    /// [`DeleteCapability`](crate::blog::auth::caps::DeleteCapability`) capability.
    #[delete("/capabilities/<id>")]
    pub fn delete(
        db: DB,
        _capabilities: auth::Capabilities<auth::caps::DeleteCapability>,
        id: RUuid,
    ) -> Result<Json<capabilities::Data>, Status> {
        let id = ruuid_to_uuid(id);
        db.delete_capability_with_id(id)
            .map(Json)
            .map_err(|e| Error::from(e).into())
    }
}
