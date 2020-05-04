//! A collection of types related to the capabilities, which belong to an user.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "diesel")]
use crate::schema::*;

/// Data representing a complete row in the table.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(
    feature = "diesel",
    derive(Identifiable, Associations, Queryable),
    belongs_to(parent = "crate::models::users::Data", foreign_key = "user_id"),
    table_name = "capabilities"
)]
pub struct Data {
    /// The id of the row.
    pub id: uuid::Uuid,
    /// The time this row was created.
    pub created_at: DateTime<Utc>,
    /// The creator of this record.
    pub created_by: Option<uuid::Uuid>,
    /// The id of the user the capability belongs to.
    pub user_id: uuid::Uuid,
    /// The capability represented by the row.
    pub capability: String,
}

/// Data representing a new capability, but with an id. This is a convenience struct so that the
/// user does not need to create an id manually.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "diesel", derive(Insertable), table_name = "capabilities")]
pub struct NewWithId<'a> {
    /// The id of the row being inserted.
    id: uuid::Uuid,
    /// The creator of the row.
    created_by: uuid::Uuid,
    /// The id of the user the capability belongs to.
    user_id: uuid::Uuid,
    /// The capability represented by this record.
    capability: &'a str,
}
#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "server")]
impl<'a> From<New<'a>> for NewWithId<'a> {
    fn from(new: New<'a>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            created_by: new.created_by,
            user_id: new.user_id,
            capability: new.capability,
        }
    }
}

/// Represents a new capability for a specific user.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct New<'a> {
    /// The creator of this capability.
    pub created_by: uuid::Uuid,
    /// The id of the capability owner.
    pub user_id: uuid::Uuid,
    /// The capability category itself.
    pub capability: &'a str,
}
