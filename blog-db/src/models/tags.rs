//! Models used to represent post tags.

use crate::schema::*;
use serde::{Deserialize, Serialize};

/// Data representing a complete row in the table.
#[derive(Identifiable, Queryable, Serialize, Deserialize)]
#[table_name = "tags"]
pub struct Data {
    /// The id of the new record.
    pub id: uuid::Uuid,
    /// The name of the tag.
    pub name: String,
    /// A short description of the tag.
    pub description: String,
}

/// Data to be inserted as a new row in the table. Automatically adds an id to the struct
/// [`New`](crate::models::tags::New).
#[derive(Identifiable, Insertable, Serialize, Deserialize)]
#[table_name = "tags"]
pub struct NewWithId<'a> {
    /// The id of the new record.
    id: uuid::Uuid,
    /// The name of the tag.
    name: &'a str,
    /// A short description of the tag.
    description: &'a str,
    /// The creator of the tag.
    created_by: uuid::Uuid,
}
impl<'a> From<New<'a>> for NewWithId<'a> {
    fn from(new_tag: New<'a>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name: new_tag.name,
            description: new_tag.description,
            created_by: new_tag.created_by,
        }
    }
}

/// A simple new tag record, without the id.
#[derive(Serialize, Deserialize)]
pub struct New<'a> {
    /// The name of the tag.
    pub name: &'a str,
    /// A short description of the tag.
    pub description: &'a str,
    /// The creator of the tag.
    pub created_by: uuid::Uuid,
}

/// An update to the name and description of the tag.
#[derive(AsChangeset, Serialize, Deserialize)]
#[table_name = "tags"]
pub struct Update {
    /// The name of the tag.
    pub name: Option<String>,
    /// A short description of the tag.
    pub description: Option<String>,
}
