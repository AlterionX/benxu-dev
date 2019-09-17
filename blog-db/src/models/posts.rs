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
impl Data {
    pub fn strip_meta(self) -> DataNoMeta {
        self.into()
    }
}

#[derive(Serialize, Deserialize)]
pub struct DataNoMeta {
    pub published_at: Option<DateTime<Utc>>,
    pub published_by: Option<uuid::Uuid>,
    pub archived_at: Option<DateTime<Utc>>,
    pub archived_by: Option<uuid::Uuid>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<uuid::Uuid>,
    pub title: String,
    pub body: String,
}
impl From<Data> for DataNoMeta {
    fn from(d: Data) -> Self {
        Self {
            published_at: d.published_at,
            published_by: d.published_by,
            archived_at: d.archived_at,
            archived_by: d.archived_by,
            deleted_at: d.deleted_at,
            deleted_by: d.deleted_by,
            title: d.title,
            body: d.body,
        }
    }
}

#[derive(Identifiable, Insertable, Serialize, Deserialize)]
#[table_name="posts"]
pub struct NewWithId<'a> {
    id: uuid::Uuid,
    created_by: uuid::Uuid,
    updated_by: uuid::Uuid,

    published_at: Option<DateTime<Utc>>,
    published_by: Option<uuid::Uuid>,
    archived_at: Option<DateTime<Utc>>,
    archived_by: Option<uuid::Uuid>,
    deleted_at: Option<DateTime<Utc>>,
    deleted_by: Option<uuid::Uuid>,

    title: &'a str,
    body: &'a str,
}
impl<'a> From<New<'a>> for NewWithId<'a> {
    fn from(new: New<'a>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            created_by: new.created_by,
            updated_by: new.updated_by,

            published_at: new.published_at,
            published_by: new.published_by,
            archived_at: new.archived_at,
            archived_by: new.archived_by,
            deleted_at: new.deleted_at,
            deleted_by: new.deleted_by,

            title: new.title,
            body: new.body,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct New<'a> {
    pub created_by: uuid::Uuid,
    pub updated_by: uuid::Uuid,

    pub published_at: Option<DateTime<Utc>>,
    pub published_by: Option<uuid::Uuid>,
    pub archived_at: Option<DateTime<Utc>>,
    pub archived_by: Option<uuid::Uuid>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<uuid::Uuid>,

    pub title: &'a str,
    pub body: &'a str,
}
impl<'a> From<(&'a NewNoMeta, uuid::Uuid)> for New<'a> {
    fn from((reference, creator): (&'a NewNoMeta, uuid::Uuid)) -> Self {
        Self {
            created_by: creator,
            updated_by: creator,

            published_at: reference.published_at,
            published_by: reference.published_by,
            archived_at: reference.archived_at,
            archived_by: reference.archived_by,
            deleted_at: reference.deleted_at,
            deleted_by: reference.deleted_by,

            title: reference.title.as_str(),
            body: reference.body.as_str(),
        }
    }
}
impl<'a> From<(&'a NewNoMeta, uuid::Uuid)> for NewWithId<'a> {
    fn from(conv: (&'a NewNoMeta, uuid::Uuid)) -> Self {
        (conv.into(): New).into()
    }
}

#[derive(Serialize, Deserialize)]
pub struct NewNoMeta {
    pub published_at: Option<DateTime<Utc>>,
    pub published_by: Option<uuid::Uuid>,
    pub archived_at: Option<DateTime<Utc>>,
    pub archived_by: Option<uuid::Uuid>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<uuid::Uuid>,

    pub title: String,
    pub body: String,
}
impl NewNoMeta {
    pub fn new_with_no_flags(title: String, body: String) -> Self {
        Self {
            published_at: None,
            published_by: None,
            archived_at: None,
            archived_by: None,
            deleted_at: None,
            deleted_by: None,

            title: title,
            body: body,
        }
    }
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
pub struct Editing {
    updated_by: uuid::Uuid,
}
impl Editing {
    pub fn new(updated_by: uuid::Uuid) -> Self {
        Self {
            updated_by: updated_by,
        }
    }
}

#[derive(AsChangeset, Serialize, Deserialize)]
#[table_name="posts"]
pub struct Publishing {
    updated_by: uuid::Uuid,
}
impl Publishing {
    pub fn new(updated_by: uuid::Uuid) -> Self {
        Self {
            updated_by: updated_by,
        }
    }
}

#[derive(AsChangeset, Serialize, Deserialize)]
#[table_name="posts"]
pub struct Archival {
    updated_by: uuid::Uuid,
    archived_at: DateTime<Utc>,
    archived_by: uuid::Uuid,
}
impl Archival {
    pub fn new(archived_by: uuid::Uuid) -> Self {
        Self {
            updated_by: archived_by,
            archived_at: Utc::now(),
            archived_by: archived_by,
        }
    }
}

#[derive(AsChangeset, Serialize, Deserialize)]
#[table_name="posts"]
pub struct Deletion {
    updated_by: uuid::Uuid,
    deleted_at: DateTime<Utc>,
    deleted_by: uuid::Uuid,
}
impl Deletion {
    pub fn new(deleted_by: uuid::Uuid) -> Self {
        Self {
            updated_by: deleted_by,
            deleted_at: Utc::now(),
            deleted_by: deleted_by,
        }
    }
}
