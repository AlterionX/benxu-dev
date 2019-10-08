//! Models used for querying user data.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "diesel")]
use crate::schema::*;

/// Data representing a complete row in the table.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(
    feature = "diesel",
    derive(Identifiable, Queryable),
    table_name = "users"
)]
pub struct Data {
    /// Id of the record.
    pub id: uuid::Uuid,
    /// User's user name.
    pub user_name: String,
    /// Time when user was created.
    pub created_at: DateTime<Utc>,
    /// User id of person who created the user.
    pub created_by: Option<uuid::Uuid>,
    /// Time when user was last updated.
    pub updated_at: DateTime<Utc>,
    /// User id of person who last updated the user.
    pub updated_by: Option<uuid::Uuid>,
    /// Optional first name.
    pub first_name: Option<String>,
    /// Optional last name.
    pub last_name: Option<String>,
    /// Optional email.
    pub email: Option<String>,
}
impl Data {
    /// Strip meta data from user to send back to client.
    pub fn strip_meta(self) -> DataNoMeta {
        DataNoMeta::from(self)
    }
}

/// Data representing the user building_clocks()
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DataNoMeta {
    /// Id of the record.
    pub id: uuid::Uuid,
    /// User's user name.
    pub user_name: String,
    /// Time when user was created.
    pub created_at: DateTime<Utc>,
    /// User id of person who created the user.
    pub created_by: Option<uuid::Uuid>,
    /// Time when user was last updated.
    pub updated_at: DateTime<Utc>,
    /// User id of person who last updated the user.
    pub updated_by: Option<uuid::Uuid>,
    /// Optional first name.
    pub first_name: Option<String>,
    /// Optional last name.
    pub last_name: Option<String>,
    /// Optional email.
    pub email: Option<String>,
}
impl From<Data> for DataNoMeta {
    fn from(data: Data) -> Self {
        Self {
            id: data.id,
            user_name: data.user_name,
            created_at: data.created_at,
            created_by: data.created_by,
            updated_at: data.updated_at,
            updated_by: data.updated_by,
            first_name: data.first_name,
            last_name: data.last_name,
            email: data.email,
        }
    }
}

/// Represents a new user with its id.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "diesel", derive(Insertable), table_name = "users")]
pub struct NewWithId<'a> {
    /// User's email.
    email: &'a str,
    /// Id of the record.
    id: uuid::Uuid,
    /// User's user name.
    user_name: &'a str,
    /// User id of person who created the user.
    created_by: Option<uuid::Uuid>,
    /// User id of person who last updated the user.
    updated_by: Option<uuid::Uuid>,
    /// Optional first name.
    first_name: &'a str,
    /// Optional last name.
    last_name: &'a str,
}
#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "server")]
impl<'a> From<New<'a>> for NewWithId<'a> {
    fn from(new: New<'a>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            user_name: new.user_name,
            created_by: new.created_by,
            updated_by: new.updated_by,
            first_name: new.first_name,
            last_name: new.last_name,
            email: new.email,
        }
    }
}

/// Represents a new user.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct New<'a> {
    /// User's user name.
    pub user_name: &'a str,
    /// User id of person who created the user.
    pub created_by: Option<uuid::Uuid>,
    /// User id of person who last updated the user.
    pub updated_by: Option<uuid::Uuid>,
    /// Optional first name.
    pub first_name: &'a str,
    /// Optional last name.
    pub last_name: &'a str,
    /// User's email.
    pub email: &'a str,
}
impl<'a> From<(&'a NewNoMeta, Option<uuid::Uuid>)> for New<'a> {
    fn from((source, user): (&'a NewNoMeta, Option<uuid::Uuid>)) -> Self {
        Self {
            user_name: &source.user_name,
            created_by: user,
            updated_by: user,
            first_name: &source.first_name,
            last_name: &source.last_name,
            email: &source.email,
        }
    }
}

/// Represents a new user without meta info.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NewNoMeta {
    /// User's user name.
    pub user_name: String,
    /// Optional first name.
    pub first_name: String,
    /// Optional last name.
    pub last_name: String,
    /// User's email.
    pub email: String,
}

/// Represents updates to the data structure.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "diesel", derive(AsChangeset), table_name = "users")]
pub struct Changed<'a> {
    /// The users's user name.
    pub user_name: Option<&'a str>,
    /// The user who most recently retired.
    pub updated_by: Option<uuid::Uuid>,
    /// The first name of the user.
    pub first_name: Option<&'a str>,
    /// The last name of the user.
    pub last_name: Option<&'a str>,
    /// The email of the user.
    pub email: Option<&'a str>,
}
impl<'a> From<(&'a ChangedNoMeta, Option<uuid::Uuid>)> for Changed<'a> {
    fn from((source, updater): (&'a ChangedNoMeta, Option<uuid::Uuid>)) -> Self {
        Self {
            user_name: source.user_name.as_ref().map(String::as_str),
            updated_by: updater,
            first_name: source.first_name.as_ref().map(String::as_str),
            last_name: source.last_name.as_ref().map(String::as_str),
            email: source.email.as_ref().map(String::as_str),
        }
    }
}

/// A list of changes without meta information.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChangedNoMeta {
    /// The users's user name.
    pub user_name: Option<String>,
    /// The first name of the user.
    pub first_name: Option<String>,
    /// The last name of the user.
    pub last_name: Option<String>,
    /// The email of the user.
    pub email: Option<String>,
}
