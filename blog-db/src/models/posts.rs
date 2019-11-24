//! Models representing different aspects of posts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "diesel")]
use crate::schema::*;

/// Data representing a complete row in the table.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(
    feature = "diesel",
    derive(Identifiable, Queryable),
    table_name = "posts"
)]
pub struct Data {
    /// The id of the record.
    pub id: uuid::Uuid,
    /// The time at which the record was created.
    pub created_at: DateTime<Utc>,
    /// The id of the user who created the record.
    pub created_by: uuid::Uuid,
    /// The time at which the record was last updated.
    pub updated_at: DateTime<Utc>,
    /// The id of the user who last updated the record.
    pub updated_by: uuid::Uuid,
    /// The time at which the record was published. [`None`] means that the record has not been
    /// published.
    pub published_at: Option<DateTime<Utc>>,
    /// The id of the user who published the record. [`None`] means that the record has not been
    /// published.
    pub published_by: Option<uuid::Uuid>,
    /// The time at which the record was archived. [`None`] means that the record has not been
    /// archived.
    pub archived_at: Option<DateTime<Utc>>,
    /// The id of the user who archived the record. [`None`] means that the record has not been
    /// archived.
    pub archived_by: Option<uuid::Uuid>,
    /// The time at which the record was deleted. [`None`] means that the record has not been
    /// deleted.
    pub deleted_at: Option<DateTime<Utc>>,
    /// The id of the user who "deleted" the record. [`None`] means that the record has not been
    /// deleted.
    pub deleted_by: Option<uuid::Uuid>,
    /// The title of the blog post.
    pub title: String,
    /// The body of the blog post.
    pub body: String,
    /// Friendly name for the blog post.
    pub slug: Option<String>,
}
impl Data {
    /// Strips the meta data before sending it to a client.
    pub fn strip_meta(self) -> DataNoMeta {
        self.into()
    }
}

/// Almost the same as [`Data`](crate::models::posts::Data) but without the id, created, and
/// updated information.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DataNoMeta {
    /// The id of the record.
    pub id: uuid::Uuid,
    /// The time at which the record was created.
    pub created_at: DateTime<Utc>,
    /// The id of the user who created the record.
    pub created_by: uuid::Uuid,
    /// The time at which the record was published. [`None`] means that the record has not been
    /// published.
    pub published_at: Option<DateTime<Utc>>,
    /// The id of the user who published the record. [`None`] means that the record has not been
    /// published.
    pub published_by: Option<uuid::Uuid>,
    /// The time at which the record was archived. [`None`] means that the record has not been
    /// archived.
    pub archived_at: Option<DateTime<Utc>>,
    /// The id of the user who archived the record. [`None`] means that the record has not been
    /// archived.
    pub archived_by: Option<uuid::Uuid>,
    /// The time at which the record was deleted. [`None`] means that the record has not been
    /// deleted.
    pub deleted_at: Option<DateTime<Utc>>,
    /// The id of the user who "deleted" the record. [`None`] means that the record has not been
    /// deleted.
    pub deleted_by: Option<uuid::Uuid>,
    /// The title of the blog post.
    pub title: String,
    /// The body of the blog post.
    pub body: String,
    /// Friendly name for the blog post.
    pub slug: Option<String>,
}
impl From<Data> for DataNoMeta {
    fn from(d: Data) -> Self {
        Self {
            id: d.id,
            created_at: d.created_at,
            created_by: d.created_by,
            published_at: d.published_at,
            published_by: d.published_by,
            archived_at: d.archived_at,
            archived_by: d.archived_by,
            deleted_at: d.deleted_at,
            deleted_by: d.deleted_by,
            title: d.title,
            body: d.body,
            slug: d.slug,
        }
    }
}
impl DataNoMeta {
    pub fn is_published(&self) -> bool {
        self.published_at.is_some() && self.archived_at.is_none() && self.deleted_at.is_none()
    }
}

/// Data representing the id, title, the publishing time, and author of a post.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(
    feature = "diesel",
    derive(Identifiable, Queryable),
    table_name = "posts"
)]
pub struct BasicData {
    /// The id of the record.
    pub id: uuid::Uuid,
    /// The time at which the record was created.
    pub created_at: DateTime<Utc>,
    /// The time at which the record was published. [`None`] means that the record has not been
    /// published.
    pub published_at: Option<DateTime<Utc>>,
    /// The time at which the record was archived. [`None`] means that the record has not been
    /// archived.
    pub archived_at: Option<DateTime<Utc>>,
    /// The time at which the record was deleted. [`None`] means that the record has not been
    /// deleted.
    pub deleted_at: Option<DateTime<Utc>>,
    /// The title of the blog post.
    pub title: String,
    /// The body of the blog post.
    pub body: String,
    /// Friendly name for the blog post.
    pub slug: Option<String>,
}
impl BasicData {
    pub fn is_published(&self) -> bool {
        self.published_at.is_some() && self.archived_at.is_none() && self.deleted_at.is_none()
    }
}
#[cfg(feature = "diesel")]
impl BasicData {
    pub const COLUMNS: (
        posts::id,
        posts::created_at,
        posts::published_at,
        posts::archived_at,
        posts::deleted_at,
        posts::title,
        posts::body,
        posts::slug,
    ) = (
        posts::id,
        posts::created_at,
        posts::published_at,
        posts::archived_at,
        posts::deleted_at,
        posts::title,
        posts::body,
        posts::slug,
    );
}

/// Represents a new post.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(
    feature = "diesel",
    derive(Identifiable, Insertable),
    table_name = "posts"
)]
pub struct NewWithId<'a> {
    /// The id of the record.
    id: uuid::Uuid,
    /// The id of the user who created the record.
    created_by: uuid::Uuid,
    /// The id of the user who last updated the record.
    updated_by: uuid::Uuid,

    /// The time at which the record was published. [`None`] means that the record has not been
    /// published.
    published_at: Option<DateTime<Utc>>,
    /// The id of the user who published the record. [`None`] means that the record has not been
    /// published.
    published_by: Option<uuid::Uuid>,
    /// The time at which the record was archived. [`None`] means that the record has not been
    /// archived.
    archived_at: Option<DateTime<Utc>>,
    /// The id of the user who archived the record. [`None`] means that the record has not been
    /// archived.
    archived_by: Option<uuid::Uuid>,
    /// The time at which the record was deleted. [`None`] means that the record has not been
    /// deleted.
    deleted_at: Option<DateTime<Utc>>,
    /// The id of the user who "deleted" the record. [`None`] means that the record has not been
    /// deleted.
    deleted_by: Option<uuid::Uuid>,

    /// The title of the blog post.
    title: &'a str,
    /// The body of the blog post.
    body: &'a str,
    /// The friendly name for the blog post.
    slug: Option<&'a str>,
}
#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "server")]
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
            slug: new.slug,
        }
    }
}

/// Represents a new post without an id.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct New<'a> {
    /// The id of the user who created the record.
    pub created_by: uuid::Uuid,
    /// The id of the user who last updated the record.
    pub updated_by: uuid::Uuid,

    /// The time at which the record was published. [`None`] means that the record has not been
    /// published.
    pub published_at: Option<DateTime<Utc>>,
    /// The id of the user who published the record. [`None`] means that the record has not been
    /// published.
    pub published_by: Option<uuid::Uuid>,
    /// The time at which the record was archived. [`None`] means that the record has not been
    /// archived.
    pub archived_at: Option<DateTime<Utc>>,
    /// The id of the user who archived the record. [`None`] means that the record has not been
    /// archived.
    pub archived_by: Option<uuid::Uuid>,
    /// The time at which the record was deleted. [`None`] means that the record has not been
    /// deleted.
    pub deleted_at: Option<DateTime<Utc>>,
    /// The id of the user who "deleted" the record. [`None`] means that the record has not been
    /// deleted.
    pub deleted_by: Option<uuid::Uuid>,

    /// The title of the blog post.
    pub title: &'a str,
    /// The body of the blog post.
    pub body: &'a str,
    /// The friendly name for the blog post.
    pub slug: Option<&'a str>,
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
            slug: reference.slug.as_ref().map(String::as_str),
        }
    }
}
#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "server")]
impl<'a> From<(&'a NewNoMeta, uuid::Uuid)> for NewWithId<'a> {
    fn from(conv: (&'a NewNoMeta, uuid::Uuid)) -> Self {
        (conv.into(): New).into()
    }
}

/// Represents a new post without an id as well as the created by and updated by fields.
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NewNoMeta {
    /// The time at which the record was published. [`None`] means that the record has not been
    /// published.
    pub published_at: Option<DateTime<Utc>>,
    /// The id of the user who published the record. [`None`] means that the record has not been
    /// published.
    pub published_by: Option<uuid::Uuid>,
    /// The time at which the record was archived. [`None`] means that the record has not been
    /// archived.
    pub archived_at: Option<DateTime<Utc>>,
    /// The id of the user who archived the record. [`None`] means that the record has not been
    /// archived.
    pub archived_by: Option<uuid::Uuid>,
    /// The time at which the record was deleted. [`None`] means that the record has not been
    /// deleted.
    pub deleted_at: Option<DateTime<Utc>>,
    /// The id of the user who "deleted" the record. [`None`] means that the record has not been
    /// deleted.
    pub deleted_by: Option<uuid::Uuid>,

    /// The title of the blog post.
    pub title: String,
    /// The body of the blog post.
    pub body: String,
    /// The friendly name for the blog post.
    pub slug: Option<String>,
}
impl NewNoMeta {
    /// Default everything other than the title and body to [`None`].
    pub fn new_with_no_flags(title: String, body: String) -> Self {
        Self {
            published_at: None,
            published_by: None,
            archived_at: None,
            archived_by: None,
            deleted_at: None,
            deleted_by: None,

            title,
            body,
            slug: None,
        }
    }
}

/// Struct representing changes to the body and title of the post.
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "diesel", derive(AsChangeset), table_name = "posts")]
pub struct Changed {
    /// The title of the blog post.
    pub title: Option<String>,
    /// The body of the blog post.
    pub body: Option<String>,
}

/// Struct representing the editing of the blog post.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "diesel", derive(AsChangeset), table_name = "posts")]
pub struct Editing {
    /// The person who last updated the post.
    updated_by: uuid::Uuid,
}
impl Editing {
    /// Constructs the struct with assumed time of editing (now).
    pub fn new(updated_by: uuid::Uuid) -> Self {
        Self { updated_by }
    }
}

/// Struct representing the publishing of the blog post.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "diesel", derive(AsChangeset), table_name = "posts")]
pub struct Publishing {
    /// The person who last updated the post.
    updated_by: uuid::Uuid,
    /// The time at which the record was deleted.
    published_at: DateTime<Utc>,
    /// The id of the user who "deleted" the record.
    published_by: uuid::Uuid,
}
impl Publishing {
    /// Constructs the struct with assumed time of publishing (now).
    pub fn new(published_by: uuid::Uuid) -> Self {
        Self {
            updated_by: published_by,
            published_at: Utc::now(),
            published_by,
        }
    }
}

/// Struct representing the archival of the blog post.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "diesel", derive(AsChangeset), table_name = "posts")]
pub struct Archival {
    /// The person who last updated the post.
    updated_by: uuid::Uuid,
    /// The time at which the record was deleted.
    archived_at: DateTime<Utc>,
    /// The id of the user who "deleted" the record.
    archived_by: uuid::Uuid,
}
impl Archival {
    /// Constructs the struct with assumed time of archival (now).
    pub fn new(archived_by: uuid::Uuid) -> Self {
        Self {
            updated_by: archived_by,
            archived_at: Utc::now(),
            archived_by,
        }
    }
}

/// Struct representing the deletion operation on the struct.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "diesel", derive(AsChangeset), table_name = "posts")]
pub struct Deletion {
    /// The person who last updated the post.
    updated_by: uuid::Uuid,
    /// The time at which the record was deleted.
    deleted_at: DateTime<Utc>,
    /// The id of the user who "deleted" the record.
    deleted_by: uuid::Uuid,
}
impl Deletion {
    /// Constructs the struct with assumed time of deletion (now).
    pub fn new(deleted_by: uuid::Uuid) -> Self {
        Self {
            updated_by: deleted_by,
            deleted_at: Utc::now(),
            deleted_by,
        }
    }
}
