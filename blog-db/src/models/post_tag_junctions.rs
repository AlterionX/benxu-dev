//! A collection of types related to the many to many relation between posts and tags.

use serde::{Deserialize, Serialize};
#[cfg(feature = "diesel")]
use crate::schema::*;

#[derive(Serialize, Deserialize)]
/// Data representing a single post to tag relation.
#[cfg_attr(
    feature = "diesel",
    derive(Identifiable, Queryable, Associations),
    belongs_to(super::posts::Post),
    belongs_to(super::tags::Tag),
)]
pub struct PostTagJunction {
    /// A primary key composed of (post's id, tag's id) to ensure that the relation is unique.
    #[cfg_attr(feature = "diesel", primary_key(nonstandard))]
    pub id: (uuid::Uuid, uuid::Uuid),
    /// The post id represented by this relation.
    pub post_id: uuid::Uuid,
    /// The tag id represented by this relation.
    pub tag_id: uuid::Uuid,
    /// The user id of the creator of this relation.
    pub created_by: uuid::Uuid,
}

/// A new post to tag relation.
#[derive(Serialize, Deserialize)]
#[cfg_attr(
    feature = "diesel",
    derive(Insertable),
    table_name = "post_tag_junctions",
)]
pub struct NewPostTagJunction {
    /// The post id represented by this relation.
    pub post_id: uuid::Uuid,
    /// The tag id represented by this relation.
    pub tag_id: uuid::Uuid,
    /// The user id of the creator of this relation.
    pub created_by: uuid::Uuid,
}

