//! A collection of types related to the permissions, which belong to an user.

use crate::{models, schema::*};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Data representing a complete row in the table.
#[derive(Identifiable, Associations, Queryable, Serialize, Deserialize)]
#[belongs_to(parent = "models::users::Data", foreign_key = "user_id")]
#[table_name = "permissions"]
pub struct Data {
    /// The id of the row.
    pub id: uuid::Uuid,
    /// The time this row was created.
    pub created_at: DateTime<Utc>,
    /// The creator of this record.
    pub created_by: Option<uuid::Uuid>,
    /// The id of the user the permission belongs to.
    pub user_id: uuid::Uuid,
    /// The permission represented by the row.
    pub permission: String,
}

/// Data representing a new permission, but with an id. This is a convenience struct so that the
/// user does not need to create an id manually.
#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "permissions"]
pub struct NewWithId<'a> {
    /// The id of the row being inserted.
    id: uuid::Uuid,
    /// The creator of the row.
    created_by: uuid::Uuid,
    /// The id of the user the permission belongs to.
    user_id: uuid::Uuid,
    /// The permission represented by this record.
    permission: &'a str,
}
impl<'a> From<New<'a>> for NewWithId<'a> {
    fn from(new: New<'a>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            created_by: new.created_by,
            user_id: new.user_id,
            permission: new.permission,
        }
    }
}

/// Represents a new permission for a specific user.
#[derive(Serialize, Deserialize)]
pub struct New<'a> {
    /// The creator of this permission.
    pub created_by: uuid::Uuid,
    /// The id of the permission owner.
    pub user_id: uuid::Uuid,
    /// The permission category itself.
    pub permission: &'a str,
}
