use crate::{schema::*, models::option_datefmt};
use serde::{
    Serialize,
    Deserialize,
};
use chrono::{DateTime, Utc};

#[derive(Identifiable, Queryable, Serialize, Deserialize)]
#[table_name="posts"]
pub struct Data {
    pub id: uuid::Uuid,
    pub created_at: DateTime<Utc>,
    pub created_by: uuid::Uuid,
    pub updated_at: DateTime<Utc>,
    pub updated_by: uuid::Uuid,
    pub published_at: Option<DateTime<Utc>>,
    pub published_by: Option<uuid::Uuid>,
    pub archived_at: Option<DateTime<Utc>>,
    pub archived_by: Option<uuid::Uuid>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<uuid::Uuid>,
    pub title: String,
    pub body: String,
}

#[derive(Identifiable, Insertable, Serialize, Deserialize)]
#[table_name="posts"]
pub struct NewWithId<'a> {
    id: uuid::Uuid,
    title: &'a str,
    body: &'a str,
}
impl<'a> From<New<'a>> for NewWithId<'a> {
    fn from(new: New<'a>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            title: new.title,
            body: new.body,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct New<'a> {
    pub title: &'a str,
    pub body: &'a str,
}

#[derive(AsChangeset, Serialize, Deserialize)]
#[table_name="posts"]
pub struct Changed<'a> {
    pub title: Option<&'a str>,
    pub body: Option<&'a str>,
    #[serde(default, deserialize_with = "option_datefmt")]
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(AsChangeset, Serialize, Deserialize)]
#[table_name="posts"]
pub struct Publishing {
    published_at: DateTime<Utc>,
    published_by: uuid::Uuid,
}
impl Publishing {
    pub fn new(published_by: uuid::Uuid) -> Self {
        Self {
            published_at: Utc::now(),
            published_by: published_by,
        }
    }
}

#[derive(AsChangeset, Serialize, Deserialize)]
#[table_name="posts"]
pub struct Archival {
    archived_at: DateTime<Utc>,
    archived_by: uuid::Uuid,
}
impl Archival {
    pub fn new(archived_by: uuid::Uuid) -> Self {
        Self {
            archived_at: Utc::now(),
            archived_by: archived_by,
        }
    }
}

