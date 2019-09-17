//! A collection of types related to the many to many relation between posts and tags.

use crate::schema::*;
use serde::{Deserialize, Serialize};

/// Data representing a single post to tag relation.
#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(super::posts::Post)]
#[belongs_to(super::tags::Tag)]
pub struct PostTagJunction {
    /// A primary key composed of (post's id, tag's id) to ensure that the relation is unique.
    #[primary_key(nonstandard)]
    pub id: (uuid::Uuid, uuid::Uuid),
    /// The post id represented by this relation.
    pub post_id: uuid::Uuid,
    /// The tag id represented by this relation.
    pub tag_id: uuid::Uuid,
    /// The user id of the creator of this relation.
    pub created_by: uuid::Uuid,
}

/// A new post to tag relation.
#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "post_tag_junctions"]
pub struct NewPostTagJunction {
    /// The post id represented by this relation.
    pub post_id: uuid::Uuid,
    /// The tag id represented by this relation.
    pub tag_id: uuid::Uuid,
    /// The user id of the creator of this relation.
    pub created_by: uuid::Uuid,
}

